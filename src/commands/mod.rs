mod admin;
mod user;

use crate::BotState;

use anyhow::anyhow;

type Error = anyhow::Error;
type CommandResult = Result<(), Error>;
type Context<'a> = poise::Context<'a, BotState, Error>;
type ApplicationContext<'a> = poise::ApplicationContext<'a, BotState, Error>;

#[poise::command(
    slash_command,
    subcommands(
        "user::help",
        "user::submit",
        "user::revoke",
        "user::played",
        "admin::exchange",
    )
)]
pub async fn reb(_ctx: Context<'_>) -> CommandResult {
    Err(anyhow!("/reb root command should never be executed"))
}
