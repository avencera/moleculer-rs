mod disconnect;
mod discover;
mod heartbeat;
mod info;
mod ping;
mod pong;

use std::collections::HashMap;
use std::sync::Arc;

use act_zero::runtimes::tokio::spawn_actor;
use act_zero::*;
use async_trait::async_trait;
use log::error;
use thiserror::Error;

use crate::{
    config,
    config::{Channel, Config, Transporter},
    nats,
};

use self::{
    disconnect::{Disconnect, DisconnectMessage},
    discover::Discover,
    heartbeat::Heartbeat,
    ping::{Ping, PingTargeted},
    pong::Pong,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to start listeners actor")]
    UnableToStartListeners,

    #[error(transparent)]
    NatsError(#[from] nats::Error),

    #[error(transparent)]
    DeserializeError(#[from] config::DeserializeError),

    #[error(transparent)]
    SerializeError(#[from] config::SerializeError),
}

#[async_trait]
impl Actor for ChannelSupervisor {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        self.pid = pid.downgrade();
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("ChannelSupervisor Actor Error: {:?}", error);
        // do not stop on actor error
        false
    }
}

pub struct ChannelSupervisor {
    conn: nats::Conn,
    config: Arc<Config>,
    pid: WeakAddr<Self>,
    channels: HashMap<Channel, String>,

    // channels
    event: Addr<Event>,

    request: Addr<Request>,
    response: Addr<Response>,

    discover: Addr<Discover>,
    discover_targeted: Addr<DiscoverTargeted>,

    info: Addr<Info>,
    info_targeted: Addr<InfoTargeted>,

    heartbeat: Addr<Heartbeat>,

    ping: Addr<Ping>,
    ping_targeted: Addr<PingTargeted>,

    pong: Addr<Pong>,
    disconnect: Addr<Disconnect>,
}

impl ChannelSupervisor {
    async fn new(config: Arc<Config>) -> Self {
        let channels = Channel::build_hashmap(&config);

        let conn = match &config.transporter {
            Transporter::Nats(nats_address) => nats::Conn::new(nats_address)
                .await
                .expect("NATS should connect"),
        };

        Self {
            conn,
            config,
            channels,

            pid: WeakAddr::detached(),
            event: Addr::detached(),

            request: Addr::detached(),
            response: Addr::detached(),

            discover: Addr::detached(),
            discover_targeted: Addr::detached(),

            info: Addr::detached(),
            info_targeted: Addr::detached(),

            heartbeat: Addr::detached(),

            ping: Addr::detached(),
            ping_targeted: Addr::detached(),

            pong: Addr::detached(),
            disconnect: Addr::detached(),
        }
    }

    async fn start_listeners(&mut self) -> ActorResult<()> {
        self.heartbeat =
            spawn_actor(Heartbeat::new(self.pid.clone(), &self.config, &self.conn).await);
        send!(self.heartbeat.listen());

        self.ping = spawn_actor(Ping::new(self.pid.clone(), &self.config, &self.conn).await);
        send!(self.ping.listen());

        self.ping_targeted =
            spawn_actor(PingTargeted::new(self.pid.clone(), &self.config, &self.conn).await);
        send!(self.ping_targeted.listen());

        self.pong = spawn_actor(Pong::new(self.pid.clone(), &self.config, &self.conn).await);
        send!(self.pong.listen());

        self.disconnect =
            spawn_actor(Disconnect::new(self.pid.clone(), &self.config, &self.conn).await);
        send!(self.disconnect.listen());

        Produces::ok(())
    }

    async fn publish_to_channel<T>(&self, channel: T, message: Vec<u8>) -> ActorResult<()>
    where
        T: AsRef<str>,
    {
        let res = self.conn.send(channel.as_ref(), message).await;

        if let Err(err) = res {
            error!("Unable to send heartbeat: {}", err)
        }

        Produces::ok(())
    }

    async fn publish(&self, channel: Channel, message: Vec<u8>) -> ActorResult<()> {
        let channel = self
            .channels
            .get(&channel)
            .expect("should always find channel");

        let _ = self.publish_to_channel(channel.as_str(), message);

        Produces::ok(())
    }

    async fn send_disconnect(&self) -> ActorResult<()> {
        let msg = DisconnectMessage::new(&self.config.node_id);

        let _ = self.publish(Channel::Disconnect, self.config.serialize(msg)?);

        Produces::ok(())
    }
}

impl Actor for Event {}
struct Event {
    parent: WeakAddr<ChannelSupervisor>,
}

impl Event {
    fn new(parent: WeakAddr<ChannelSupervisor>) -> Self {
        Self { parent }
    }
}

impl Actor for Request {}
struct Request {
    parent: WeakAddr<ChannelSupervisor>,
}

impl Request {
    fn new(parent: WeakAddr<ChannelSupervisor>) -> Self {
        Self { parent }
    }
}

impl Actor for Response {}
struct Response {
    parent: WeakAddr<ChannelSupervisor>,
}

impl Response {
    fn new(parent: WeakAddr<ChannelSupervisor>) -> Self {
        Self { parent }
    }
}

impl Actor for DiscoverTargeted {}
struct DiscoverTargeted {
    parent: WeakAddr<ChannelSupervisor>,
}

impl DiscoverTargeted {
    fn new(parent: WeakAddr<ChannelSupervisor>) -> Self {
        Self { parent }
    }
}

impl Actor for Info {}
struct Info {
    parent: WeakAddr<ChannelSupervisor>,
}

impl Info {
    fn new(parent: WeakAddr<ChannelSupervisor>) -> Self {
        Self { parent }
    }
}

impl Actor for InfoTargeted {}
struct InfoTargeted {
    parent: WeakAddr<ChannelSupervisor>,
}

impl InfoTargeted {
    fn new(parent: WeakAddr<ChannelSupervisor>) -> Self {
        Self { parent }
    }
}

pub async fn subscribe_to_channels(config: Arc<Config>) -> Result<(), Error> {
    let registry = spawn_actor(ChannelSupervisor::new(config).await);
    let registry_clone = registry.clone();

    call!(registry.start_listeners())
        .await
        .map_err(|_| Error::UnableToStartListeners)?;

    // detects SIGTERM and sends disconnect package
    let _ = ctrlc::set_handler(move || send!(registry_clone.send_disconnect()));

    registry.termination().await;

    Ok(())
}
