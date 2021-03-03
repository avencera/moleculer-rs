mod util;

mod config;
mod service;

mod channel;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ChannelError(#[from] channel::Error),
}

pub async fn start() -> Result<(), Error> {
    channel::subscribe_to_channels(config::Config::default())
        .await
        .map_err(Error::ChannelError)
}
