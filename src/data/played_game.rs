use poise::serenity_prelude::UserId;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct PlayedGame {
    pub id: u32,
    pub link: String,
    pub member: UserId,
    pub is_manual: bool,
}
