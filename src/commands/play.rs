use songbird::input::{
    // HlsRequest,
    Input,
    // YoutubeDl as YoutubeDLP,
};

// use reqwest::Client;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use unnamed_bot::types::{Context, Error};

/// accepts a youtube url and plays the audio
#[poise::command(
    slash_command,
    prefix_command,
    description_localized("es-ES", "acepta un url de youtube y reproduce el audio")
)]
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

    let start_time = Instant::now();

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

    let audio_data = output.stdout;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird Voice client not initialized")?
        .clone();

    let handler = manager.join(guild_id, channel_id).await;

    match handler {
        Ok(handler) => {
            let mut handler_lock = handler.lock().await;
            let source = Input::from(audio_data);
            handler_lock.enqueue_input(source).await;

            let elapsed = start_time.elapsed();
            log::info!("yt-dlp executed in {:?}", elapsed);

            ctx.say(format!(
                "added to queue, position: {}",
                handler_lock.queue().len()
            ))
            .await
            .map_err(|e| {
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

    // // Verificar si la salida está vacía
    // if output.stdout.is_empty() {
    //     log::error!("yt-dlp output is empty");
    //     return Err(Error::from("yt-dlp output is empty"));
    // }

    // let audio_data = output.stdout;
    // log::info!("Downloaded audio data size: {} bytes", audio_data.len());

    // let mut child = Command::new("yt-dlp")
    // .args(&ytdl_args)
    // .stdout(std::process::Stdio::piped())
    // .spawn()
    // .map_err(|e| {
    //     log::error!("Failed to execute yt-dlp: {}", e);
    //     if e.kind() == std::io::ErrorKind::NotFound {
    //         Error::from("could not find executable 'yt-dlp' on path")
    //     } else {
    //         Error::from(e)
    //     }
    // })?;

    // let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;

    // let manager = songbird::get(ctx.serenity_context())
    //     .await
    //     .ok_or("Songbird Voice client not initialized")?
    //     .clone();

    // let handler = manager.join(guild_id, channel_id).await;

    // match handler {
    //     Ok(handler) => {
    //         let mut handler_lock = handler.lock().await;

    //         let start_time = Instant::now();

    //         let src = YoutubeDLP::new(Client::new(), url.clone());

    //         handler_lock.enqueue_input(src.into()).await;

    //         let elapsed = start_time.elapsed();
    //         log::info!("Enqueued audio in {:?}", elapsed);

    //         ctx.say(format!("added to queue, position: {}", handler_lock.queue().len()))
    //             .await
    //             .map_err(|e| {
    //                 log::error!("Failed to send message: {}", e);
    //                 e
    //             })?;
    //     }
    //     Err(e) => {
    //         ctx.say("Failed to join voice channel").await.map_err(|e| {
    //             log::error!("Failed to send message: {}", e);
    //             e
    //         })?;
    //         return Err(Box::new(e) as Error);
    //     }
    // }

    Ok(())
}
