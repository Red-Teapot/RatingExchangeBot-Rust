use poise::serenity_prelude::{ChannelId, GuildId};
use sqlx::{query, query_as, Pool, Sqlite};
use time::Duration;
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::warn;

use crate::{
    jam_types::JamType,
    models::{
        types::{Sqlx, SqlxConvertible, UtcDateTime},
        Exchange, ExchangeState,
    },
};

#[derive(Debug)]
pub struct ExchangeRepository {
    pool: Pool<Sqlite>,
    events: Sender<ExchangeStorageEvent>,
}

pub struct CreateExchange {
    pub guild: GuildId,
    pub channel: ChannelId,
    pub jam_type: JamType,
    pub jam_link: String,
    pub slug: String,
    pub display_name: String,
    pub start: UtcDateTime,
    pub duration: Duration,
    pub games_per_member: u8,
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

    pub async fn create_exchange(
        &self,
        create_exchange: CreateExchange,
    ) -> Result<Exchange, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let state = ExchangeState::NotStartedYet.to_sqlx();
        let end = create_exchange.start + create_exchange.duration;

        let exchange = {
            let guild = create_exchange.guild.to_sqlx();
            let jam_type = create_exchange.jam_type.to_sqlx();
            let channel = create_exchange.channel.to_sqlx();

            query_as!(
                Exchange,
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
                    submissions_end)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING
                    id,
                    guild as "guild!: Sqlx<GuildId>",
                    channel as "channel!: Sqlx<ChannelId>",
                    jam_type as "jam_type!: Sqlx<JamType>",
                    jam_link as "jam_link!",
                    slug as "slug!",
                    display_name as "display_name!",
                    state as "state!: Sqlx<ExchangeState>",
                    submissions_start as "submissions_start!: UtcDateTime",
                    submissions_end as "submissions_end!: UtcDateTime"
                "#,
                guild,
                channel,
                jam_type,
                create_exchange.jam_link,
                create_exchange.slug,
                create_exchange.display_name,
                state,
                create_exchange.start,
                end,
            )
            .fetch_one(&mut transaction)
            .await?
        };

        transaction.commit().await?;

        let _ = self.events.send(ExchangeStorageEvent::ExchangesUpdated); // Don't care if it actually gets received

        Ok(exchange)
    }

    pub async fn get_overlapping_exchanges(
        &self,
        guild: GuildId,
        channel: ChannelId,
        start: UtcDateTime,
        end: UtcDateTime,
    ) -> Result<Vec<Exchange>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let overlapping_exchanges = {
            let guild = guild.to_sqlx();
            let channel = channel.to_sqlx();

            query_as!(
                Exchange,
                r#"
                SELECT
                    id,
                    guild as "guild!: Sqlx<GuildId>",
                    channel as "channel!: Sqlx<ChannelId>",
                    jam_type as "jam_type!: Sqlx<JamType>",
                    jam_link as "jam_link!",
                    slug as "slug!",
                    display_name as "display_name!",
                    state as "state!: Sqlx<ExchangeState>",
                    submissions_start as "submissions_start!: UtcDateTime",
                    submissions_end as "submissions_end!: UtcDateTime"
                FROM exchanges
                WHERE guild = $1
                    AND channel = $2
                    AND submissions_start < $4
                    AND submissions_end > $3
                "#,
                guild,
                channel,
                start,
                end,
            )
            .fetch_all(&mut transaction)
            .await?
        };

        transaction.commit().await?;

        Ok(overlapping_exchanges)
    }

    pub async fn get_upcoming_exchanges(
        &self,
        guild: GuildId,
        after: UtcDateTime,
    ) -> Result<Vec<Exchange>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let upcoming_exchanges = {
            let guild = guild.to_sqlx();
            query_as!(
                Exchange,
                r#"
                SELECT
                    id,
                    guild as "guild!: Sqlx<GuildId>",
                    channel as "channel!: Sqlx<ChannelId>",
                    jam_type as "jam_type!: Sqlx<JamType>",
                    jam_link as "jam_link!",
                    slug as "slug!",
                    display_name as "display_name!",
                    state as "state!: Sqlx<ExchangeState>",
                    submissions_start as "submissions_start!: UtcDateTime",
                    submissions_end as "submissions_end!: UtcDateTime"
                FROM exchanges
                WHERE guild = $1 AND submissions_end > $2
                ORDER BY submissions_start, display_name
                "#,
                guild,
                after,
            )
            .fetch_all(&mut transaction)
            .await?
        };

        transaction.commit().await?;

        Ok(upcoming_exchanges)
    }

    pub async fn delete_exchange(&self, guild: GuildId, slug: &str) -> Result<bool, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let guild = guild.to_sqlx();
        let query_result = query!(
            r#"DELETE FROM exchanges WHERE guild = $1 AND slug = $2"#,
            guild,
            slug,
        )
        .execute(&mut transaction)
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
