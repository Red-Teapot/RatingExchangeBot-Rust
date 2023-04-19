use poise::async_trait;
use sqlx::{Pool, Postgres};
use tiny_tokio_actor::{Actor, ActorContext, Handler, Message};

use crate::RebotSystemEvent;

pub struct ExchangeStorage {
    _pool: Pool<Postgres>,
}

#[async_trait]
impl Actor<RebotSystemEvent> for ExchangeStorage {}

#[derive(Clone)]
pub struct CreateExchangeMessage {}

impl Message for CreateExchangeMessage {
    type Response = ();
}

#[async_trait]
impl Handler<RebotSystemEvent, CreateExchangeMessage> for ExchangeStorage {
    async fn handle(
        &mut self,
        _msg: CreateExchangeMessage,
        _ctx: &mut ActorContext<RebotSystemEvent>,
    ) {
    }
}
