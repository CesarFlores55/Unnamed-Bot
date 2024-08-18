use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use std::{env, sync::{Arc, Mutex}};
use tokio::signal;

mod commands;
use unamed_bot::types::Data;
use crate::commands::age::age;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");
    // client.unwrap().start().await.unwrap();

    let shard_manager = Arc::new(Mutex::new(client.shard_manager.clone()));

    tokio::spawn(async move {
        if let Err(err) = client.start().await {
            eprintln!("Client error: {:?}", err);
        }
    });

    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

    println!("Received Ctrl+C, shutting down...");
    shard_manager.lock().unwrap().shutdown_all().await;
}