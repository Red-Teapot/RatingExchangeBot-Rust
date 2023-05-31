use sqlx::{FromRow, Type};

use super::types::{Sqlx, UtcDateTime};

#[derive(Copy, Clone, Debug, Type)]
#[repr(i32)]
pub enum ExchangeRoundState {
    NotStartedYet,
    AcceptingSubmissions,
    WaitingToSendAssignments,
    AssignmentsSent,
}

#[derive(FromRow, Clone, Debug)]
pub struct ExchangeRound {
    pub id: i64,
    pub exchange_id: i64,
    pub submissions_start_at: UtcDateTime,
    pub submissions_end_at: UtcDateTime,
    pub assignments_sent_at: UtcDateTime,
    pub games_per_member: u32,
    pub state: Sqlx<ExchangeRoundState>,
}
