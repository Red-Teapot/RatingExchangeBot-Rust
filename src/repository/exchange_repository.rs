use std::num::NonZeroU8;

use poise::serenity_prelude::{ChannelId, GuildId};
use sqlx::{query, query_as, query_scalar, Pool, Sqlite};
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::warn;

use crate::{
    jam_types::JamType,
    models::{types::UtcDateTime, Exchange, ExchangeId, ExchangeState, NewExchange},
};

use super::conversion::{DBConvertible, DBFromConversionError};

#[derive(Debug)]
pub struct ExchangeRepository {
    pool: Pool<Sqlite>,
    events: Sender<ExchangeStorageEvent>,
}

#[derive(Clone, Copy, Debug)]
pub enum ExchangeStorageEvent {
    ExchangesUpdated,
}

impl ExchangeRepository {
    pub fn new(pool: Pool<Sqlite>) -> ExchangeRepository {
        ExchangeRepository {
            pool,
            events: tokio::sync::broadcast::channel(128).0,
        }
    }

    pub async fn create_exchange(&self, exchange: NewExchange) -> Result<Exchange, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let created_exchange = {
            let guild = exchange.guild.to_db()?;
            let channel = exchange.channel.to_db()?;
            let jam_type = exchange.jam_type.to_db()?;
            let state = ExchangeState::NotStartedYet.to_db()?;
            let submissions_start = exchange.submissions_start.to_db()?;
            let submissions_end = exchange.submissions_end.to_db()?;
            let games_per_member = exchange.games_per_member.to_db()?;

            query_as!(
                SqlExchange,
                r#"
                INSERT INTO exchanges (
                    guild,
                    channel,
                    jam_type,
                    jam_link,
                    slug,
                    display_name,
                    state,
                    submissions_start,
                    submissions_end,
                    games_per_member)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING
                    id AS "id!",
                    guild AS "guild!",
                    channel AS "channel!",
                    jam_type AS "jam_type!",
                    jam_link AS "jam_link!",
                    slug AS "slug!",
                    display_name AS "display_name!",
                    state AS "state!",
                    submissions_start AS "submissions_start!",
                    submissions_end AS "submissions_end!",
                    games_per_member AS "games_per_member!"
                "#,
                guild,
                channel,
                jam_type,
                exchange.jam_link,
                exchange.slug,
                exchange.display_name,
                state,
                submissions_start,
                submissions_end,
                games_per_member,
            )
            .fetch_one(&mut *transaction)
            .await?
        };

        transaction.commit().await?;

        // Don't care if it actually gets received
        let _ = self.events.send(ExchangeStorageEvent::ExchangesUpdated);

        Ok(Exchange::from_db(&created_exchange)?)
    }

    pub async fn get_overlapping_exchanges(
        &self,
        guild: GuildId,
        channel: ChannelId,
        slug: &str,
        start: UtcDateTime,
        end: UtcDateTime,
    ) -> Result<Vec<Exchange>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let overlapping_exchanges = {
            let guild = guild.to_db()?;
            let channel = channel.to_db()?;
            let start = start.to_db()?;
            let end = end.to_db()?;

            query_as!(
                SqlExchange,
                r#"
                SELECT * FROM exchanges
                WHERE (guild = $1 AND channel = $2 AND submissions_start < $4 AND submissions_end > $3)
                    OR (guild = $1 AND slug = $5)
                "#,
                guild,
                channel,
                start,
                end,
                slug,
            )
            .fetch_all(&mut *transaction)
            .await?
        };

        transaction.commit().await?;

        let overlapping_exchanges: Result<Vec<Exchange>, DBFromConversionError> =
            overlapping_exchanges
                .iter()
                .map(|e| Exchange::from_db(e))
                .collect();
        Ok(overlapping_exchanges?)
    }

    pub async fn get_running_exchange(
        &self,
        guild: GuildId,
        channel: ChannelId,
        date: UtcDateTime,
    ) -> Result<Option<Exchange>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let running_exchange = {
            let guild = guild.to_db()?;
            let channel = channel.to_db()?;
            let date = date.to_db()?;
            let accepting_submissions = ExchangeState::AcceptingSubmissions.to_db()?;

            query_as!(
                SqlExchange,
                r#"
                SELECT * FROM exchanges
                WHERE guild = $1 
                    AND channel = $2 
                    AND submissions_start <= $3 
                    AND submissions_end >= $3
                    AND state = $4
                "#,
                guild,
                channel,
                date,
                accepting_submissions,
            )
            .fetch_optional(&mut *transaction)
            .await?
        };

        transaction.commit().await?;

        Ok(running_exchange
            .map(|e| Exchange::from_db(&e))
            .transpose()?)
    }

    pub async fn get_upcoming_exchanges_in_guild(
        &self,
        guild: GuildId,
        after: UtcDateTime,
    ) -> Result<Vec<Exchange>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let upcoming_exchanges = {
            let guild = guild.to_db()?;
            let after = after.to_db()?;

            query_as!(
                SqlExchange,
                r#"
                SELECT * FROM exchanges
                WHERE guild = $1 AND submissions_end > $2
                ORDER BY submissions_start, display_name
                "#,
                guild,
                after,
            )
            .fetch_all(&mut *transaction)
            .await?
        };

        transaction.commit().await?;

        let upcoming_exchanges: Result<Vec<Exchange>, DBFromConversionError> = upcoming_exchanges
            .iter()
            .map(|e| Exchange::from_db(e))
            .collect();
        Ok(upcoming_exchanges?)
    }

    pub async fn get_starting_exchanges(
        &self,
        date: UtcDateTime,
    ) -> Result<Vec<Exchange>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let starting_exchanges = {
            let not_started_yet = ExchangeState::NotStartedYet.to_db()?;
            let date = date.to_db()?;

            query_as!(
                SqlExchange,
                r#"
                SELECT * FROM exchanges
                WHERE state = $1 AND submissions_start <= $2
                ORDER BY submissions_start, guild
                "#,
                not_started_yet,
                date,
            )
            .fetch_all(&mut *transaction)
            .await?
        };

        transaction.commit().await?;

        let starting_exchanges: Result<Vec<Exchange>, DBFromConversionError> = starting_exchanges
            .iter()
            .map(|e| Exchange::from_db(e))
            .collect();
        Ok(starting_exchanges?)
    }

    pub async fn get_ending_exchanges(
        &self,
        date: UtcDateTime,
    ) -> Result<Vec<Exchange>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let ending_exchanges = {
            let accepting_submissions = ExchangeState::AcceptingSubmissions.to_db()?;
            let date = date.to_db()?;

            query_as!(
                SqlExchange,
                r#"
                SELECT * FROM exchanges
                WHERE state = $1 AND submissions_end <= $2
                ORDER BY submissions_end, guild
                "#,
                accepting_submissions,
                date,
            )
            .fetch_all(&mut *transaction)
            .await?
        };

        transaction.commit().await?;

        let ending_exchanges: Result<Vec<Exchange>, DBFromConversionError> = ending_exchanges
            .iter()
            .map(|e| Exchange::from_db(e))
            .collect();
        Ok(ending_exchanges?)
    }

    pub async fn get_closest_exchange_end_or_start_date(
        &self,
    ) -> Result<Option<UtcDateTime>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let upcoming_exchange_date = {
            let not_started_yet = ExchangeState::NotStartedYet.to_db()?;
            let accepting_submissions = ExchangeState::AcceptingSubmissions.to_db()?;

            query_scalar!(
                r#"
                SELECT
                    IIF(state = $1, submissions_start, submissions_end) AS "closest_date!"
                FROM exchanges
                WHERE state IN ($2, $3)
                ORDER BY "closest_date!"
                LIMIT 1
                "#,
                not_started_yet,
                not_started_yet,
                accepting_submissions,
            )
            .fetch_optional(&mut *transaction)
            .await?
        };

        transaction.commit().await?;

        match upcoming_exchange_date {
            Some(date) => Ok(Some(UtcDateTime::from_db(&date)?)),
            None => Ok(None),
        }
    }

    pub async fn update_exchange_state(
        &self,
        exchange_id: ExchangeId,
        state: ExchangeState,
    ) -> Result<(), anyhow::Error> {
        let mut transaction = self.pool.begin().await?;
        let state = state.to_db()?;
        let exchange_id = exchange_id.to_db()?;

        query!(
            r#"
            UPDATE exchanges SET state = $1 WHERE id = $2
            "#,
            state,
            exchange_id,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn delete_exchange(&self, guild: GuildId, slug: &str) -> Result<bool, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let guild = guild.to_db()?;
        let query_result = query!(
            r#"DELETE FROM exchanges WHERE guild = $1 AND slug = $2"#,
            guild,
            slug,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        let _ = self.events.send(ExchangeStorageEvent::ExchangesUpdated); // Don't care if it actually gets received

        let exchanges_deleted = query_result.rows_affected();

        if exchanges_deleted > 1 {
            warn!("Deleted more than one exchange. Guild: {guild}, slug: {slug}");
        }

        Ok(exchanges_deleted > 0)
    }

    pub fn subscribe(&self) -> Receiver<ExchangeStorageEvent> {
        self.events.subscribe()
    }
}

pub struct SqlExchange {
    id: i64,
    guild: i64,
    channel: i64,
    jam_type: String,
    jam_link: String,
    slug: String,
    display_name: String,
    state: String,
    submissions_start: String,
    submissions_end: String,
    games_per_member: i64,
}

impl DBConvertible for Exchange {
    type DBType = SqlExchange;

    fn to_db(&self) -> Result<Self::DBType, super::conversion::DBToConversionError> {
        Ok(SqlExchange {
            id: self.id.to_db()?,
            guild: self.guild.to_db()?,
            channel: self.channel.to_db()?,
            jam_type: self.jam_type.to_db()?,
            jam_link: self.jam_link.clone(),
            slug: self.slug.clone(),
            display_name: self.display_name.clone(),
            state: self.state.to_db()?,
            submissions_start: self.submissions_start.to_db()?,
            submissions_end: self.submissions_end.to_db()?,
            games_per_member: self.games_per_member.to_db()?,
        })
    }

    fn from_db(value: &Self::DBType) -> Result<Self, super::conversion::DBFromConversionError> {
        Ok(Exchange {
            id: ExchangeId::from_db(&value.id)?,
            guild: GuildId::from_db(&value.guild)?,
            channel: ChannelId::from_db(&value.channel)?,
            jam_type: JamType::from_db(&value.jam_type)?,
            jam_link: value.jam_link.clone(),
            slug: value.slug.clone(),
            display_name: value.display_name.clone(),
            state: ExchangeState::from_db(&value.state)?,
            submissions_start: UtcDateTime::from_db(&value.submissions_start)?,
            submissions_end: UtcDateTime::from_db(&value.submissions_end)?,
            games_per_member: NonZeroU8::from_db(&value.games_per_member)?,
        })
    }
}
