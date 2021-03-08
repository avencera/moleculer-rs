use crate::{
    config::{Channel, Config},
    nats::Conn,
};

use super::{ChannelSupervisor, Error};
use act_zero::*;
use async_nats::{Message, Subscription};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::SystemTime};

impl Actor for Discover {}
pub struct Discover {
    config: Arc<Config>,
    parent: WeakAddr<ChannelSupervisor>,
    channel: Subscription,
}

impl Discover {
    pub async fn new(
        parent: WeakAddr<ChannelSupervisor>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            parent,
            channel: conn
                .subscribe(&Channel::DiscoverTargeted.channel_to_string(&config))
                .await
                .unwrap(),
            config: Arc::clone(config),
        }
    }

    pub async fn listen(&mut self) {
        info!("Listening for DISCOVER messages");

        while let Some(msg) = self.channel.next().await {
            match self.handle_message(msg).await {
                Ok(_) => debug!("Successfully handled DISCOVER message"),
                Err(e) => error!("Unable to handle DISCOVER message: {}", e),
            }
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Error> {
        Ok(())
    }
}
