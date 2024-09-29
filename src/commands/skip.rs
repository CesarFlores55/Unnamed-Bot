use unnamed_bot::types::{Context, Error};

/// Skip the current song
#[poise::command(
    slash_command,
    prefix_command,
    description_localized("es-ES", "salta la canción actual")
)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("guild not found")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client not initialized")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if queue.len() > 0 {
            queue.skip().expect("Error skipping");
            ctx.say("Skipped").await?;
        } else {
            ctx.say("Nothing on queue to skip").await?;
        }
    } else {
        ctx.say("Not in a voice channel").await?;
    }
    Ok(())
}
