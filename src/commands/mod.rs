mod camel_slug;

mod arguments;
mod exchange;
mod played;
mod revoke;
mod submit;

use crate::BotState;

pub use exchange::exchange;
pub use played::played;
pub use revoke::revoke;
pub use submit::submit;

type CommandResult = Result<(), CommandError>;
type Context<'a> = poise::Context<'a, BotState, CommandError>;
type ApplicationContext<'a> = poise::ApplicationContext<'a, BotState, CommandError>;

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("{message}")]
    User { message: String },
    #[error("{message}")]
    Internal { message: String },
    #[error(transparent)]
    Serenity(#[from] serenity::Error),
}

fn user_err(message: impl Into<String>) -> CommandError {
    CommandError::User {
        message: message.into(),
    }
}

fn internal_err(message: impl Into<String>) -> CommandError {
    CommandError::Internal {
        message: message.into(),
    }
}
