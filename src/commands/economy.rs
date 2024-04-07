use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use serenity::Colour;

#[poise::command(slash_command, prefix_command)]
pub async fn signup(
    ctx: Context<'_>,
    #[description = "create a user in the  economy"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.as_ref().unwrap_or_else(|| ctx.author());
    let guild = ctx
        .guild_id()
        .unwrap()
        .to_partial_guild(&ctx.http())
        .await?;
    let db = &ctx.data().db;

    let rows = sqlx::query!(
        "
        INSERT into economy (user_id, guild_id, job, wallet, bank) 
        VALUES ($1, $2, $3, $4, $5) 
        ON CONFLICT (user_id, guild_id) DO NOTHING",
        user.id.to_string(),
        guild.id.to_string(),
        "no job",
        0,
        0
    )
    .execute(db)
    .await?;

    if rows.rows_affected() == 0 {
        let embed = serenity::CreateEmbed::new()
            .thumbnail(user.avatar_url().unwrap_or_default())
            .title(format!("{}'s bank info", user.name))
            .description("You already have a account!")
            .color(Colour::BLUE);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let embed = serenity::CreateEmbed::new()
        .thumbnail(user.avatar_url().unwrap_or_default())
        .title(format!("{}'s bank info", user.name))
        .description("You already have a account!")
        .color(Colour::BLUE);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, user_cooldown = 10)]
pub async fn bal(ctx: Context<'_>) -> Result<(), Error> {
    let user = ctx.author();
    let guild = ctx
        .guild_id()
        .unwrap()
        .to_partial_guild(&ctx.http())
        .await?;
    let db = &ctx.data().db;

    let record = sqlx::query!(
        "SELECT wallet, job, bank 
        FROM economy 
        WHERE user_id = $1 AND guild_id = $2",
        user.id.to_string(),
        guild.id.to_string()
    )
    .fetch_one(db)
    .await?;

    let wallet_balance = match record.wallet {
        Some(bal) => bal.to_string(),
        None => "0".to_string(),
    };

    let bank_balance = match record.bank {
        Some(bal) => bal.to_string(),
        None => "0".to_string(),
    };

    let job_title = match record.job {
        Some(job) => job.to_string(),
        None => "No job".to_string(),
    };

    let embed = serenity::CreateEmbed::new()
        .thumbnail(user.avatar_url().unwrap_or_default())
        .title(format!("{}'s Bank Information", user.name))
        .field("Wallet Balance", wallet_balance, false)
        .field("Bank Balance", bank_balance, false)
        .field("Job", job_title, false)
        .color(Colour::BLUE);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
