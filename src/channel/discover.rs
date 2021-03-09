use crate::{
    config::{Channel, Config},
    nats::Conn,
};

use super::{messages::incoming, messages::outgoing, ChannelSupervisor, Error};
use act_zero::*;
use async_nats::{Message, Subscription};
use async_trait::async_trait;
use log::{debug, error, info};
use std::sync::Arc;

#[async_trait]
impl Actor for Discover {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        send!(pid.listen());
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("Discover Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
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
                .subscribe(&Channel::Discover.channel_to_string(&config))
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

    pub async fn broadcast(&self) {
        let msg = outgoing::DiscoverMessage::new(&self.config.node_id);
        send!(self.parent.publish(
            Channel::Discover,
            self.config
                .serialize(msg)
                .expect("should always serialize discover msg")
        ));
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Error> {
        let discover: incoming::DiscoverMessage = self.config.deserialize(&msg.data)?;
        let info = outgoing::InfoMessage::new(&self.config);
        let channel = format!(
            "{}.{}",
            Channel::Info.channel_to_string(&self.config),
            discover.sender
        );

        send!(self
            .parent
            .publish_to_channel(channel, self.config.serialize(info)?));

        Ok(())
    }
}

#[async_trait]
impl Actor for DiscoverTargeted {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        send!(pid.listen());
        send!(self.parent.broadcast_discover());
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("DiscoverTargeted Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}

// This one shouldn't be used to much, DISCOVER packets are usually sent to the DISCOVER broadcast channel
pub struct DiscoverTargeted {
    config: Arc<Config>,
    parent: WeakAddr<ChannelSupervisor>,
    channel: Subscription,
}

impl DiscoverTargeted {
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
        info!("Listening for DISCOVER (targeted) messages");

        while let Some(msg) = self.channel.next().await {
            match self.handle_message(msg).await {
                Ok(_) => debug!("Successfully handled DISCOVER (targeted) message"),
                Err(e) => error!("Unable to handle DISCOVER (targeted) message: {}", e),
            }
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Error> {
        let discover: incoming::DiscoverMessage = self.config.deserialize(&msg.data)?;
        let info = outgoing::InfoMessage::new(&self.config);
        let channel = format!(
            "{}.{}",
            Channel::Info.channel_to_string(&self.config),
            discover.sender
        );

        send!(self
            .parent
            .publish_to_channel(channel, self.config.serialize(info)?));

        Ok(())
    }
}
