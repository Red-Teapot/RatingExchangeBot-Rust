use std::num::NonZeroU8;

use poise::serenity_prelude::{ChannelId, GuildId, UserId};
use thiserror::Error;
use time::{format_description::well_known::Iso8601, OffsetDateTime};

use crate::{
    jam_types::JamType,
    models::{types::UtcDateTime, ExchangeId, ExchangeState, PlayedGameId, SubmissionId},
};

pub trait DBConvertible: Sized {
    type DBType;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError>;

    fn from_db(value: &Self::DBType) -> Result<Self, DBFromConversionError>;
}

#[derive(Debug, Error)]
pub enum DBFromConversionError {
    #[error("Failed to parse datetime: {0}")]
    DateTime(#[from] time::error::Parse),
    #[error("Failed to parse enum variant: {0}")]
    NoSuchVariant(String),
    #[error("Invalid number: {0}")]
    InvalidNumber(i64),
}

#[derive(Debug, Error)]
pub enum DBToConversionError {
    #[error("Failed to format datetime")]
    DateTime(#[from] time::error::Format),
}

impl DBConvertible for UtcDateTime {
    type DBType = String;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError> {
        let string = OffsetDateTime::from(*self).format(&Iso8601::DEFAULT)?;
        Ok(string)
    }

    fn from_db(db_value: &Self::DBType) -> Result<Self, DBFromConversionError> {
        let datetime = OffsetDateTime::parse(db_value, &Iso8601::DEFAULT)?;
        Ok(UtcDateTime::from(datetime))
    }
}

// TODO: Check i64 before conversion to avoid panic
impl DBConvertible for ExchangeId {
    type DBType = i64;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError> {
        Ok(self.0 as _)
    }

    fn from_db(value: &Self::DBType) -> Result<Self, DBFromConversionError> {
        Ok(ExchangeId(*value as _))
    }
}

impl DBConvertible for SubmissionId {
    type DBType = i64;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError> {
        Ok(self.0 as _)
    }

    fn from_db(value: &Self::DBType) -> Result<Self, DBFromConversionError> {
        Ok(SubmissionId(*value as _))
    }
}

impl DBConvertible for PlayedGameId {
    type DBType = i64;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError> {
        Ok(self.0 as _)
    }

    fn from_db(value: &Self::DBType) -> Result<Self, DBFromConversionError> {
        Ok(PlayedGameId(*value as _))
    }
}

impl DBConvertible for UserId {
    type DBType = i64;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError> {
        Ok(self.get() as _)
    }

    fn from_db(value: &Self::DBType) -> Result<Self, DBFromConversionError> {
        Ok(UserId::new(*value as _))
    }
}

impl DBConvertible for GuildId {
    type DBType = i64;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError> {
        Ok(self.get() as _)
    }

    fn from_db(value: &Self::DBType) -> Result<Self, DBFromConversionError> {
        Ok(GuildId::new(*value as _))
    }
}

impl DBConvertible for ChannelId {
    type DBType = i64;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError> {
        Ok(self.get() as _)
    }

    fn from_db(value: &Self::DBType) -> Result<Self, DBFromConversionError> {
        Ok(ChannelId::new(*value as _))
    }
}

impl DBConvertible for ExchangeState {
    type DBType = String;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError> {
        Ok(match self {
            ExchangeState::NotStartedYet => "NotStartedYet",
            ExchangeState::AcceptingSubmissions => "AcceptingSubmissions",
            ExchangeState::AssignmentsSent => "AssignmentsSent",
            ExchangeState::MissedByBot => "MissedByBot",
            ExchangeState::AssignmentError => "AssignmentError",
        }
        .to_string())
    }

    fn from_db(value: &Self::DBType) -> Result<Self, DBFromConversionError> {
        match value.as_str() {
            "NotStartedYet" => Ok(ExchangeState::NotStartedYet),
            "AcceptingSubmissions" => Ok(ExchangeState::AcceptingSubmissions),
            "AssignmentsSent" => Ok(ExchangeState::AssignmentsSent),
            "MissedByBot" => Ok(ExchangeState::MissedByBot),
            "AssignmentError" => Ok(ExchangeState::AssignmentError),

            unknown => Err(DBFromConversionError::NoSuchVariant(unknown.to_string())),
        }
    }
}

impl DBConvertible for JamType {
    type DBType = String;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError> {
        Ok(match self {
            JamType::Itch => "Itch",
            JamType::LudumDare => "LudumDare",
        }
        .to_string())
    }

    fn from_db(value: &Self::DBType) -> Result<Self, DBFromConversionError> {
        match value.as_str() {
            "Itch" => Ok(JamType::Itch),
            "LudumDare" => Ok(JamType::LudumDare),

            unknown => Err(DBFromConversionError::NoSuchVariant(unknown.to_string())),
        }
    }
}

impl DBConvertible for NonZeroU8 {
    type DBType = i64;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError> {
        Ok(self.get() as _)
    }

    fn from_db(value: &Self::DBType) -> Result<Self, DBFromConversionError> {
        if *value >= NonZeroU8::MIN.get() as _ && *value <= NonZeroU8::MAX.get() as _ {
            Ok(NonZeroU8::new(*value as _).expect("Checked by the guard"))
        } else {
            Err(DBFromConversionError::InvalidNumber(*value))
        }
    }
}
