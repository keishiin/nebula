use crate::Error;

pub async fn create_tables(db: &sqlx::PgPool) -> Result<(), Error> {
    sqlx::query!(
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

    sqlx::query!(
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

    Ok(())
}
