use std::{sync::Arc, thread, time::Duration};

use time::OffsetDateTime;
use tokio::{runtime::Handle, select};
use tracing::{error, info, info_span, Instrument};

use crate::storage::{ExchangeStorage, ExchangeStorageEvent};

pub struct AssignmentService {
    next_assignments_time: Option<OffsetDateTime>,
    exchange_storage: Arc<ExchangeStorage>,
}

impl AssignmentService {
    pub fn create_and_start(exchange_storage: Arc<ExchangeStorage>) {
        let service = AssignmentService {
            next_assignments_time: None,
            exchange_storage,
        };

        service.start();
    }

    fn start(mut self) {
        // Make sure it's on a separate thread due to possible heavy computations.
        let rt_handle = Handle::current();
        thread::spawn(move || {
            rt_handle.block_on(async move {
                self.reschedule().await;

                let mut exchange_events = self.exchange_storage.subscribe();

                loop {
                    let sleep_duration = self
                        .next_assignments_time
                        .map(|time| {
                            let duration_raw = time - OffsetDateTime::now_utc();
                            std::time::Duration::from_millis(
                                duration_raw.whole_milliseconds().try_into().unwrap(),
                            )
                        })
                        .unwrap_or(Duration::from_secs(60 * 60));

                    select! {
                        _ = tokio::time::sleep(sleep_duration) => {
                            self.perform_assignments().await;
                            self.reschedule().await;
                        }

                        evt = exchange_events.recv() => {
                            match evt {
                                Ok(ExchangeStorageEvent::ExchangesUpdated) => self.reschedule().await,
                                Err(err) => error!("Error while receiving an exchange event: {err:?}"),
                            }
                        }
                    }
                }
            }.instrument(info_span!("main_loop")));
        });
    }

    #[tracing::instrument(skip(self))]
    async fn perform_assignments(&mut self) {
        info!("Performing assignments");
    }

    #[tracing::instrument(skip(self))]
    async fn reschedule(&mut self) {
        info!("Rescheduling");
    }
}
