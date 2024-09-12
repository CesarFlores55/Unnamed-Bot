use poise::serenity_prelude;
use songbird::SerenityInit;
use std::{
    env,
    sync::{Arc, Mutex},
};

mod commands;
use crate::commands::{join, leave, play};
use unnamed_bot::types::Data;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity_prelude::GatewayIntents::non_privileged()
        | serenity_prelude::GatewayIntents::GUILD_MEMBERS
        | serenity_prelude::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![play::play(), join::join(), leave::leave()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!!".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity_prelude::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    let shard_manager = Arc::new(Mutex::new(client.shard_manager.clone()));

    tokio::spawn(async move {
        if let Err(err) = client.start().await {
            eprintln!("Client error: {:?}", err);
        }
    });

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    println!("Received Ctrl+C, shutting down...");
    shard_manager.lock().unwrap().shutdown_all().await;
}
