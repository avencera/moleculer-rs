mod util;

pub mod config;
pub mod service;

mod channel;
mod nats;

use std::{collections::HashMap, sync::Arc};

use act_zero::runtimes::tokio::spawn_actor;
use act_zero::*;
use async_trait::async_trait;
use channel::{messages::outgoing, ChannelSupervisor};
use config::{Channel, Serializer};
use service::{Event, Service};
use thiserror::Error;

pub(crate) mod built_info {
    #[allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ChannelError(#[from] channel::Error),
}

pub struct ServiceBroker {
    pub namespace: String,
    pub node_id: String,
    pub instance_id: String,
    pub services: Vec<Service>,
    pub events: HashMap<String, Event>,
    pub serializer: Serializer,

    pid: Addr<Self>,
    channel_supervisor: Addr<ChannelSupervisor>,
    config: Arc<config::Config>,
}

#[async_trait]
impl Actor for ServiceBroker {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        self.pid = pid.clone();

        let channel_supervisor = channel::start_supervisor(pid, Arc::clone(&self.config))
            .await
            .map_err(Error::ChannelError)?;

        send!(self.pid.broadcast_info());
        send!(channel_supervisor.broadcast_discover());

        self.channel_supervisor = channel_supervisor.clone();

        self.pid
            .send_fut(async move { channel::listen_for_disconnect(channel_supervisor).await });

        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        log::error!("ServiceBroker Actor Error: {:?}", error);
        // do not stop on actor error
        false
    }
}
impl ServiceBroker {
    pub fn new(config: config::Config, services: Vec<Service>) -> Self {
        let events = services
            .iter()
            .flat_map(|service| service.events.clone())
            .collect();

        Self {
            namespace: config.namespace.clone(),
            node_id: config.node_id.clone(),
            instance_id: config.instance_id.clone(),
            services,
            events,
            serializer: config.serializer.clone(),

            pid: Addr::detached(),
            channel_supervisor: Addr::detached(),
            config: Arc::new(config),
        }
    }

    pub async fn start(self) {
        let addr = spawn_actor(self);
        addr.termination().await;
    }

    pub async fn broadcast_info(&self) -> ActorResult<()> {
        self.send_info(Channel::Info.channel_to_string(&self.config))
            .await
    }

    pub(crate) async fn send_info(&self, channel: String) -> ActorResult<()> {
        let info = outgoing::InfoMessage::new(&self.config, &self.services);
        send!(self
            .channel_supervisor
            .publish_to_channel(channel, self.serializer.serialize(info)?));

        Produces::ok(())
    }
}
