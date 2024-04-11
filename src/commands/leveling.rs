use crate::Error;
use ::serenity::all::Mentionable;
use poise::serenity_prelude as serenity;

pub async fn insert_or_update_leveling(
    db: &sqlx::PgPool,
    guild_id: &str,
    user_id: &str,
) -> Result<(), Error> {
    let res = sqlx::query!(
        "
        INSERT INTO leveling 
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
        sqlx::query!(
            "
            UPDATE leveling
            SET curr_exp = curr_exp + 10, msg_count = msg_count + 1
            WHERE user_id = $1 AND guild_id = $2
            ",
            user_id,
            guild_id
        )
        .execute(db)
        .await?;
    }

    Ok(())
}

pub async fn check_level_up(
    ctx: &serenity::Context,
    new_message: &serenity::Message,
    db: &sqlx::PgPool,
    guild_id: &str,
    user_id: &str,
) -> Result<(), Error> {
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

    let exp_next_level = user_data.next_lvl.map(|value| value + 100);
    if user_data.curr_exp >= exp_next_level {
        sqlx::query!(
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

        let response = format!("{} you have leveled up! ", new_message.author.mention());
        new_message.reply(ctx, response).await?;
    }

    Ok(())
}
