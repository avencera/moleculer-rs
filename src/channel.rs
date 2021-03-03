mod name;
mod nats;

use std::time::Duration;

use act_zero::runtimes::tokio::{spawn_actor, Timer};
use act_zero::timer::Tick;
use act_zero::*;
use async_trait::async_trait;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use sysinfo::{ProcessExt, ProcessorExt, RefreshKind, System, SystemExt};
use thiserror::Error;

use crate::config::{Config, Transporter};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to start listeners actor")]
    UnableToStartListeners,

    #[error(transparent)]
    NatsError(nats::Error),

    #[error("unknown channel error")]
    Unknown,
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
    config: Config,
    pid: WeakAddr<Self>,

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
    async fn new(config: Config) -> Self {
        let conn = match &config.transporter {
            Transporter::Nats(nats_address) => nats::Conn::new(nats_address)
                .await
                .expect("NATS should connect"),
        };

        Self {
            conn: conn,
            config: config,
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
        self.heartbeat =
            spawn_actor(Heartbeat::new(self.pid.clone(), self.conn.clone(), &self.config).await);

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
    conn: nats::Conn,
    timer: Timer,
    parent: Addr<Registry>,
    channel: String,
    heartbeat_interval: u32,
    system: sysinfo::System,
    node_id: String,
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
            let msg = HeartbeatMessage {
                ver: "4",
                sender: &self.node_id,
                cpu: self.system.get_global_processor_info().get_cpu_usage(),
            };

            let res = self
                .conn
                .send(&self.channel, serde_json::to_vec(&msg)?)
                .await;

            if let Err(err) = res {
                error!("Unable to send heartbeat: {}", err)
            }
        }
        Produces::ok(())
    }
}

impl Heartbeat {
    async fn new(parent: WeakAddr<Registry>, conn: nats::Conn, config: &Config) -> Self {
        Self {
            conn: conn,
            parent: parent.upgrade(),
            channel: name::heartbeat(config),
            heartbeat_interval: config.heartbeat_interval,
            timer: Timer::default(),
            system: System::new_with_specifics(RefreshKind::new().with_cpu()),
            node_id: config.node_id.clone(),
        }
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
    parent: WeakAddr<Registry>,
}

impl Ping {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
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

pub async fn subscribe_to_channels(config: Config) -> Result<(), Error> {
    let registry = spawn_actor(Registry::new(config).await);
    call!(registry.start_listeners())
        .await
        .map_err(|_| Error::UnableToStartListeners)?;

    registry.termination().await;

    Ok(())
}
