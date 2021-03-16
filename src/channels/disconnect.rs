use crate::{
    broker::ServiceBroker,
    config::{Channel, Config},
    nats::Conn,
};

use super::{messages::incoming::DisconnectMessage, Error};
use act_zero::*;
use async_nats::Message;
use async_trait::async_trait;
use log::{debug, error, info};
use std::sync::Arc;

#[async_trait]
impl Actor for Disconnect {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        let pid_clone = pid.clone();
        send!(pid_clone.listen(pid));
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("Disconnect Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
pub struct Disconnect {
    broker: WeakAddr<ServiceBroker>,
    config: Arc<Config>,
    conn: Conn,
}

impl Disconnect {
    pub async fn new(broker: WeakAddr<ServiceBroker>, config: &Arc<Config>, conn: &Conn) -> Self {
        Self {
            broker,
            conn: conn.clone(),
            config: Arc::clone(config),
        }
    }

    pub async fn listen(&mut self, pid: Addr<Self>) {
        info!("Listening for DISCONNECT messages");

        let channel = self
            .conn
            .subscribe(&Channel::Disconnect.channel_to_string(&self.config))
            .await
            .unwrap();

        pid.clone().send_fut(async move {
            while let Some(msg) = channel.next().await {
                match call!(pid.handle_message(msg)).await {
                    Ok(_) => debug!("Successfully handled DISCONNECT message"),
                    Err(e) => error!("Unable to handle DISCONNECT message: {}", e),
                }
            }
        })
    }

    async fn handle_message(&self, msg: Message) -> ActorResult<()> {
        let disconnect_msg: DisconnectMessage = self.config.serializer.deserialize(&msg.data)?;

        send!(self.broker.handle_disconnect_message(disconnect_msg));

        Produces::ok(())
    }
}
