use crate::{
    config::{Channel, Config},
    nats::Conn,
};

use super::{ChannelSupervisor, Error};
use act_zero::runtimes::tokio::Timer;
use act_zero::timer::Tick;
use act_zero::*;
use async_nats::{Message, Subscription};
use async_trait::async_trait;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use sysinfo::{ProcessorExt, RefreshKind, System, SystemExt};

pub struct Heartbeat {
    config: Arc<Config>,
    timer: Timer,
    channel: Subscription,
    parent: Addr<ChannelSupervisor>,
    heartbeat_interval: u32,
    system: sysinfo::System,
}

#[async_trait]
impl Actor for Heartbeat {
    async fn started(&mut self, addr: Addr<Self>) -> ActorResult<()> {
        // Start the timer
        self.timer
            .set_timeout_for_strong(addr, Duration::from_secs(self.heartbeat_interval as u64));

        Produces::ok(())
    }
}

#[async_trait]
impl Tick for Heartbeat {
    async fn tick(&mut self) -> ActorResult<()> {
        self.system.refresh_cpu();

        if self.timer.tick() {
            let _ = self.send_heartbeat().await;
        }
        Produces::ok(())
    }
}

impl Heartbeat {
    pub async fn new(
        parent: WeakAddr<ChannelSupervisor>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            config: Arc::clone(config),
            parent: parent.upgrade(),
            channel: conn
                .subscribe(&Channel::Heartbeat.channel_to_string(&config))
                .await
                .unwrap(),
            heartbeat_interval: config.heartbeat_interval,
            timer: Timer::default(),
            system: System::new_with_specifics(RefreshKind::new().with_cpu()),
        }
    }

    pub async fn listen(&mut self) {
        info!("Listening for HEARTBEAT messages");

        while let Some(msg) = self.channel.next().await {
            match self.handle_message(msg).await {
                Ok(_) => debug!("Successfully handled HEARTBEAT message"),
                Err(e) => error!("Unable to handle HEARTBEAT message: {}", e),
            }
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Error> {
        // let heartbeat_msg: HeartbeatMessageOwned = self.config.deserialize(&msg.data)?;
        // do nothing with incoming heartbeat messages for now
        Ok(())
    }

    async fn send_heartbeat(&self) -> ActorResult<()> {
        let msg = HeartbeatMessage {
            ver: "4",
            sender: &self.config.node_id,
            cpu: self.system.get_global_processor_info().get_cpu_usage(),
        };

        send!(self
            .parent
            .publish(Channel::Heartbeat, self.config.serialize(msg)?));

        Produces::ok(())
    }
}

#[derive(Serialize)]
struct HeartbeatMessage<'a> {
    ver: &'static str,
    sender: &'a str,
    cpu: f32,
}

#[derive(Deserialize)]
struct HeartbeatMessageOwned {
    ver: String,
    sender: String,
    cpu: f32,
}
