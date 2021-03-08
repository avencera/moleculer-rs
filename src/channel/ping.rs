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

impl Actor for Ping {}
pub struct Ping {
    config: Arc<Config>,
    channel: Subscription,
    parent: WeakAddr<ChannelSupervisor>,
}

#[derive(Deserialize)]
struct PingMessage {
    ver: String,
    sender: String,
    id: String,
    time: i64,
}

#[derive(Serialize)]
struct PongMessage<'a> {
    ver: String,
    sender: &'a str,
    id: String,
    time: i64,
    arrived: i64,
}

impl<'a> From<(PingMessage, &'a str)> for PongMessage<'a> {
    fn from(from: (PingMessage, &'a str)) -> Self {
        let (ping, node_id) = from;

        Self {
            ver: ping.ver,
            id: ping.id,
            sender: node_id,
            time: ping.time,
            arrived: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("now should always be before unix epoch")
                .as_millis() as i64,
        }
    }
}

impl Ping {
    pub async fn new(
        parent: WeakAddr<ChannelSupervisor>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            parent,
            channel: conn
                .subscribe(&Channel::Ping.channel_to_string(&config))
                .await
                .unwrap(),
            config: Arc::clone(config),
        }
    }

    pub async fn listen(&mut self) {
        info!("Listening for PING messages");

        while let Some(msg) = self.channel.next().await {
            match self.handle_message(msg).await {
                Ok(_) => debug!("Successfully handled PING message"),
                Err(e) => error!("Unable to handle PING message: {}", e),
            }
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Error> {
        let ping_message: PingMessage = self.config.deserialize(&msg.data)?;
        let channel = format!(
            "{}.{}",
            Channel::PongPrefix.channel_to_string(&self.config),
            &ping_message.sender
        );

        let pong_message: PongMessage = (ping_message, self.config.node_id.as_str()).into();

        send!(self
            .parent
            .publish_to_channel(channel, self.config.serialize(pong_message)?));

        Ok(())
    }
}

impl Actor for PingTargeted {}
pub struct PingTargeted {
    config: Arc<Config>,
    channel: Subscription,
    parent: WeakAddr<ChannelSupervisor>,
}

impl PingTargeted {
    pub async fn new(
        parent: WeakAddr<ChannelSupervisor>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            parent,
            channel: conn
                .subscribe(&Channel::PingTargeted.channel_to_string(&config))
                .await
                .unwrap(),
            config: Arc::clone(config),
        }
    }

    pub async fn listen(&mut self) {
        info!("Listening for Ping messages");

        while let Some(msg) = self.channel.next().await {
            match self.handle_message(msg).await {
                Ok(_) => debug!("Successfully handled PING message"),
                Err(e) => error!("Unable to handle PING message: {}", e),
            }
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Error> {
        let ping_message: PingMessage = self.config.deserialize(&msg.data)?;
        let channel = format!(
            "{}.{}",
            Channel::PongPrefix.channel_to_string(&self.config),
            &ping_message.sender
        );

        let pong_message: PongMessage = (ping_message, self.config.node_id.as_str()).into();

        send!(self
            .parent
            .publish_to_channel(channel, self.config.serialize(pong_message)?));

        Ok(())
    }
}
