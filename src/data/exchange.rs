use poise::serenity_prelude::{ChannelId, GuildId};

use crate::jam_types::JamType;

use super::types::Sqlx;

#[derive(Clone, Debug)]
pub struct Exchange {
    pub id: i32,
    pub guild: Sqlx<GuildId>,
    pub jam_type: JamType,
    pub jam_link: String,
    pub slug: String,
    pub display_name: String,
    pub submission_channel: Sqlx<ChannelId>,
}
