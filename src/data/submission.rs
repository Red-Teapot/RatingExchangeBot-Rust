use poise::serenity_prelude::UserId;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(FromRow)]
pub struct Submission {
    pub id: Option<u32>,
    pub exchange_round_id: Option<u32>,
    pub link: String,
    pub submitter: UserId,
    pub submitted_at: OffsetDateTime,
}
