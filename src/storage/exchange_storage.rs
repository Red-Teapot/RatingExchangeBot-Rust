use poise::serenity_prelude::{ChannelId, GuildId};
use sqlx::{query_as, Pool, Postgres};
use time::{Duration, OffsetDateTime};

use crate::{
    data::{
        types::{SqlxConvertible, UtcDateTime},
        Exchange, ExchangeRound, ExchangeRoundState,
    },
    jam_types::JamType,
};

pub struct ExchangeStorage {
    pool: Pool<Postgres>,
}

impl ExchangeStorage {
    pub fn new(pool: Pool<Postgres>) -> ExchangeStorage {
        ExchangeStorage { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_exchange(
        &self,
        guild_id: GuildId,
        jam_type: JamType,
        jam_link: String,
        slug: String,
        display_name: String,
        submission_channel: ChannelId,
        num_rounds: u8,
        first_round_start: OffsetDateTime,
        submission_duration: Duration,
        round_duration: Duration,
        games_per_member: u8,
    ) -> Result<(Exchange, Vec<ExchangeRound>), anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let exchange = query_as!(
            Exchange,
            r#"INSERT INTO exchanges (
                   guild,
                   jam_type,
                   jam_link,
                   slug,
                   display_name,
                   submission_channel)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING
                   id,
                   guild as "guild: _",
                   jam_type as "jam_type: _",
                   jam_link,
                   slug,
                   display_name,
                   submission_channel as "submission_channel: _""#,
            guild_id.to_sqlx(),
            jam_type as i32,
            jam_link,
            slug,
            display_name,
            submission_channel.to_sqlx(),
        )
        .fetch_one(&mut transaction)
        .await?;

        let rounds = {
            let mut vec = Vec::with_capacity(num_rounds as _);

            let mut round_start = first_round_start;
            for _ in 0..num_rounds {
                let round_start_utc = UtcDateTime::from(round_start);
                let submissions_end_utc = UtcDateTime::from(round_start + submission_duration);
                let assignments_sent_at_utc = UtcDateTime::from(round_start + round_duration);

                vec.push(
                    query_as!(
                        ExchangeRound,
                        r#"INSERT INTO exchange_rounds (
                           exchange_id,
                           submissions_start_at,
                           submissions_end_at,
                           assignments_sent_at,
                           games_per_member,
                           state)
                       VALUES ($1, $2, $3, $4, $5, $6)
                       RETURNING
                           id,
                           exchange_id,
                           submissions_start_at as "submissions_start_at: UtcDateTime",
                           submissions_end_at as "submissions_end_at: UtcDateTime",
                           assignments_sent_at as "assignments_sent_at: UtcDateTime",
                           games_per_member,
                           state as "state: ExchangeRoundState""#,
                        exchange.id,
                        round_start_utc.to_sqlx(),
                        submissions_end_utc.to_sqlx(),
                        assignments_sent_at_utc.to_sqlx(),
                        games_per_member as i32,
                        ExchangeRoundState::NotStartedYet as i32,
                    )
                    .fetch_one(&mut transaction)
                    .await?,
                );

                round_start += round_duration;
            }

            vec
        };

        transaction.commit().await?;

        Ok((exchange, rounds))
    }
}
