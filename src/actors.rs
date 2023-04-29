use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};

#[async_trait]
pub trait Actor {
    type Message: Send;
    type Response: Send;

    async fn handle_message(&mut self, message: &Self::Message) -> Self::Response;
}

pub struct ActorHandle<T: Actor> {
    message_sender: mpsc::Sender<MessageWrap<T>>,
}

impl<T: Actor> ActorHandle<T> {
    pub fn new(message_sender: mpsc::Sender<MessageWrap<T>>) -> ActorHandle<T> {
        ActorHandle { message_sender }
    }
}

impl<T: Actor> ActorHandle<T> {
    pub async fn send(&mut self, message: T::Message) -> T::Response {
        let (response_sender, response_receiver) = oneshot::channel();

        let _ = self
            .message_sender
            .send(MessageWrap {
                message,
                respond_to: response_sender,
            })
            .await;

        response_receiver.await.expect("The actor has died")
    }
}

pub struct MessageWrap<T: Actor> {
    pub message: T::Message,
    pub respond_to: oneshot::Sender<T::Response>,
}
