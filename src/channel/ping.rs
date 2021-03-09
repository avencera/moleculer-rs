use crate::{
    config::{Channel, Config},
    nats::Conn,
};

use super::{
    messages::{incoming::PingMessage, outgoing::PongMessage},
    ChannelSupervisor, Error,
};

use act_zero::*;
use async_nats::{Message, Subscription};
use async_trait::async_trait;
use log::{debug, error, info};
use std::sync::Arc;

#[async_trait]
impl Actor for Ping {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        send!(pid.listen());
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("Ping Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
pub struct Ping {
    config: Arc<Config>,
    channel: Subscription,
    parent: WeakAddr<ChannelSupervisor>,
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

#[async_trait]
impl Actor for PingTargeted {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        send!(pid.listen());
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("PingTargeted Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
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
        info!("Listening for PING (targeted) messages");

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
