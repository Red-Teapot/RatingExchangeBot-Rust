use poise::serenity_prelude::{ChannelId, GuildId};
use sqlx::FromRow;

#[derive(FromRow)]
pub struct Exchange {
    pub id: u32,
    pub guild: GuildId,
    pub slug: String,
    pub display_name: String,
    pub submission_channel: ChannelId,
}
