use std::error::Error;

use poise::serenity_prelude::GuildId;
use serenity::all::RoleId;
use sqlx::{query, Pool, Sqlite};

use crate::repository::conversion::DBConvertible;

pub struct SettingRepository {
    pool: Pool<Sqlite>,
}

impl SettingRepository {
    pub fn new(pool: Pool<Sqlite>) -> SettingRepository {
        SettingRepository { pool }
    }

    pub async fn get<S: Setting>(&self, guild_id: GuildId) -> Result<Option<S>, Box<dyn Error>> {
        let mut transaction = self.pool.begin().await?;

        let value = {
            let guild_id = guild_id.to_db()?;
            let key = S::KEY;

            query!(
                r#"
                    SELECT value FROM guild_settings
                    WHERE guild = $1 AND key = $2 
                "#,
                guild_id,
                key,
            )
            .fetch_optional(&mut *transaction)
            .await?
            .map(|v| v.value)
        };

        transaction.commit().await?;

        value.map(|v| S::from_db(&v)).transpose()
    }

    pub async fn set<S: Setting>(&self, guild_id: GuildId, value: S) -> Result<(), Box<dyn Error>> {
        let mut transaction = self.pool.begin().await?;

        {
            let guild_id = guild_id.to_db()?;
            let key = S::KEY;
            let value = value.to_db()?;

            query!(
                r#"
                    INSERT INTO guild_settings (guild, key, value)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (guild, key) DO UPDATE SET value = $3;
                "#,
                guild_id,
                key,
                value,
            )
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }
}

pub trait Setting: Sized {
    const KEY: &'static str;

    fn to_db(&self) -> Result<String, Box<dyn Error>>;
    fn from_db(value: &str) -> Result<Self, Box<dyn Error>>;
}

pub struct ManagerRole {
    pub role: RoleId,
}

impl Setting for ManagerRole {
    const KEY: &'static str = "ManagerRole";

    fn to_db(&self) -> Result<String, Box<dyn Error>> {
        Ok(format!("{}", self.role.get()))
    }

    fn from_db(value: &str) -> Result<Self, Box<dyn Error>> {
        Ok(ManagerRole {
            role: RoleId::new(value.parse()?),
        })
    }
}
