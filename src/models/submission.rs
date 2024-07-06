use poise::serenity_prelude::UserId;

use super::{exchange::ExchangeId, types::UtcDateTime};

#[derive(Clone, Copy, Debug)]
pub struct SubmissionId(pub u64);

#[derive(Debug)]
pub struct Submission {
    pub id: SubmissionId,
    pub exchange_id: ExchangeId,
    pub link: String,
    pub submitter: UserId,
    pub submitted_at: UtcDateTime,
}

// TODO: Find a way to avoid such copy-paste
#[derive(Debug)]
pub struct NewSubmission {
    pub exchange_id: ExchangeId,
    pub link: String,
    pub submitter: UserId,
    pub submitted_at: UtcDateTime,
}
