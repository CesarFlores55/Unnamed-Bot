// slash command to join a voice channel if the user is in one, if not return a error message

use unnamed_bot::types::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let (guild_id, channel_id) = {
        let guild = ctx.guild().ok_or("guild not found")?;
        let channel_id = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
            .ok_or("you are not in a voice channel")?;
        (guild.id, channel_id)
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird Voice client not initialized")?
        .clone();

    let handler = manager.join(guild_id, channel_id).await;

    match handler {
        Ok(_) => {
            ctx.say("Joined the voice channel!").await?;
            Ok(())
        }
        Err(e) => Err(Box::new(e) as Error),
    }
}
