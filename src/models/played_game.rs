use poise::serenity_prelude::UserId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayedGameId(pub u64);

#[derive(Debug, PartialEq, Eq)]
pub struct PlayedGame {
    pub id: PlayedGameId,
    pub link: String,
    pub member: UserId,
    pub is_manual: bool,
}
