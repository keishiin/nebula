mod commands;
mod utils;

use commands::economy::{bal, change_job, daily, deposit, jobs, signup, withdraw, work};
use commands::leveling::leaderboard;
use commands::misc_commands::{age, avatar, help};
use commands::pokemon::{get_pokemon_by_name, get_random_pokemon};
use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use sqlx::PgPool;
use std::collections::HashMap;
use utils::{event_handler::event_handler, on_error::on_error};

struct Data {
    pub db: PgPool,
    pub jobs: HashMap<&'static str, (i64, i64)>,
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
            commands: vec![
                // misc_commands
                age(),
                help(),
                avatar(),
                // economy commands
                signup(),
                bal(),
                withdraw(),
                deposit(),
                change_job(),
                jobs(),
                work(),
                daily(),
                // pokemon
                get_random_pokemon(),
                get_pokemon_by_name(),
                // leaderboard
                leaderboard(),
            ],
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
                let jobs = [
                    "Farmer",
                    "Trader",
                    "Craftsman",
                    "Medic",
                    "Engineer",
                    "Teacher",
                    "Entertainer",
                    "Security Guard",
                    "Chef",
                    "Artist",
                ];
                let incomes = [
                    5000, 10000, 15000, 20000, 25000, 30000, 35000, 40000, 45000, 50000,
                ];

                let mut job_map = HashMap::new();
                for (index, job) in jobs.iter().enumerate() {
                    job_map.insert(*job, ((index as i64) * 10, incomes[index]));
                }

                Ok(Data {
                    db: pool,
                    jobs: job_map,
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();

    Ok(())
}
