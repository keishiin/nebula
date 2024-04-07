#![warn(clippy::str_to_string)]

mod commands;
mod utils;

use commands::economy::{bal, signup};
use commands::misc_commands::{age, avatar, help};
use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use sqlx::PgPool;
use utils::{event_handler::event_handler, on_error::on_error};

struct Data {
    pub db: PgPool,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let db_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), help(), avatar(), signup(), bal()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            on_error: |error| Box::pin(on_error(error)),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let pool = match PgPool::connect(&db_url).await {
                    Ok(pool) => pool,
                    Err(e) => {
                        println!("Error connecting to db: {}", e);
                        return Err(e.into());
                    }
                };

                Ok(Data { db: pool })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();

    Ok(())
}
