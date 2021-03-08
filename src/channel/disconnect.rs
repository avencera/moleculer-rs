use crate::{
    config::{Channel, Config},
    nats::Conn,
};

use super::{ChannelSupervisor, Error};
use act_zero::*;
use async_nats::{Message, Subscription};
use async_trait::async_trait;
use log::{debug, error, info};
use std::sync::Arc;

#[async_trait]
impl Actor for Disconnect {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        send!(pid.listen());
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("Disconnect Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
pub struct Disconnect {
    parent: WeakAddr<ChannelSupervisor>,
    config: Arc<Config>,
    channel: Subscription,
}

impl Disconnect {
    pub async fn new(
        parent: WeakAddr<ChannelSupervisor>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            parent,
            channel: conn
                .subscribe(&Channel::Disconnect.channel_to_string(&config))
                .await
                .unwrap(),
            config: Arc::clone(config),
        }
    }

    pub async fn listen(&mut self) {
        info!("Listening for DISCONNECT messages");

        while let Some(msg) = self.channel.next().await {
            match self.handle_message(msg).await {
                Ok(_) => debug!("Successfully handled DISCONNECT message"),
                Err(e) => error!("Unable to handle DISCONNECT message: {}", e),
            }
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Error> {
        // let disconnect_msg: DisconnectMessageOwned = self.config.deserialize(&msg.data)?;
        // do nothing with incoming disconnect messages for now
        Ok(())
    }
}
