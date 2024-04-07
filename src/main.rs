use ::serenity::all::Mentionable;
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
    ctx: &serenity::Context,
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

            if new_message.author.id != 1225575257453232191 {
                let res = sqlx::query!(
                    "
                    INSERT into leveling 
                    (user_id, guild_id, level, curr_exp, next_lvl, msg_count)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    ON CONFLICT (user_id) DO NOTHING
                    ",
                    user_id,
                    guild_id,
                    1,
                    0,
                    100,
                    0
                )
                .execute(db)
                .await?;

                if res.rows_affected() == 0 {
                    // update the msg count and curr exp for the user
                    let _ = sqlx::query!(
                        "
                        UPDATE leveling
                        SET curr_exp = curr_exp + 10, msg_count = msg_count + 1
                        WHERE user_id = $1 AND guild_id = $2
                        ",
                        user_id,
                        guild_id
                    )
                    .execute(db)
                    .await?
                    .rows_affected();

                    let user_data = sqlx::query!(
                        "
                        SELECT curr_exp, level, next_lvl FROM leveling
                        WHERE user_id = $1 AND guild_id = $2
                        ",
                        user_id,
                        guild_id
                    )
                    .fetch_one(db)
                    .await?;

                    if user_data.curr_exp >= user_data.next_lvl {
                        // calc exp required for next level
                        let exp_next_level = user_data.next_lvl.map(|value| value + 100);

                        // need to update the users level and curr_exp == 0, next_lvl = exp_next_level
                        let _ = sqlx::query!(
                            "
                            UPDATE leveling 
                            SET level = level + 1, curr_exp = 0, next_lvl = $1 
                            WHERE user_id = $2 AND guild_id = $3
                            ", exp_next_level, user_id, guild_id
                        )
                        .execute(db)
                        .await?;

                        // send the msg informing the user that they have leveled up
                        // let http = ctx.http();
                        let response =
                            format!("{} you have levled up! ", new_message.author.mention());
                        // http.say(new_message.channel_id, response).await?;

                        // i dont like this way of doing it
                        // TODO: find a way to use ctx.say instead
                        new_message.reply(ctx, response).await?;
                    }
                }
            }
        }

        _ => {}
    }
    Ok(())
}
