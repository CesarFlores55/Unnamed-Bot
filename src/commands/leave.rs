//slash command to leave the voice channel if the bot is in one, if not return a error message
use unnamed_bot::types::{Context, Error};

#[poise::command(slash_command, prefix_command, description_localized("en-US","Leave the voice channel"))]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("guild not found")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird Voice client not initialized")?
        .clone();

    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id).await?;
        ctx.say("Left the voice channel!").await?;
        Ok(())
    } else {
        ctx.say("I'm not in a voice channel!").await?;
        Err("I'm not in a voice channel!".into())
    }
}