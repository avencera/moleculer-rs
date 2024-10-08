use crate::{
    config::{Channel, Config},
    nats::Conn,
};

use super::{
    messages::{incoming::PingMessage, outgoing::PongMessage},
    ChannelSupervisor,
};

use act_zero::*;
use async_nats::Message;
use async_trait::async_trait;
use futures::StreamExt as _;
use log::{debug, error, info};
use std::sync::Arc;

#[async_trait]
impl Actor for Ping {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        let pid_clone = pid.clone();
        send!(pid_clone.listen(pid));
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("Ping Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
pub(crate) struct Ping {
    config: Arc<Config>,
    conn: Conn,
    parent: WeakAddr<ChannelSupervisor>,
}

impl Ping {
    pub(crate) async fn new(
        parent: WeakAddr<ChannelSupervisor>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            parent,
            conn: conn.clone(),
            config: Arc::clone(config),
        }
    }

    pub(crate) async fn listen(&mut self, pid: Addr<Self>) {
        info!("Listening for PING messages");

        let mut channel = self
            .conn
            .subscribe(&Channel::Ping.channel_to_string(&self.config))
            .await
            .unwrap();

        pid.clone().send_fut(async move {
            while let Some(msg) = channel.next().await {
                match call!(pid.handle_message(msg)).await {
                    Ok(_) => debug!("Successfully handled PING message"),
                    Err(e) => error!("Unable to handle PING message: {}", e),
                }
            }
        })
    }

    async fn handle_message(&self, msg: Message) -> ActorResult<()> {
        let ping_message: PingMessage = self.config.serializer.deserialize(&msg.payload)?;
        let channel = format!(
            "{}.{}",
            Channel::PongPrefix.channel_to_string(&self.config),
            &ping_message.sender
        );

        let pong_message: PongMessage = (ping_message, self.config.node_id.as_str()).into();

        send!(self
            .parent
            .publish_to_channel(channel, self.config.serializer.serialize(pong_message)?));

        Produces::ok(())
    }
}

#[async_trait]
impl Actor for PingTargeted {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        let pid_clone = pid.clone();
        send!(pid_clone.listen(pid));
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("PingTargeted Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
pub(crate) struct PingTargeted {
    config: Arc<Config>,
    conn: Conn,
    parent: WeakAddr<ChannelSupervisor>,
}

impl PingTargeted {
    pub(crate) async fn new(
        parent: WeakAddr<ChannelSupervisor>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            parent,
            conn: conn.clone(),
            config: Arc::clone(config),
        }
    }

    pub(crate) async fn listen(&mut self, pid: Addr<Self>) {
        info!("Listening for PING (targeted) messages");

        let mut channel = self
            .conn
            .subscribe(&Channel::PingTargeted.channel_to_string(&self.config))
            .await
            .unwrap();

        pid.clone().send_fut(async move {
            while let Some(msg) = channel.next().await {
                match call!(pid.handle_message(msg)).await {
                    Ok(_) => debug!("Successfully handled PING message"),
                    Err(e) => error!("Unable to handle PING message: {}", e),
                }
            }
        })
    }

    async fn handle_message(&self, msg: Message) -> ActorResult<()> {
        let ping_message: PingMessage = self.config.serializer.deserialize(&msg.payload)?;
        let channel = format!(
            "{}.{}",
            Channel::PongPrefix.channel_to_string(&self.config),
            &ping_message.sender
        );

        let pong_message: PongMessage = (ping_message, self.config.node_id.as_str()).into();

        send!(self
            .parent
            .publish_to_channel(channel, self.config.serializer.serialize(pong_message)?));

        Produces::ok(())
    }
}
