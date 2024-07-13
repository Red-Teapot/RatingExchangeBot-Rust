use serenity::all::UserId;
use sqlx::{query, Pool, Sqlite};

use super::conversion::DBConvertible;

pub struct PlayedGameRepository {
    pool: Pool<Sqlite>,
}

impl PlayedGameRepository {
    pub fn new(pool: Pool<Sqlite>) -> PlayedGameRepository {
        PlayedGameRepository { pool }
    }

    pub async fn submit(&self, user: UserId, link: &str) -> Result<(), anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let user = user.to_db()?;
        query!(
            r#"
                INSERT INTO played_games (member, link, is_manual)
                VALUES ($1, $2, TRUE)
                ON CONFLICT (member, link) DO NOTHING
            "#,
            user,
            link,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }
}
