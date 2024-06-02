use poise::serenity_prelude::UserId;
use sqlx::FromRow;

use super::types::UtcDateTime;

#[derive(FromRow)]
pub struct Submission {
    pub id: i64,
    pub exchange_id: i64,
    pub link: String,
    pub submitter: UserId,
    pub submitted_at: UtcDateTime,
}
