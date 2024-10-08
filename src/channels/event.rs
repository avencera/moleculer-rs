use crate::{
    broker::ServiceBroker,
    channels::messages::incoming::EventMessage,
    config::{self, Channel, Config},
    nats::Conn,
};

use act_zero::*;
use async_nats::Message;
use async_trait::async_trait;
use config::DeserializeError;
use futures::StreamExt;
use log::{debug, error, info};
use std::sync::Arc;

#[async_trait]
impl Actor for Event {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        let pid_clone = pid.clone();
        send!(pid_clone.listen(pid));
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("Event Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
pub(crate) struct Event {
    config: Arc<Config>,
    broker: WeakAddr<ServiceBroker>,
    conn: Conn,
}

impl Event {
    pub(crate) async fn new(
        broker: WeakAddr<ServiceBroker>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            broker,
            conn: conn.clone(),
            config: Arc::clone(config),
        }
    }

    pub(crate) async fn listen(&mut self, pid: Addr<Self>) {
        info!("Listening for EVENT messages");
        let mut channel = self
            .conn
            .subscribe(&Channel::Event.channel_to_string(&self.config))
            .await
            .unwrap();

        pid.clone().send_fut(async move {
            while let Some(msg) = channel.next().await {
                match call!(pid.handle_message(msg)).await {
                    Ok(_) => debug!("Successfully handled EVENT message"),
                    Err(e) => error!("Unable to handle EVENT message: {}", e),
                }
            }
        })
    }

    async fn handle_message(&self, msg: Message) -> ActorResult<()> {
        let event_context: Result<EventMessage, DeserializeError> =
            self.config.serializer.deserialize(&msg.payload);

        send!(self.broker.handle_incoming_event(event_context));

        Produces::ok(())
    }
}
