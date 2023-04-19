mod camel_slug;

mod admin;
mod arguments;
mod user;

use crate::BotState;

use anyhow::anyhow;

type CommandResult = Result<(), CommandError>;
type Context<'a> = poise::Context<'a, BotState, CommandError>;
type ApplicationContext<'a> = poise::ApplicationContext<'a, BotState, CommandError>;

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("{message}")]
    InvalidArgument { message: String },
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[poise::command(
    slash_command,
    subcommands(
        "user::help",
        "user::submit",
        "user::revoke",
        "user::played",
        "admin::exchange",
    ),
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn reb(_ctx: Context<'_>) -> CommandResult {
    Err(anyhow!("/reb root command should never be executed"))?
}
