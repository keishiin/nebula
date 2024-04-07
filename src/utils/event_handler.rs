use crate::{Data, Error};
use ::serenity::all::Mentionable;
use poise::serenity_prelude as serenity;
use serenity::FullEvent;

pub async fn event_handler(
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

            let db = &framework.user_data.db;

            let _ = sqlx::query!(
                "
                CREATE TABLE IF NOT EXISTS leveling (
                    user_id VARCHAR NOT NULL,
                    guild_id VARCHAR NOT NULL,
                    level BIGINT,
                    curr_exp BIGINT,
                    next_lvl BIGINT,
                    msg_count BIGINT DEFAULT 0,
                    CONSTRAINT leveling_pkey PRIMARY KEY (user_id)
                )"
            )
            .execute(db)
            .await?;

            let _ = sqlx::query!(
                "
                CREATE TABLE IF NOT EXISTS economy (
                    user_id VARCHAR NOT NULL,
                    guild_id VARCHAR NOT NULL,
                    job VARCHAR,
                    wallet BIGINT,
                    bank BIGINT,
                    CONSTRAINT ecnomy_pkey PRIMARY KEY (user_id, guild_id)
                )"
            )
            .execute(db)
            .await?;
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
                            ",
                            exp_next_level,
                            user_id,
                            guild_id
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
