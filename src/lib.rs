mod util;

mod config;
mod service;

mod channel;
mod nats;

use std::sync::Arc;

use thiserror::Error;

pub(crate) mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ChannelError(#[from] channel::Error),
}

pub async fn start() -> Result<(), Error> {
    channel::subscribe_to_channels(Arc::new(config::Config::default()))
        .await
        .map_err(Error::ChannelError)
}
