use std::{error::Error, sync::Arc, thread};

use indoc::formatdoc;
use serenity::http::Http;
use time::{Duration, OffsetDateTime};
use tokio::{runtime::Handle, select};
use tracing::{error, info, info_span, warn, Instrument};

use crate::{
    models::types::UtcDateTime,
    repository::{
        ExchangeRepository, ExchangeStorageEvent, PlayedGameRepository, SubmissionRepository,
    },
    utils::formatting::{format_local, format_utc},
};

pub struct AssignmentService {
    http: Arc<Http>,
    exchange_repository: Arc<ExchangeRepository>,
    submission_repository: Arc<SubmissionRepository>,
    played_game_repository: Arc<PlayedGameRepository>,
}

const DEFAULT_SLEEP_DURATION: Duration = Duration::seconds(60 * 60 /* One hour */);
const EXCHANGE_START_THRESHOLD: Duration = Duration::seconds(60 * 60 /* One hour */);
const EXCHANGE_END_THRESHOLD: Duration = Duration::seconds(60 * 60 /* One hour */);

impl AssignmentService {
    pub fn create_and_start(
        http: Arc<Http>,
        exchange_repository: Arc<ExchangeRepository>,
        submission_repository: Arc<SubmissionRepository>,
        played_game_repository: Arc<PlayedGameRepository>,
    ) {
        let service = AssignmentService {
            http,
            exchange_repository,
            submission_repository,
            played_game_repository,
        };

        service.start();
    }

    fn start(mut self) {
        // Make sure it's on a separate thread due to possible heavy computations.
        let rt_handle = Handle::current();
        thread::spawn(move || {
            rt_handle.block_on(async move {
                let mut next_assignments_time = Some(OffsetDateTime::now_utc());

                let mut exchange_events = self.exchange_repository.subscribe();

                loop {
                    let sleep_duration = {
                        let duration = next_assignments_time
                        .map(|time| {
                            Duration::max(Duration::ZERO, time - OffsetDateTime::now_utc())
                        })
                        .unwrap_or(DEFAULT_SLEEP_DURATION);

                        std::time::Duration::from_millis(duration.whole_milliseconds() as _)
                    };

                    info!(
                        "Next assignments invocation scheduled at {:?} (in {:?})",
                        OffsetDateTime::now_utc() + sleep_duration,
                        sleep_duration
                    );

                    select! {
                        _ = tokio::time::sleep(sleep_duration) => {
                            if let Err(err) = self.announce_exchange_submissions_open().await {
                                error!("Could not announce exchange submissions open: {err}");
                            }

                            if let Err(err) = self.perform_assignments().await {
                                error!("Could not perform assignments: {err}");
                            }

                            next_assignments_time = match self.reschedule().await {
                                Ok(time) => time,
                                Err(err) => {
                                    error!("Could not reschedule after performing assignments: {err}");
                                    None
                                }
                            };
                        }

                        evt = exchange_events.recv() => {
                            match evt {
                                Ok(ExchangeStorageEvent::ExchangesUpdated) => {
                                    next_assignments_time = match self.reschedule().await {
                                        Ok(time) => time,
                                        Err(err) => {
                                            error!("Could not reschedule after exchanges updated event: {err}");
                                            None
                                        }
                                    };
                                },
                                Err(err) => error!("Error while receiving an exchange event: {err:?}"),
                            }
                        }
                    }
                }
            }.instrument(info_span!("main_loop")));
        });
    }

    #[tracing::instrument(skip(self))]
    async fn announce_exchange_submissions_open(&self) -> Result<(), Box<dyn Error>> {
        info!("Announcing exchange submissions opening");

        let now = OffsetDateTime::now_utc();
        let starting_exchanges = self
            .exchange_repository
            .get_starting_exchanges(UtcDateTime::from(now))
            .await?;

        for exchange in starting_exchanges {
            let late_period = now - OffsetDateTime::from(exchange.submissions_start);

            if late_period > EXCHANGE_START_THRESHOLD {
                info!(
                    "An exchange has been missed by the bot by {}: {}",
                    late_period, exchange.slug
                );
                if let Err(err) = self
                    .exchange_repository
                    .update_exchange_state(exchange.id, crate::models::ExchangeState::MissedByBot)
                    .await
                {
                    warn!(
                        "Could not set exchange {:?} state to MissedByBot: {}",
                        exchange.id, err
                    );
                }
            } else {
                {
                    let message = formatdoc! {
                        r#"
                            # Review exchange {name} starts now!

                            **Submit your jam entry using the `/submit {slug} <entry link>` command.**

                            The exchange ends on {end_local} your time or {end_utc} UTC. You should submit your entry before this deadline.

                            After the deadline, you will receive a list of entries to play and rate in your DMs.
                        "#,
                        name = exchange.display_name,
                        slug = exchange.slug,
                        end_local = format_local(exchange.submissions_end),
                        end_utc = format_utc(exchange.submissions_end),
                    };
                    exchange.channel.say(&self.http, message).await?;
                };

                if let Err(err) = self
                    .exchange_repository
                    .update_exchange_state(
                        exchange.id,
                        crate::models::ExchangeState::AcceptingSubmissions,
                    )
                    .await
                {
                    warn!(
                        "Could not set exchange {:?} state to AcceptingSubmissions: {}",
                        exchange.id, err
                    );
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn perform_assignments(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Performing assignments");

        let now = OffsetDateTime::now_utc();
        let ending_exchanges = self
            .exchange_repository
            .get_ending_exchanges(UtcDateTime::from(now))
            .await?;

        for exchange in ending_exchanges {
            let late_period = now - OffsetDateTime::from(exchange.submissions_end);

            if late_period > EXCHANGE_END_THRESHOLD {
                info!(
                    "An exchange has been missed by the bot by {}: {}",
                    late_period, exchange.slug
                );
                if let Err(err) = self
                    .exchange_repository
                    .update_exchange_state(exchange.id, crate::models::ExchangeState::MissedByBot)
                    .await
                {
                    warn!(
                        "Could not set exchange {:?} state to MissedByBot: {}",
                        exchange.id, err
                    );
                }
            } else {
                // TODO: Run the solver etc.

                {
                    let message = formatdoc! {
                        r#"
                            # Review exchange {name} has just ended!

                            **You should have received your assignments to play and rate in the DMs.**

                            If that didn't happen, please contact the moderators.
                        "#,
                        name = exchange.display_name,
                    };
                    exchange.channel.say(&self.http, message).await?;
                };

                if let Err(err) = self
                    .exchange_repository
                    .update_exchange_state(
                        exchange.id,
                        crate::models::ExchangeState::AssignmentsSent,
                    )
                    .await
                {
                    warn!(
                        "Could not set exchange {:?} state to AcceptingSubmissions: {}",
                        exchange.id, err
                    );
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn reschedule(&self) -> Result<Option<OffsetDateTime>, Box<dyn Error>> {
        info!("Rescheduling");

        match self
            .exchange_repository
            .get_closest_exchange_end_or_start_date()
            .await
        {
            Ok(Some(date)) => Ok(Some(date.into())),
            Ok(None) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}
