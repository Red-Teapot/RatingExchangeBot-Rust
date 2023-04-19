use super::CommandError;

mod exchange_slug;
mod human_datetime;
mod human_duration;
mod trimmed_string;

pub use exchange_slug::ExchangeSlug;
pub use human_datetime::HumanDateTime;
pub use human_duration::HumanDuration;
pub use trimmed_string::TrimmedString;

pub fn invalid_argument(message: String) -> CommandError {
    CommandError::InvalidArgument { message }
}
