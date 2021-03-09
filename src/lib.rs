mod util;

pub mod config;
mod service;

mod channel;
mod nats;

use std::sync::Arc;

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

pub async fn start(config: config::Config) -> Result<(), Error> {
    channel::subscribe_to_channels(Arc::new(config))
        .await
        .map_err(Error::ChannelError)
}
