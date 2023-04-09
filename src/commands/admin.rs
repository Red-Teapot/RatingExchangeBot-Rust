use poise::{serenity_prelude::Channel, ChoiceParameter};

use crate::commands::*;

#[poise::command(
    slash_command,
    guild_only,
    subcommands("exchange_create"),
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn exchange(_ctx: Context<'_>) -> CommandResult {
    Err(anyhow!("/reb exchange command should never be executed"))
}

#[derive(ChoiceParameter)]
pub enum JamType {
    #[name = "Itch.io jam"]
    Itch,
    #[name = "Ludum Dare"]
    LudumDare,
}

/// Create a rating exchange
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, rename = "create")]
pub async fn exchange_create(
    ctx: ApplicationContext<'_>,

    #[rename = "type"]
    #[description = "The jam type"]
    _jam_type: JamType,

    #[rename = "link"]
    #[description = "The jam link. Must correspond to the jam type"]
    _jam_link: String,

    #[description = "The channel to post round announcements in"] _submission_channel: Channel,

    #[description = "The number of rounds. Defaults to 5"]
    #[min = 1]
    #[max = 255]
    _rounds: Option<u8>,

    #[description = "The number of games assigned to each member. Defaults to 5"]
    #[min = 1]
    #[max = 255]
    _games_per_member: Option<u8>,

    #[description = "The date and time of the first round. Defaults to now"] _start: Option<String>,

    #[description = "The duration of submission period. May be shorter than the round duration. Defaults to 24 hours"]
    _submission_duration: Option<String>,

    #[description = "The duration of a round. Defaults to submission duration"]
    _round_duration: Option<String>,

    #[description = "The name of the exchange to use in commands. Must consist only of `A-Za-z0-9_-`"]
    _slug: Option<String>,

    #[description = "The display name of the exchange to use in announcements"]
    _display_name: Option<String>,
) -> CommandResult {
    ctx.say("Not implemented yet").await?;
    Ok(())
}
