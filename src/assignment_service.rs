use std::{error::Error, sync::Arc, thread};

use indoc::formatdoc;
use poise::serenity_prelude::UserId;
use serenity::http::Http;
use time::{Duration, OffsetDateTime};
use tokio::{runtime::Handle, select, sync::Notify};
use tracing::{debug, error, info, info_span, warn, Instrument};

use crate::{
    models::{types::UtcDateTime, Exchange, ExchangeState, Submission},
    repository::{
        ExchangeRepository, ExchangeStorageEvent, PlayedGameRepository, SubmissionRepository,
    },
    solver::dinic,
    utils::{
        assignment_network::AssignmentNetwork,
        formatting::{format_local, format_utc},
    },
};

pub struct AssignmentService {
    shutdown: Arc<Notify>,
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
        shutdown: Arc<Notify>,
        http: Arc<Http>,
        exchange_repository: Arc<ExchangeRepository>,
        submission_repository: Arc<SubmissionRepository>,
        played_game_repository: Arc<PlayedGameRepository>,
    ) {
        let service = AssignmentService {
            shutdown,
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
                let shutdown_notify = self.shutdown.clone();

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
                        _ = shutdown_notify.notified() => {
                            break
                        }

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
                    .update_exchange_state(exchange.id, ExchangeState::MissedByBot)
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

                            **Submit your jam entry using the `/submit <entry link>` command.**

                            The exchange ends on {end_local} your time or {end_utc} UTC. You should submit your entry before this deadline.

                            After the deadline, you will receive a list of entries to play and rate in your DMs.
                        "#,
                        name = exchange.display_name,
                        end_local = format_local(exchange.submissions_end),
                        end_utc = format_utc(exchange.submissions_end),
                    };
                    exchange.channel.say(&self.http, message).await?;
                };

                if let Err(err) = self
                    .exchange_repository
                    .update_exchange_state(exchange.id, ExchangeState::AcceptingSubmissions)
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
                    .update_exchange_state(exchange.id, ExchangeState::MissedByBot)
                    .await
                {
                    warn!(
                        "Could not set exchange {:?} state to MissedByBot: {}",
                        exchange.id, err
                    );
                }
            } else {
                if let Err(err) = self.perform_assignments_for_exchange(&exchange).await {
                    error!("Could not perform assignments for exchange {exchange:?}: {err}");
                    if let Err(err) = self
                        .exchange_repository
                        .update_exchange_state(exchange.id, ExchangeState::AssignmentError)
                        .await
                    {
                        warn!(
                            "Could not set exchange {:?} state to AssignmentError: {}",
                            exchange.id, err
                        );
                    }
                    continue;
                }

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
                    .update_exchange_state(exchange.id, ExchangeState::AssignmentsSent)
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

    async fn perform_assignments_for_exchange(
        &self,
        exchange: &Exchange,
    ) -> Result<(), Box<dyn Error>> {
        let submissions = self
            .submission_repository
            .get_submissions_for_exchange(exchange.id)
            .await?;
        let played_games = self
            .played_game_repository
            .get_played_games_for_exchange(exchange.id)
            .await?;

        let mut network = AssignmentNetwork::build(exchange, submissions, &played_games);

        dinic::solve(&mut network.network);

        debug!("Solved network: {network:?}");

        let assignments = network.get_assignments();

        for (user, assignments) in assignments {
            if let Err(err) = self
                .send_user_assignments(exchange, user, &assignments)
                .await
            {
                warn!("Could not send assignments to user {user}: {err}");
            }
        }

        Ok(())
    }

    async fn send_user_assignments(
        &self,
        exchange: &Exchange,
        user: UserId,
        assignments: &[Submission],
    ) -> Result<(), Box<dyn Error>> {
        let message = if assignments.is_empty() {
            formatdoc! {
                r#"
                    # Could not assign you any entries for {exchange_name}

                    This probably means you have already played all entries for this exchange, or the algorithm could not find a solution.

                    No actions are needed on your side.
                "#,
                exchange_name = exchange.display_name,
            }
        } else {
            let assignments_str = assignments
                .iter()
                .map(|assignment| format!("- {}", assignment.link))
                .collect::<Vec<String>>()
                .join("\n");

            formatdoc! {
                r#"
                   # Here are your assignments

                   {assignments_str}

                   You are supposed to play and rate the assignments before the jam ends.

                   If you decide to rate some entries outside of the assignments, you can use the `/played <entry link>` command.
                   This will make sure these entries won't be assigned to you in the future.
                "#,
                assignments_str = assignments_str,
            }
        };

        let channel = user.create_dm_channel(&self.http).await?;

        channel.say(&self.http, message).await?;

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
