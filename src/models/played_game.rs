use poise::serenity_prelude::UserId;

#[derive(Clone, Copy, Debug)]
pub struct PlayedGameId(pub u64);

#[derive(Debug)]
pub struct PlayedGame {
    pub id: PlayedGameId,
    pub link: String,
    pub member: UserId,
    pub is_manual: bool,
}
