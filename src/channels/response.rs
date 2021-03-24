use crate::{
    channels::messages::incoming::ResponseMessage,
    config::{Channel, Config},
    nats::Conn,
};

use act_zero::runtimes::tokio::{spawn_actor, Timer};
use act_zero::timer::Tick;
use act_zero::*;
use async_nats::Message;
use async_trait::async_trait;
use log::{debug, error, info};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::oneshot::Sender;

type RequestId = String;

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
pub(crate) struct Response {
    config: Arc<Config>,
    waiters: HashMap<RequestId, Addr<ResponseWaiter>>,
    conn: Conn,
}

impl Response {
    pub(crate) async fn new(config: &Arc<Config>, conn: &Conn) -> Self {
        Self {
            conn: conn.clone(),
            config: Arc::clone(config),
            waiters: HashMap::new(),
        }
    }

    pub(crate) async fn start_response_waiter(
        &mut self,
        timeout: i32,
        node_name: String,
        request_id: RequestId,
        tx: Sender<Value>,
    ) {
        let response_waiter_pid = spawn_actor(ResponseWaiter::new(
            timeout,
            request_id.clone(),
            node_name,
            tx,
        ));

        self.waiters.insert(request_id, response_waiter_pid);
    }

    pub(crate) async fn listen(&mut self, pid: Addr<Self>) {
        info!("Listening for RESPONSE messages");
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

    async fn timeout_reached(&mut self, request_id: String) {
        self.waiters.remove(&request_id);
    }

    async fn handle_message(&mut self, msg: Message) -> ActorResult<()> {
        let response: ResponseMessage = self.config.serializer.deserialize(&msg.data)?;
        let response_id = response.id.clone();

        if let Some(response_waiter) = self.waiters.get(&response_id) {
            let response_waiter = response_waiter.clone();

            // wether send_response succeeds or fails we should remove it from hashmap
            let _ = call!(response_waiter.send_response(response)).await;
            self.waiters.remove(&response_id);
        }

        Produces::ok(())
    }
}

#[async_trait]
impl Actor for ResponseWaiter {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        self.pid = pid.downgrade();

        // Start the timer
        self.timer
            .set_timeout_for_weak(pid.downgrade(), Duration::from_millis(self.timeout as u64));

        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("ResponseWaiter Actor Error: {:?}", error);

        // stop actor on actor error
        true
    }
}

#[async_trait]
impl Tick for ResponseWaiter {
    async fn tick(&mut self) -> ActorResult<()> {
        if self.timer.tick() {
            send!(self.parent.timeout_reached(self.request_id.clone()))
        }
        Produces::ok(())
    }
}

struct ResponseWaiter {
    parent: WeakAddr<Response>,
    pid: WeakAddr<Self>,
    request_id: RequestId,

    timeout: i32,
    node_name: String,
    tx: Option<Sender<Value>>,

    timer: Timer,
}

impl ResponseWaiter {
    fn new(timeout: i32, request_id: RequestId, node_name: String, tx: Sender<Value>) -> Self {
        Self {
            parent: WeakAddr::detached(),
            pid: WeakAddr::detached(),

            request_id,
            timeout,
            node_name,
            tx: Some(tx),

            timer: Timer::default(),
        }
    }

    async fn send_response(&mut self, response: ResponseMessage) -> ActorResult<()> {
        if self.node_name != response.sender {
            // something went wrong here, should handle this error better
            error!("Node name does not match sender")
        }

        // take the tx from actor state and replace it with a none
        let tx = std::mem::take(&mut self.tx).unwrap();

        tx.send(response.data).unwrap();
        Produces::ok(())
    }
}
