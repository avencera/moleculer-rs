use crate::{
    channel::messages::incoming::EventMessage,
    config::{self, Channel, Config},
    nats::Conn,
    service,
};

use super::ChannelSupervisor;
use act_zero::*;
use async_nats::Message;
use async_trait::async_trait;
use log::{debug, error, info};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to deserialize to EventContext: {0}")]
    DeserializeFail(#[from] config::DeserializeError),

    #[error("Unable to find event '{0}' in registry")]
    EventNotFound(String),

    #[error("Unable to find callback function for event '{0}'")]
    CallbackNotFound(String),

    #[error("Call back function failed to complete: {0}")]
    CallbackFailed(String),
}

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
pub struct Event {
    config: Arc<Config>,
    parent: WeakAddr<ChannelSupervisor>,
    events: HashMap<String, service::Event>,
    conn: Conn,
}

impl Event {
    pub async fn new(
        parent: WeakAddr<ChannelSupervisor>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        let events = config
            .services
            .iter()
            .flat_map(|service| service.events.clone())
            .collect();

        Self {
            parent,
            events,
            conn: conn.clone(),
            config: Arc::clone(config),
        }
    }

    pub async fn listen(&mut self, pid: Addr<Self>) {
        info!("Listening for EVENT messages");
        let channel = self
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
        let event_context: EventMessage = self.config.serializer.deserialize(&msg.data)?;

        let event = self
            .events
            .get(&event_context.event)
            .ok_or_else(|| Error::EventNotFound(event_context.event.clone()))?;

        let callback = event
            .callback
            .clone()
            .ok_or_else(|| Error::CallbackNotFound(event_context.event.clone()))?;

        callback(event_context).map_err(|err| Error::CallbackFailed(err.to_string()))?;

        Produces::ok(())
    }
}
