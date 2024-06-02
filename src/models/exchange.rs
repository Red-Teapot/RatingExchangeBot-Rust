use poise::serenity_prelude::{ChannelId, GuildId};
use sqlx::Type;

use crate::jam_types::JamType;

use super::types::{Sqlx, UtcDateTime};

#[derive(Clone, Debug)]
pub struct Exchange {
    pub id: i64,
    pub guild: Sqlx<GuildId>,
    pub channel: Sqlx<ChannelId>,
    pub jam_type: Sqlx<JamType>,
    pub jam_link: String,
    pub slug: String,
    pub display_name: String,
    pub state: Sqlx<ExchangeState>,
    pub submissions_start: UtcDateTime,
    pub submissions_end: UtcDateTime,
}

#[derive(Copy, Clone, Debug, Type)]
#[repr(i32)]
pub enum ExchangeState {
    NotStartedYet,
    AcceptingSubmissions,
    AssignmentsSent,
}
