use poise::serenity_prelude::UserId;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(FromRow)]
pub struct Submission {
    pub id: i64,
    pub exchange_round_id: i64,
    pub link: String,
    pub submitter: UserId,
    pub submitted_at: OffsetDateTime,
}
