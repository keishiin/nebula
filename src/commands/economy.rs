use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use serenity::Colour;
use serenity::CreateEmbed;

#[poise::command(slash_command)]
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
        let embed = CreateEmbed::new()
            .thumbnail(user.avatar_url().unwrap_or_default())
            .title(format!("{}'s bank info", user.name))
            .description("You already have a account!")
            .color(Colour::BLUE);
        ctx.send(CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let embed = CreateEmbed::new()
        .thumbnail(user.avatar_url().unwrap_or_default())
        .title(format!("{}'s bank info", user.name))
        .description("You already have a account!")
        .color(Colour::BLUE);
    ctx.send(CreateReply::default().embed(embed)).await?;

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

    let embed = CreateEmbed::new()
        .thumbnail(user.avatar_url().unwrap_or_default())
        .title(format!("{}'s Bank Information", user.name))
        .field(
            "Wallet Balance",
            record.wallet.unwrap_or(0).to_string(),
            false,
        )
        .field("Bank Balance", record.bank.unwrap_or(0).to_string(), false)
        .field(
            "Job",
            record.job.unwrap_or("No Job".to_string()).to_string(),
            false,
        )
        .color(Colour::BLUE);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn deposit(
    ctx: Context<'_>,
    #[description = "amount to deposit"] amt: i64,
) -> Result<(), Error> {
    let user = ctx.author();
    let guild = ctx
        .guild_id()
        .unwrap()
        .to_partial_guild(&ctx.http())
        .await?;
    let db = &ctx.data().db;

    let record = sqlx::query!(
        "SELECT wallet from economy WHERE user_id = $1 AND guild_id = $2",
        user.id.to_string(),
        guild.id.to_string()
    )
    .fetch_one(db)
    .await?;

    let wallet_balence = match record.wallet {
        Some(bal) => bal,
        None => {
            ctx.say("you do not have any balance in your wallet")
                .await?;
            return Ok(());
        }
    };

    if amt > wallet_balence {
        ctx.say("You are trying to deposit more money than you have in your wallet")
            .await?;
        return Ok(());
    }

    let result = sqlx::query!(
        "UPDATE economy 
        SET wallet = wallet - $1, bank = bank + $2 
        WHERE user_id = $3 AND guild_id = $4",
        amt,
        amt,
        user.id.to_string(),
        guild.id.to_string()
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        ctx.say("unable to deposit").await?;
        return Ok(());
    }

    ctx.say(format!("Deposited ${} into your bank", amt))
        .await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn withdraw(
    ctx: Context<'_>,
    #[description = "amount to withdraw"] amt: i64,
) -> Result<(), Error> {
    let user = ctx.author();
    let guild = ctx
        .guild_id()
        .unwrap()
        .to_partial_guild(&ctx.http())
        .await?;
    let db = &ctx.data().db;

    let record = sqlx::query!(
        "SELECT bank from economy WHERE user_id = $1 AND guild_id = $2",
        user.id.to_string(),
        guild.id.to_string()
    )
    .fetch_one(db)
    .await?;

    let bank_balence = match record.bank {
        Some(bal) => bal,
        None => {
            ctx.say("you do not have any balance in your bank").await?;
            return Ok(());
        }
    };

    if amt > bank_balence {
        ctx.say("You are trying to withdraw more money than you have in your bank")
            .await?;
        return Ok(());
    }

    let result = sqlx::query!(
        "UPDATE economy 
        SET wallet = wallet + $1, bank = bank - $2 
        WHERE user_id = $3 AND guild_id = $4",
        amt,
        amt,
        user.id.to_string(),
        guild.id.to_string()
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        ctx.say("unable to withdraw").await?;
        return Ok(());
    }

    ctx.say(format!("Withdrew ${} into your bank", amt)).await?;

    Ok(())
}

#[poise::command(slash_command, user_cooldown = 86400)]
pub async fn daily(ctx: Context<'_>) -> Result<(), Error> {
    let user = ctx.author();
    let guild = ctx
        .guild_id()
        .unwrap()
        .to_partial_guild(&ctx.http())
        .await?;
    let db = &ctx.data().db;

    let result = sqlx::query!(
        "UPDATE economy SET wallet = wallet + 10000 WHERE user_id = $1 AND guild_id = $2",
        user.id.to_string(),
        guild.id.to_string()
    )
    .execute(db)
    .await?
    .rows_affected();

    if result == 0 {
        ctx.say("Unable able to collect daily").await?;
        return Ok(());
    }

    ctx.say("$10000 added to your wallet").await?;

    Ok(())
}
