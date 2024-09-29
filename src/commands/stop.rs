use unnamed_bot::types::{Context, Error};

/// Stop the playback
#[poise::command(
    slash_command,
    prefix_command,
    description_localized("es-ES", "detiene la reproducción")
)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("guild not found")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client not initialized")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if queue.len() > 0 {
            queue.stop();
            ctx.say("Stopped").await?;
        } else {
            ctx.say("Nothing on queue to stop").await?;
        }
    } else {
        ctx.say("Not in a voice channel").await?;
    }
    Ok(())
}
