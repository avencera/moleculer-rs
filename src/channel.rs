mod nats;

use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, time::SystemTime};

use act_zero::runtimes::tokio::{spawn_actor, Timer};
use act_zero::timer::Tick;
use act_zero::*;
use async_nats::{Message, Subscription};
use async_trait::async_trait;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessorExt, RefreshKind, System, SystemExt};
use thiserror::Error;

use crate::config::{self, Channel, Config, Transporter};

use self::nats::Conn;

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
impl Actor for Registry {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        self.pid = pid.downgrade();
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("Registry Actor Error: {:?}", error);
        // do not stop on actor error
        false
    }
}

struct Registry {
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

impl Registry {
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
        self.event = spawn_actor(Event::new(self.pid.clone()));
        self.heartbeat = spawn_actor(Heartbeat::new(self.pid.clone(), &self.config).await);

        self.ping = spawn_actor(Ping::new(self.pid.clone(), &self.config, &self.conn).await);
        send!(self.ping.listen());

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
        let msg = DisconnectMessage {
            ver: "4",
            sender: &self.config.node_id,
        };

        let _ = self.publish(Channel::Disconnect, self.config.serialize(msg)?);

        Produces::ok(())
    }
}

impl Actor for Event {}
struct Event {
    parent: WeakAddr<Registry>,
}

impl Event {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
    }
}

impl Actor for Request {}
struct Request {
    parent: WeakAddr<Registry>,
}

impl Request {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
    }
}

impl Actor for Response {}
struct Response {
    parent: WeakAddr<Registry>,
}

impl Response {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
    }
}

impl Actor for Discover {}
struct Discover {
    parent: WeakAddr<Registry>,
}

impl Discover {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
    }
}

impl Actor for DiscoverTargeted {}
struct DiscoverTargeted {
    parent: WeakAddr<Registry>,
}

impl DiscoverTargeted {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
    }
}

impl Actor for Info {}
struct Info {
    parent: WeakAddr<Registry>,
}

impl Info {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
    }
}

impl Actor for InfoTargeted {}
struct InfoTargeted {
    parent: WeakAddr<Registry>,
}

impl InfoTargeted {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
    }
}

struct Heartbeat {
    config: Arc<Config>,
    timer: Timer,
    parent: Addr<Registry>,
    heartbeat_interval: u32,
    system: sysinfo::System,
}

#[async_trait]
impl Actor for Heartbeat {
    async fn started(&mut self, addr: Addr<Self>) -> ActorResult<()> {
        // Start the timer
        self.timer
            .set_timeout_for_strong(addr, Duration::from_secs(self.heartbeat_interval as u64));

        Produces::ok(())
    }
}

#[async_trait]
impl Tick for Heartbeat {
    async fn tick(&mut self) -> ActorResult<()> {
        self.system.refresh_cpu();

        if self.timer.tick() {
            let _ = self.send_heartbeat().await;
        }
        Produces::ok(())
    }
}

impl Heartbeat {
    async fn new(parent: WeakAddr<Registry>, config: &Arc<Config>) -> Self {
        Self {
            config: Arc::clone(config),
            parent: parent.upgrade(),
            heartbeat_interval: config.heartbeat_interval,
            timer: Timer::default(),
            system: System::new_with_specifics(RefreshKind::new().with_cpu()),
        }
    }

    async fn send_heartbeat(&self) -> ActorResult<()> {
        let msg = HeartbeatMessage {
            ver: "4",
            sender: &self.config.node_id,
            cpu: self.system.get_global_processor_info().get_cpu_usage(),
        };

        send!(self
            .parent
            .publish(Channel::Heartbeat, self.config.serialize(msg)?));

        Produces::ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct HeartbeatMessage<'a> {
    ver: &'static str,
    sender: &'a str,
    cpu: f32,
}

impl Actor for Ping {}
struct Ping {
    config: Arc<Config>,
    channel: Subscription,
    parent: WeakAddr<Registry>,
}

#[derive(Deserialize)]
struct PingMessage {
    ver: String,
    sender: String,
    id: String,
    time: i64,
}

#[derive(Serialize)]
struct PongMessage<'a> {
    ver: String,
    sender: &'a str,
    id: String,
    time: i64,
    arrived: i64,
}

impl<'a> From<(PingMessage, &'a str)> for PongMessage<'a> {
    fn from(from: (PingMessage, &'a str)) -> Self {
        let (ping, node_id) = from;

        Self {
            ver: ping.ver,
            id: ping.id,
            sender: node_id,
            time: ping.time,
            arrived: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("now should always be before unix epoch")
                .as_millis() as i64,
        }
    }
}

impl Ping {
    pub async fn new(parent: WeakAddr<Registry>, config: &Arc<Config>, conn: &Conn) -> Self {
        Self {
            parent,
            channel: conn
                .subscribe(&Channel::Ping.channel_to_string(&config))
                .await
                .unwrap(),
            config: Arc::clone(config),
        }
    }

    pub async fn listen(&mut self) {
        info!("Listening for Ping messages");

        while let Some(msg) = self.channel.next().await {
            match self.handle_message(msg).await {
                Ok(_) => debug!("Successfully handled PING message"),
                Err(e) => error!("Unable to handle PING message: {}", e),
            }
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Error> {
        let ping_message: PingMessage = self.config.deserialize(&msg.data)?;
        let channel = format!(
            "{}.{}",
            Channel::PongPrefix.channel_to_string(&self.config),
            &ping_message.sender
        );

        let pong_message: PongMessage = (ping_message, self.config.node_id.as_str()).into();

        send!(self
            .parent
            .publish_to_channel(channel, self.config.serialize(pong_message)?));

        Ok(())
    }
}

impl Actor for PingTargeted {}
struct PingTargeted {
    parent: WeakAddr<Registry>,
}

impl PingTargeted {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
    }
}

impl Actor for Pong {}
struct Pong {
    parent: WeakAddr<Registry>,
}

impl Pong {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
    }
}

impl Actor for Disconnect {}
struct Disconnect {
    parent: WeakAddr<Registry>,
}

impl Disconnect {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
    }
}

#[derive(Serialize, Deserialize)]
struct DisconnectMessage<'a> {
    ver: &'static str,
    sender: &'a str,
}

pub async fn subscribe_to_channels(config: Arc<Config>) -> Result<(), Error> {
    let registry = spawn_actor(Registry::new(config).await);
    let registry_clone = registry.clone();

    call!(registry.start_listeners())
        .await
        .map_err(|_| Error::UnableToStartListeners)?;

    // detects SIGTERM and sends disconnect package
    let _ = ctrlc::set_handler(move || send!(registry_clone.send_disconnect()));

    registry.termination().await;

    Ok(())
}
