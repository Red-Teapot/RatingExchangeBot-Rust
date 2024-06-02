use poise::serenity_prelude::{ChannelId, GuildId, UserId};
use sqlx::{Database, Decode, Encode, Sqlite, Type};

use crate::{jam_types::JamType, models::ExchangeState};

/// A trait that converts the types into corresponding sqlx database types.
pub trait SqlxConvertible<'q, 'r, DB: Database> {
    type DBType: Type<DB> + Encode<'q, DB> + Decode<'r, DB>;

    fn to_sqlx(&self) -> Self::DBType;
    fn from_sqlx(value: Self::DBType) -> Self;
}

/// A wrapper around a type to make it compatible with `sqlx::query!` macros.
#[derive(Debug, Copy, Clone)]
pub struct Sqlx<T>(pub T);

impl<T> From<T> for Sqlx<T> {
    fn from(value: T) -> Self {
        Sqlx(value)
    }
}

impl<'q, 'r, T, DB> Type<DB> for Sqlx<T>
where
    DB: Database,
    T: SqlxConvertible<'q, 'r, DB>,
{
    fn type_info() -> <DB as Database>::TypeInfo {
        T::DBType::type_info()
    }
}

impl<'q, 'r, T, DB> Encode<'q, DB> for Sqlx<T>
where
    DB: Database,
    T: SqlxConvertible<'q, 'r, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        self.0.to_sqlx().encode_by_ref(buf)
    }
}

impl<'q, 'r, T, DB> Decode<'r, DB> for Sqlx<T>
where
    DB: Database,
    T: SqlxConvertible<'q, 'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        T::DBType::decode(value).map(|v| Sqlx(T::from_sqlx(v)))
    }
}

// SqlxConvertible implementations for types

impl SqlxConvertible<'_, '_, Sqlite> for GuildId {
    type DBType = i64;

    fn to_sqlx(&self) -> Self::DBType {
        self.0 as _
    }

    fn from_sqlx(value: Self::DBType) -> Self {
        GuildId(value as _)
    }
}

impl SqlxConvertible<'_, '_, Sqlite> for ChannelId {
    type DBType = i64;

    fn to_sqlx(&self) -> Self::DBType {
        self.0 as _
    }

    fn from_sqlx(value: Self::DBType) -> Self {
        ChannelId(value as _)
    }
}

impl SqlxConvertible<'_, '_, Sqlite> for UserId {
    type DBType = i64;

    fn to_sqlx(&self) -> Self::DBType {
        self.0 as _
    }

    fn from_sqlx(value: Self::DBType) -> Self {
        UserId(value as _)
    }
}

impl SqlxConvertible<'_, '_, Sqlite> for JamType {
    type DBType = String;

    fn to_sqlx(&self) -> Self::DBType {
        match self {
            JamType::Itch => "Itch".to_string(),
            JamType::LudumDare => "LudumDare".to_string(),
        }
    }

    fn from_sqlx(value: Self::DBType) -> Self {
        match value.as_str() {
            "Itch" => JamType::Itch,
            "LudumDare" => JamType::LudumDare,
            _ => panic!("Unexpected jam type value: {value}"),
        }
    }
}

impl SqlxConvertible<'_, '_, Sqlite> for ExchangeState {
    type DBType = String;

    fn to_sqlx(&self) -> Self::DBType {
        use ExchangeState::*;

        match self {
            NotStartedYet => "NotStartedYet".to_string(),
            AcceptingSubmissions => "AcceptingSubmissions".to_string(),
            AssignmentsSent => "AssignmentsSent".to_string(),
        }
    }

    fn from_sqlx(value: Self::DBType) -> Self {
        use ExchangeState::*;

        match value.as_str() {
            "NotStartedYet" => NotStartedYet,
            "AcceptingSubmissions" => AcceptingSubmissions,
            "AssignmentsSent" => AssignmentsSent,
            _ => panic!("Unexpected exchange round state value: {value}"),
        }
    }
}
