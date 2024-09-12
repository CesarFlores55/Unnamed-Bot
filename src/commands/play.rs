// slash command that executes a system command(yt-dlp) and returns the output, expects an url(youtube) as argument
// the args for yt-dlp are -j -f ba[abr>0][vcodec=none]/best --no-playlist

use serde_json::Value;
use songbird::input::Input;
use tokio::time::{timeout, Duration};
use unnamed_bot::types::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Youtube url"] url: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("guild not found")?;

    let channel_id = ctx
        .guild()
        .and_then(|guild| {
            guild
                .voice_states
                .get(&ctx.author().id)
                .and_then(|vs| vs.channel_id)
        })
        .ok_or("you are not in a voice channel")?;

    log::info!("Received URL: {}", url);

    ctx.defer().await?;

    let ytdl_args = [
        "-j",
        "-f",
        "ba[abr>0][vcodec=none]/best",
        "--no-playlist",
        "--extract-audio",
        "--audio-format",
        "opus",
        "-o",
        "-",
        &url,
    ];

    let output = timeout(
        Duration::from_secs(30),
        tokio::process::Command::new("yt-dlp")
            .args(&ytdl_args)
            .output(),
    )
    .await
    .map_err(|e| {
        log::error!("yt-dlp command timed out: {}", e);
        Error::from("yt-dlp command timed out")
    })?
    .map_err(|e| {
        log::error!("Failed to execute yt-dlp: {}", e);
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::from("could not find executable 'yt-dlp' on path")
        } else {
            Error::from(e)
        }
    })?;

    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr[..]).unwrap_or("<no error message>");
        log::error!(
            "yt-dlp failed with status code {}: {}",
            output.status,
            stderr
        );
        return Err(Error::from(format!(
            "yt-dlp failed with non-zero status code: {}",
            stderr
        )));
    }

    // Verificar si la salida está vacía
    if output.stdout.is_empty() {
        log::error!("yt-dlp output is empty");
        return Err(Error::from("yt-dlp output is empty"));
    }

    // Parsear la salida JSON para extraer el título
    let stdout = std::str::from_utf8(&output.stdout[..]).unwrap_or("{}");
    let json: Value = serde_json::from_str(stdout).map_err(|e| {
        log::error!("Failed to parse yt-dlp output as JSON: {}", e);
        Error::from("Failed to parse yt-dlp output as JSON")
    })?;
    let title = json["title"].as_str().unwrap_or("Unknown title");

    let audio_data = output.stdout;
    log::info!("Downloaded audio data size: {} bytes", audio_data.len());

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird Voice client not initialized")?
        .clone();

    let handler = manager.join(guild_id, channel_id).await;

    match handler {
        Ok(handler) => {
            let mut handler = handler.lock().await;
            let source = Input::from(audio_data);
            handler.enqueue_input(source).await;

            ctx.say(format!("Playing: {}", title)).await.map_err(|e| {
                log::error!("Failed to send message: {}", e);
                e
            })?;
        }
        Err(e) => {
            ctx.say("Failed to join voice channel").await.map_err(|e| {
                log::error!("Failed to send message: {}", e);
                e
            })?;
            return Err(Box::new(e) as Error);
        }
    }

    Ok(())
}
