use async_nats::{ConnectErrorKind, Subject, SubscribeError, Subscriber};
use bytes::Bytes;
use log::{error, warn};
use thiserror::Error;

type Result<T> = std::result::Result<T, self::Error>;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Unable to connect to NATS: {0}")]
    UnableToConnect(#[from] async_nats::error::Error<ConnectErrorKind>),

    #[error("Unable to subscribe to channel ({0}): {1}")]
    UnableToSubscribe(String, SubscribeError),
}

#[derive(Clone)]
pub(crate) struct Conn {
    pub(crate) conn: async_nats::Client,
}

impl Conn {
    pub(crate) async fn new(nats_address: &str) -> Result<Conn> {
        let conn = async_nats::connect(nats_address)
            .await
            .map_err(Error::UnableToConnect)?;

        Ok(Conn { conn })
    }

    pub(crate) async fn send(&self, channel: &str, message: Vec<u8>) -> Result<()> {
        let message = Bytes::from(message);
        let channel = Subject::from(channel);

        let mut retries: i8 = 0;
        let mut result = self.conn.publish(channel.clone(), message.clone()).await;

        // keep retrying if publish fails
        while result.is_err() {
            let channel = channel.clone();
            let message = message.clone();

            retries += 1;
            let error_message = format!("Failed to send message, failed {} times", retries);

            // before 5 retries log as warning, as error after
            if retries < 5 {
                warn!("{}", &error_message)
            } else {
                error!("{}", &error_message)
            }

            result = self.conn.publish(channel, message).await;
        }

        Ok(())
    }

    pub(crate) async fn subscribe(&self, channel: &str) -> Result<Subscriber> {
        let channel = Subject::from(channel);

        self.conn
            .subscribe(channel.clone())
            .await
            .map_err(|e| Error::UnableToSubscribe(channel.to_string(), e))
    }
}
