mod name;
mod nats;

use act_zero::runtimes::tokio::spawn_actor;
use act_zero::*;
use async_trait::async_trait;
use log::error;
use thiserror::Error;

use crate::config::{Config, Transporter};

#[derive(Error, Debug)]
enum ChannelError {
    #[error("unknown channel error")]
    Unknown,

    #[error(transparent)]
    NatsError(nats::Error),
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

impl Actor for Heartbeat {}
struct Heartbeat {
    parent: WeakAddr<Registry>,
}

impl Heartbeat {
    fn new(parent: WeakAddr<Registry>) -> Self {
        Self { parent }
    }
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

pub async fn subscribe_to_channels() -> Result<(), ChannelError> {
    Ok(())
}
