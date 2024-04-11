use crate::commands::leveling::{check_level_up, insert_or_update_leveling};
use crate::utils::sql_helper::create_tables;
use crate::{Data, Error};
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

            create_tables(&framework.user_data.db).await?;
        }

        FullEvent::Message { new_message } => {
            println!("{}: {}", new_message.author.name, new_message.content);

            let guild_id = new_message.guild_id.unwrap().to_string();
            let user_id = new_message.author.id.to_string();
            let db = &framework.user_data.db;

            if new_message.author.id != 1225575257453232191 {
                insert_or_update_leveling(db, &guild_id, &user_id).await?;
                check_level_up(ctx, new_message, db, &guild_id, &user_id).await?;
            }
        }

        _ => {}
    }
    Ok(())
}
