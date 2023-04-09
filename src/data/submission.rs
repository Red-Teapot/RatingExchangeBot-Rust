use poise::serenity_prelude::UserId;
use sqlx::FromRow;
use time::PrimitiveDateTime;

#[derive(FromRow)]
pub struct Submission {
    pub id: u32,
    pub exchange_round_id: u32,
    pub link: String,
    pub submitter: UserId,
    pub submitted_at: PrimitiveDateTime,
}
