use sqlx::FromRow;
use time::PrimitiveDateTime;

#[allow(dead_code)]
#[repr(u32)]
pub enum ExchangeState {
    NotStartedYet,
    AcceptingSubmissions,
    WaitingToSendAssignments,
    AssignmentsSent,
}

#[derive(FromRow)]
pub struct ExchangeRound {
    pub id: u32,
    pub exchange_id: u32,
    pub submissions_start_at: PrimitiveDateTime,
    pub submissions_end_at: PrimitiveDateTime,
    pub assignments_sent_at: PrimitiveDateTime,
    pub state: ExchangeState,
}
