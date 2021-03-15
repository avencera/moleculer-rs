use crate::{
    config::{Channel, Config},
    nats::Conn,
};

use super::messages::outgoing::HeartbeatMessage;
use super::{ChannelSupervisor, Error};
use act_zero::runtimes::tokio::Timer;
use act_zero::timer::Tick;
use act_zero::*;
use async_nats::{Message, Subscription};
use async_trait::async_trait;
use log::{debug, error, info};
use std::{sync::Arc, time::Duration};
use sysinfo::{ProcessorExt, RefreshKind, System, SystemExt};

pub struct Heartbeat {
    pid: Addr<Self>,
    config: Arc<Config>,
    timer: Timer,
    conn: Conn,
    parent: Addr<ChannelSupervisor>,
    heartbeat_interval: u32,
    system: sysinfo::System,
}

#[async_trait]
impl Actor for Heartbeat {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        let pid_clone = pid.clone();
        send!(pid_clone.listen(pid));

        // Start the timer
        self.timer.set_timeout_for_strong(
            pid_clone.clone(),
            Duration::from_secs(self.heartbeat_interval as u64),
        );

        self.pid = pid_clone;

        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("Heartbeat Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}

#[async_trait]
impl Tick for Heartbeat {
    async fn tick(&mut self) -> ActorResult<()> {
        self.system.refresh_cpu();

        if self.timer.tick() {
            self.timer.set_timeout_for_strong(
                self.pid.clone(),
                Duration::from_secs(self.heartbeat_interval as u64),
            );
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
            pid: Addr::detached(),
            config: Arc::clone(config),
            parent: parent.upgrade(),
            conn: conn.clone(),
            heartbeat_interval: config.heartbeat_interval,
            timer: Timer::default(),
            system: System::new_with_specifics(RefreshKind::new().with_cpu()),
        }
    }

    pub async fn listen(&mut self, pid: Addr<Self>) {
        info!("Listening for HEARTBEAT messages");

        let channel = self
            .conn
            .subscribe(&Channel::Heartbeat.channel_to_string(&self.config))
            .await
            .unwrap();

        pid.clone().send_fut(async move {
            while let Some(msg) = channel.next().await {
                match call!(pid.handle_message(msg)).await {
                    Ok(_) => debug!("Successfully handled HEARTBEAT message"),
                    Err(e) => error!("Unable to handle HEARTBEAT message: {}", e),
                }
            }
        })
    }

    async fn handle_message(&self, _msg: Message) -> ActorResult<()> {
        // TODO: handle and save to registry
        // let heartbeat_msg: HeartbeatMessageOwned = self.config.serializer.deserialize(&msg.data)?;
        // do nothing with incoming heartbeat messages for now
        Produces::ok(())
    }

    async fn send_heartbeat(&self) -> ActorResult<()> {
        let msg = HeartbeatMessage::new(
            &self.config.node_id,
            self.system.get_global_processor_info().get_cpu_usage(),
        );

        send!(self
            .parent
            .publish(Channel::Heartbeat, self.config.serializer.serialize(msg)?));

        Produces::ok(())
    }
}
