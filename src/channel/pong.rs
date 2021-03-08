use crate::{
    config::{Channel, Config},
    nats::Conn,
};

use super::{ChannelSupervisor, Error};
use act_zero::*;
use async_nats::{Message, Subscription};
use log::{debug, error, info};
use serde::Deserialize;
use std::{sync::Arc, time::SystemTime};

impl Actor for Pong {}
pub struct Pong {
    config: Arc<Config>,
    channel: Subscription,
    parent: WeakAddr<ChannelSupervisor>,
}

#[derive(Deserialize)]
struct PongMessage {
    ver: String,
    sender: String,
    id: String,
    time: i64,
    arrived: i64,
}

impl Pong {
    pub async fn new(
        parent: WeakAddr<ChannelSupervisor>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            parent,
            channel: conn
                .subscribe(&Channel::Pong.channel_to_string(&config))
                .await
                .unwrap(),
            config: Arc::clone(config),
        }
    }

    pub async fn listen(&mut self) {
        info!("Listening for PONG messages");

        while let Some(msg) = self.channel.next().await {
            match self.handle_message(msg).await {
                Ok(_) => debug!("Successfully handled PONG message"),
                Err(e) => error!("Unable to handle PONG message: {}", e),
            }
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Error> {
        // let pong_msg: PongMessage = self.config.deserialize(&msg.data)?;
        // do nothing with incoming disconnect messages for now
        Ok(())
    }
}
