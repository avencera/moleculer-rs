use crate::{
    broker::ServiceBroker,
    channels::messages::incoming::ResponseMessage,
    config::{self, Channel, Config},
    nats::Conn,
};

use act_zero::*;
use async_nats::Message;
use async_trait::async_trait;
use config::DeserializeError;
use log::{debug, error, info};
use std::sync::Arc;

#[async_trait]
impl Actor for Response {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        let pid_clone = pid.clone();
        send!(pid_clone.listen(pid));
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("Response Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
pub struct Response {
    config: Arc<Config>,
    broker: WeakAddr<ServiceBroker>,
    conn: Conn,
}

impl Response {
    pub async fn new(broker: WeakAddr<ServiceBroker>, config: &Arc<Config>, conn: &Conn) -> Self {
        Self {
            broker,
            conn: conn.clone(),
            config: Arc::clone(config),
        }
    }

    pub async fn listen(&mut self, pid: Addr<Self>) {
        info!("Listening for REQUEST messages");
        let channel = self
            .conn
            .subscribe(&Channel::Response.channel_to_string(&self.config))
            .await
            .unwrap();

        pid.clone().send_fut(async move {
            while let Some(msg) = channel.next().await {
                match call!(pid.handle_message(msg)).await {
                    Ok(_) => debug!("Successfully handled REQUEST message"),
                    Err(e) => error!("Unable to handle REQUEST message: {}", e),
                }
            }
        })
    }

    async fn handle_message(&self, msg: Message) -> ActorResult<()> {
        // let request_context: Result<ResponseMessage, DeserializeError> =
        //     self.config.serializer.deserialize(&msg.data);

        Produces::ok(())
    }
}
