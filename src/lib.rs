mod util;

pub mod config;
pub mod service;

mod channel;
mod nats;

use std::sync::Arc;

use service::Service;
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

pub async fn start(config: config::Config, services: Vec<Service>) -> Result<(), Error> {
    let config = config.add_services(services);

    channel::subscribe_to_channels(Arc::new(config))
        .await
        .map_err(Error::ChannelError)
}
