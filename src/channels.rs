pub mod messages;

mod event;

mod disconnect;
mod discover;
mod heartbeat;
mod info;
mod ping;
mod pong;
mod request;
mod response;

use std::collections::HashMap;
use std::sync::Arc;

use act_zero::runtimes::tokio::spawn_actor;
use act_zero::*;
use async_trait::async_trait;
use log::{debug, error};
use thiserror::Error;

use crate::{
    broker::ServiceBroker,
    config,
    config::{Channel, Config, Transporter},
    nats,
};

use self::{
    disconnect::Disconnect,
    discover::{Discover, DiscoverTargeted},
    event::Event,
    heartbeat::Heartbeat,
    info::{Info, InfoTargeted},
    messages::outgoing::DisconnectMessage,
    ping::{Ping, PingTargeted},
    pong::Pong,
    request::Request,
    response::Response,
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
    broker: Addr<ServiceBroker>,

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
    async fn new(broker: Addr<ServiceBroker>, config: Arc<Config>) -> Self {
        let channels = Channel::build_hashmap(&config);

        let conn = match &config.transporter {
            Transporter::Nats(nats_address) => nats::Conn::new(nats_address)
                .await
                .expect("NATS should connect"),
        };

        Self {
            broker,
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
        let broker_pid = self.broker.clone().downgrade();

        self.heartbeat = spawn_actor(
            Heartbeat::new(
                self.pid.clone(),
                broker_pid.clone(),
                &self.config,
                &self.conn,
            )
            .await,
        );

        self.ping = spawn_actor(Ping::new(self.pid.clone(), &self.config, &self.conn).await);

        self.ping_targeted =
            spawn_actor(PingTargeted::new(self.pid.clone(), &self.config, &self.conn).await);

        self.pong = spawn_actor(Pong::new(self.pid.clone(), &self.config, &self.conn).await);

        self.disconnect =
            spawn_actor(Disconnect::new(broker_pid.clone(), &self.config, &self.conn).await);

        self.discover = spawn_actor(
            Discover::new(
                broker_pid.clone(),
                self.pid.clone(),
                &self.config,
                &self.conn,
            )
            .await,
        );

        self.discover_targeted =
            spawn_actor(DiscoverTargeted::new(broker_pid.clone(), &self.config, &self.conn).await);

        self.info = spawn_actor(Info::new(broker_pid.clone(), &self.config, &self.conn).await);

        self.info_targeted =
            spawn_actor(InfoTargeted::new(broker_pid.clone(), &self.config, &self.conn).await);

        self.event = spawn_actor(Event::new(broker_pid.clone(), &self.config, &self.conn).await);

        self.request =
            spawn_actor(Request::new(broker_pid.clone(), &self.config, &self.conn).await);

        self.response = spawn_actor(Response::new(broker_pid, &self.config, &self.conn).await);

        Produces::ok(())
    }

    pub async fn broadcast_discover(&self) {
        send!(self.discover.broadcast());
    }

    pub async fn publish_to_channel<T>(&self, channel: T, message: Vec<u8>) -> ActorResult<()>
    where
        T: AsRef<str>,
    {
        let res = self.conn.send(channel.as_ref(), message).await;

        if let Err(err) = res {
            error!("Unable to send message: {}", err)
        }

        Produces::ok(())
    }

    async fn publish(&self, channel: Channel, message: Vec<u8>) -> ActorResult<()> {
        let channel = self
            .channels
            .get(&channel)
            .expect("should always find channel");

        let _ = self.publish_to_channel(channel.as_str(), message).await;

        debug!("Message published to channel: {}", channel);

        Produces::ok(())
    }

    async fn send_disconnect(&self) -> ActorResult<()> {
        let msg = DisconnectMessage::new(&self.config.node_id);

        let _ = self
            .publish(Channel::Disconnect, self.config.serializer.serialize(msg)?)
            .await;

        debug!("Disconnect message sent");
        Produces::ok(())
    }
}

pub async fn start_supervisor(
    broker: Addr<ServiceBroker>,
    config: Arc<Config>,
) -> Result<Addr<ChannelSupervisor>, Error> {
    let channel_supervisor = spawn_actor(ChannelSupervisor::new(broker, config).await);

    call!(channel_supervisor.start_listeners())
        .await
        .map_err(|_| Error::UnableToStartListeners)?;

    Ok(channel_supervisor)
}

pub async fn listen_for_disconnect(supervisor: Addr<ChannelSupervisor>) {
    // detects SIGTERM and sends disconnect package
    let _ = ctrlc::set_handler(move || {
        send!(supervisor.send_disconnect());
        println!("Exiting molecular....");
        std::thread::sleep(std::time::Duration::from_millis(100));
        std::process::exit(1);
    });
}
