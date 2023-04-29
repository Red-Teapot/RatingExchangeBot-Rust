use async_trait::async_trait;
use log::warn;
use time::OffsetDateTime;
use tokio::{select, sync::mpsc};

use crate::actors::{Actor, ActorHandle, MessageWrap};

pub struct Service {
    next_assignments_time: Option<OffsetDateTime>,
    message_receiver: mpsc::Receiver<MessageWrap<Service>>,
}

impl Service {
    pub fn new() -> (Service, ActorHandle<Service>) {
        let (sender, receiver) = mpsc::channel(128);

        let service = Service {
            next_assignments_time: None,
            message_receiver: receiver,
        };

        let handle = ActorHandle::new(sender);

        (service, handle)
    }

    pub fn start(mut self) {
        tokio::spawn(async move {
            loop {
                let sleep_duration = self.next_assignments_time.map(|time| {
                    let duration_raw = time - OffsetDateTime::now_utc();
                    std::time::Duration::from_millis(
                        duration_raw.whole_milliseconds().try_into().unwrap(),
                    )
                });

                select! {
                    _ = tokio::time::sleep(sleep_duration.unwrap()), if sleep_duration.is_some() => {
                        self.perform_assignments().await;
                        self.reschedule().await;
                    }

                    Some(MessageWrap { message, respond_to }) = self.message_receiver.recv() => {
                        #[allow(clippy::unit_arg)]
                        match respond_to.send(self.handle_message(&message).await) {
                            Ok(_) => (),
                            Err(response) => warn!("Could not respond to a message. Message: {:?}, response: {:?}", message, response),
                        }
                    }
                }
            }
        });
    }

    async fn perform_assignments(&mut self) {}

    async fn reschedule(&mut self) {}
}

#[derive(Debug)]
pub enum Message {
    Reschedule,
    PerformAssignments,
}

#[async_trait]
impl Actor for Service {
    type Message = Message;
    type Response = ();

    async fn handle_message(&mut self, message: &Self::Message) -> Self::Response {
        match message {
            Message::Reschedule => self.reschedule().await,
            Message::PerformAssignments => self.perform_assignments().await,
        }
    }
}
