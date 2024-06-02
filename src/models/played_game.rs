use poise::serenity_prelude::UserId;
use sqlx::FromRow;

use super::types::Sqlx;

#[derive(FromRow)]
pub struct PlayedGame {
    pub id: i64,
    pub link: String,
    pub member: Sqlx<UserId>,
    pub is_manual: bool,
}
