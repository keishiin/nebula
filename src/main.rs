use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use serenity::model::mention::Mention;
use serenity::FullEvent;
use sqlx::PgPool;

mod commands;

struct Data {
    pub db: PgPool,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const _POKEURL: &str = "https://pokeapi.co/api/v2/pokemon/";

#[poise::command(slash_command, prefix_command)]
pub async fn avatar(
    ctx: Context<'_>,
    #[description = "users avatar"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    let embed = serenity::CreateEmbed::new()
        .title(format!("{}'s avatar", Mention::from(u.id)))
        .image(u.face())
        .color(serenity::Colour::BLUE);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    let response = format!("@{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;

    Ok(())
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let db_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), help(), avatar()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
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

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            println!(
                "Ready! Logged in as {}#{}",
                data_about_bot.user.name,
                data_about_bot.user.discriminator.unwrap()
            );
        }

        FullEvent::Message { new_message } => {
            println!("{}: {}", new_message.author.name, new_message.content);

            let guild_id = new_message.guild_id.unwrap().to_string();
            let user_id = new_message.author.id.to_string();

            let db = &framework.user_data.db;

            let res = sqlx::query!(
                "
                INSERT into leveling 
                (user_id, guild_id, level, curr_exp, next_lvl)
                VALUES ($1, $2, $3, $4, $5)
                ",
                user_id,
                guild_id,
                1,
                0,
                100
            )
            .execute(db)
            .await?;

            if res.rows_affected() == 1 {
                println!("New user added to db");
            }

            let res2 = sqlx::query!(
                "
                UPDATE leveling
                SET curr_exp = curr_exp + 10
                WHERE user_id = $1 AND guild_id = $2
                ",
                user_id,
                guild_id
            )
            .execute(db)
            .await?
            .rows_affected();

            if res2 == 1 {
                println!("Insertion successful!");
            } else {
                println!("Insertion failed!");
            }
        }

        _ => {}
    }
    Ok(())
}
