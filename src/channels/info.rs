use crate::{
    broker::ServiceBroker,
    config::{Channel, Config},
    nats::Conn,
};

use super::messages::incoming::InfoMessage;
use act_zero::*;
use async_nats::Message;
use async_trait::async_trait;
use futures::StreamExt as _;
use log::{debug, error, info};
use std::sync::Arc;

#[async_trait]
impl Actor for Info {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        let pid_clone = pid.clone();
        send!(pid_clone.listen(pid));
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("Info Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
pub(crate) struct Info {
    config: Arc<Config>,
    broker: WeakAddr<ServiceBroker>,
    conn: Conn,
}

impl Info {
    pub(crate) async fn new(
        broker: WeakAddr<ServiceBroker>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            broker,
            conn: conn.clone(),
            config: Arc::clone(config),
        }
    }

    // INFO packets received when a new client connects and broadcasts it's INFO
    pub(crate) async fn listen(&mut self, pid: Addr<Self>) {
        info!("Listening for INFO messages");
        let mut channel = self
            .conn
            .subscribe(&Channel::Info.channel_to_string(&self.config))
            .await
            .unwrap();

        pid.clone().send_fut(async move {
            while let Some(msg) = channel.next().await {
                match call!(pid.handle_message(msg)).await {
                    Ok(_) => debug!("Successfully handled INFO message"),
                    Err(e) => error!("Unable to handle INFO message: {}", e),
                }
            }
        })
    }

    async fn handle_message(&self, msg: Message) -> ActorResult<()> {
        let info_message: InfoMessage = self.config.serializer.deserialize(&msg.payload)?;
        send!(self.broker.handle_info_message(info_message));

        Produces::ok(())
    }
}

#[async_trait]
impl Actor for InfoTargeted {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        let pid_clone = pid.clone();
        send!(pid_clone.listen(pid));
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("InfoTargeted Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
pub(crate) struct InfoTargeted {
    config: Arc<Config>,
    broker: WeakAddr<ServiceBroker>,
    conn: Conn,
}

impl InfoTargeted {
    pub(crate) async fn new(
        broker: WeakAddr<ServiceBroker>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            broker,
            conn: conn.clone(),
            config: Arc::clone(config),
        }
    }
    // INFO packets received are responses to DISCOVER packet sent by current client
    pub(crate) async fn listen(&mut self, pid: Addr<Self>) {
        info!("Listening for INFO (targeted) messages");

        let mut channel = self
            .conn
            .subscribe(&Channel::InfoTargeted.channel_to_string(&self.config))
            .await
            .unwrap();

        pid.clone().send_fut(async move {
            while let Some(msg) = channel.next().await {
                match call!(pid.handle_message(msg)).await {
                    Ok(_) => debug!("Successfully handled INFO message in response to DISCOVER"),
                    Err(e) => error!(
                        "Unable to handle INFO message in response to DISCOVER: {}",
                        e
                    ),
                }
            }
        })
    }

    async fn handle_message(&self, msg: Message) -> ActorResult<()> {
        let info_message: InfoMessage = self.config.serializer.deserialize(&msg.payload)?;
        send!(self.broker.handle_info_message(info_message));

        Produces::ok(())
    }
}
