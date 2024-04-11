use crate::{Context, Error};
use ::serenity::all::Mentionable;
use ::serenity::all::Timestamp;
use ::serenity::all::UserId;
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use serenity::Colour;
use serenity::CreateEmbed;

#[poise::command(slash_command)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;
    let mut description = String::new();
    let users = sqlx::query!("SELECT * FROM leveling ORDER BY level DESC")
        .fetch_all(db)
        .await?;

    for (i, user) in users.iter().enumerate() {
        let user_id = UserId::from(user.user_id.parse::<u64>().unwrap());
        description.push_str(&format!(
            "{place}. **{name}** info: {level} -> {xp}xp / {exp_required}xp \n",
            place = i + 1,
            name = user_id.to_user(&ctx).await?.global_name.unwrap(),
            level = user.level.unwrap_or(0),
            xp = user.curr_exp.unwrap_or(0),
            exp_required = user.next_lvl.map(|value| value + 100).unwrap_or(0)
        ))
    }

    let embed = CreateEmbed::new()
        .title("Leaderboard")
        .description(description)
        .timestamp(Timestamp::now())
        .color(Colour::BLUE);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

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
