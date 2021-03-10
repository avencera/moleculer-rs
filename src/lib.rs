mod util;

pub mod config;
pub mod service;

mod broker;
mod channel;
mod nats;

pub(crate) mod built_info {
    #[allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub type ServiceBroker = broker::ServiceBroker;
