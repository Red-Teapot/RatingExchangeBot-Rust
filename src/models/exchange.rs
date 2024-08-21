use std::num::NonZeroU8;

use poise::serenity_prelude::{ChannelId, GuildId};

use crate::jam_types::JamType;

use super::types::UtcDateTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExchangeId(pub u64);

#[derive(Debug)]
pub struct Exchange {
    pub id: ExchangeId,
    pub guild: GuildId,
    pub channel: ChannelId,
    pub jam_type: JamType,
    pub jam_link: String,
    pub slug: String,
    pub display_name: String,
    pub state: ExchangeState,
    pub submissions_start: UtcDateTime,
    pub submissions_end: UtcDateTime,
    pub games_per_member: NonZeroU8,
}

#[derive(Debug)]
pub struct NewExchange {
    pub guild: GuildId,
    pub channel: ChannelId,
    pub jam_type: JamType,
    pub jam_link: String,
    pub slug: String,
    pub display_name: String,
    pub state: ExchangeState,
    pub submissions_start: UtcDateTime,
    pub submissions_end: UtcDateTime,
    pub games_per_member: NonZeroU8,
}

#[derive(Clone, Copy, Debug)]
pub enum ExchangeState {
    NotStartedYet,
    AcceptingSubmissions,
    AssignmentsSent,
    MissedByBot,
    AssignmentError,
}
