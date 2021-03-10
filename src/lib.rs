mod util;

pub mod config;
pub mod service;

mod broker;
mod channel;
mod nats;

use act_zero::runtimes::tokio::spawn_actor;
use act_zero::*;
use bytes::Bytes;
use config::Config;
use service::Service;

pub(crate) mod built_info {
    #[allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Clone)]
pub struct ServiceBroker {
    addr: Addr<broker::ServiceBroker>,
}

impl ServiceBroker {
    pub fn new(config: Config) -> ServiceBroker {
        ServiceBroker {
            addr: spawn_actor(broker::ServiceBroker::new(config)),
        }
    }

    pub fn add_service(self, service: Service) -> Self {
        send!(self.addr.add_service(service));
        self
    }

    pub fn add_services(self, services: Vec<Service>) -> Self {
        send!(self.addr.add_services(services));
        self
    }

    pub async fn start(self) {
        self.addr.termination().await
    }

    pub fn emit(&self, event: String, params: Bytes) {}
}

impl From<Addr<broker::ServiceBroker>> for ServiceBroker {
    fn from(addr: Addr<broker::ServiceBroker>) -> Self {
        Self { addr }
    }
}
