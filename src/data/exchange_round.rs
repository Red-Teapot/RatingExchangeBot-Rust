use sqlx::{FromRow, Type};

use super::types::UtcDateTime;

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
    pub id: i32,
    pub exchange_id: i32,
    pub submissions_start_at: UtcDateTime,
    pub submissions_end_at: UtcDateTime,
    pub assignments_sent_at: UtcDateTime,
    pub games_per_member: i32,
    pub state: ExchangeRoundState,
}
