mod camel_slug;

mod arguments;
mod exchange;
mod played;
mod submit;

use crate::BotState;

pub use exchange::exchange;
pub use played::played;
pub use submit::submit;

type CommandResult = Result<(), CommandError>;
type Context<'a> = poise::Context<'a, BotState, CommandError>;
type ApplicationContext<'a> = poise::ApplicationContext<'a, BotState, CommandError>;

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("{message}")]
    UserError { message: String },
    #[error("{message}")]
    InternalError { message: String },
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
}

fn user_err(message: impl Into<String>) -> CommandError {
    CommandError::UserError {
        message: message.into(),
    }
}

fn internal_err(message: impl Into<String>) -> CommandError {
    CommandError::InternalError {
        message: message.into(),
    }
}
