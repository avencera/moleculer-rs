mod data_structures;
mod util;

pub mod config;
pub mod service;

mod broker;
mod channels;
mod nats;

use act_zero::runtimes::tokio::spawn_actor;
use act_zero::*;
use config::Config;
use serde_json::Value;
use service::Service;
use thiserror::Error;
use tokio::sync::oneshot::{self, error};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to receive response")]
    ReceiveError(#[from] error::RecvError),

    #[error("Unknown error")]
    UnknownError,
}

#[allow(dead_code)]
pub(crate) mod built_info {
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

    pub async fn call<S: Into<String>>(self, action: S, params: Value) -> Result<Value, Error> {
        let (tx, rx) = oneshot::channel();

        send!(self.addr.call(action.into(), params, tx));
        let response_value = rx.await?;

        Ok(response_value)
    }

    pub fn emit<S: Into<String>>(&self, event: S, params: Value) {
        send!(self.addr.emit(event.into(), params))
    }

    pub fn broadcast<S: Into<String>>(&self, event: S, params: Value) {
        send!(self.addr.broadcast(event.into(), params))
    }
}

impl From<Addr<broker::ServiceBroker>> for ServiceBroker {
    fn from(addr: Addr<broker::ServiceBroker>) -> Self {
        Self { addr }
    }
}
