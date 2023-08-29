use poise::serenity_prelude::{ChannelId, GuildId};
use sqlx::{query_as, Pool, Sqlite};
use time::{Duration, OffsetDateTime};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::{
    data::{
        types::{Sqlx, SqlxConvertible, UtcDateTime},
        Exchange, ExchangeRound, ExchangeRoundState,
    },
    jam_types::JamType,
};

#[derive(Debug)]
pub struct ExchangeStorage {
    pool: Pool<Sqlite>,
    events: Sender<ExchangeStorageEvent>,
}

pub struct CreateExchange {
    pub guild_id: GuildId,
    pub jam_type: JamType,
    pub jam_link: String,
    pub slug: String,
    pub display_name: String,
    pub submission_channel: ChannelId,
    pub num_rounds: u8,
    pub first_round_start: OffsetDateTime,
    pub submission_duration: Duration,
    pub round_duration: Duration,
    pub games_per_member: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum ExchangeStorageEvent {
    ExchangesUpdated,
}

impl ExchangeStorage {
    pub fn new(pool: Pool<Sqlite>) -> ExchangeStorage {
        ExchangeStorage {
            pool,
            events: tokio::sync::broadcast::channel(128).0,
        }
    }

    pub async fn create_exchange(
        &self,
        create_exchange: CreateExchange,
    ) -> Result<(Exchange, Vec<ExchangeRound>), anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let exchange = {
            let guild_id = create_exchange.guild_id.to_sqlx();
            let jam_type = create_exchange.jam_type.to_sqlx();
            let submission_channel = create_exchange.submission_channel.to_sqlx();

            query_as!(
                Exchange,
                r#"--sql
                INSERT INTO exchanges (
                    guild,
                    jam_type,
                    jam_link,
                    slug,
                    display_name,
                    submission_channel)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING
                    id,
                    guild as "guild!: Sqlx<GuildId>",
                    jam_type as "jam_type!: Sqlx<JamType>",
                    jam_link as "jam_link!",
                    slug as "slug!",
                    display_name as "display_name!",
                    submission_channel as "submission_channel!: Sqlx<ChannelId>"
                "#,
                guild_id,
                jam_type,
                create_exchange.jam_link,
                create_exchange.slug,
                create_exchange.display_name,
                submission_channel,
            )
            .fetch_one(&mut transaction)
            .await?
        };

        let rounds = {
            let mut vec = Vec::with_capacity(create_exchange.num_rounds as _);

            let mut round_start = create_exchange.first_round_start;
            for _ in 0..create_exchange.num_rounds {
                let round_start_utc = UtcDateTime::from(round_start);
                let submissions_end_utc =
                    UtcDateTime::from(round_start + create_exchange.submission_duration);
                let assignments_sent_at_utc =
                    UtcDateTime::from(round_start + create_exchange.round_duration);

                vec.push({
                    let round_start_utc = round_start_utc.to_sqlx();
                    let submissions_end_utc = submissions_end_utc.to_sqlx();
                    let assignments_sent_at_utc = assignments_sent_at_utc.to_sqlx();
                    let state = ExchangeRoundState::NotStartedYet.to_sqlx();

                    query_as!(
                        ExchangeRound,
                        r#"--sql
                        INSERT INTO exchange_rounds (
                            exchange_id,
                            submissions_start_at,
                            submissions_end_at,
                            assignments_sent_at,
                            games_per_member,
                            state)
                        VALUES ($1, $2, $3, $4, $5, $6)
                        RETURNING
                            id,
                            exchange_id as "exchange_id!",
                            submissions_start_at as "submissions_start_at!: UtcDateTime",
                            submissions_end_at as "submissions_end_at!: UtcDateTime",
                            assignments_sent_at as "assignments_sent_at!: UtcDateTime",
                            games_per_member as "games_per_member!: u32",
                            state as "state!: Sqlx<ExchangeRoundState>"
                        "#,
                        exchange.id,
                        round_start_utc,
                        submissions_end_utc,
                        assignments_sent_at_utc,
                        create_exchange.games_per_member,
                        state,
                    )
                    .fetch_one(&mut transaction)
                    .await?
                });

                round_start += create_exchange.round_duration;
            }

            vec
        };

        transaction.commit().await?;

        let _ = self.events.send(ExchangeStorageEvent::ExchangesUpdated); // Don't care if it actually gets received

        Ok((exchange, rounds))
    }

    pub fn subscribe(&self) -> Receiver<ExchangeStorageEvent> {
        self.events.subscribe()
    }
}
