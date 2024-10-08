use crate::qset;
use maplit::hashset;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crate::{
    channels::messages::incoming::{Client, HeartbeatMessage, InfoMessage},
    data_structures::QueueSet,
};

use act_zero::runtimes::tokio::spawn_actor;
use act_zero::runtimes::tokio::Timer;
use act_zero::timer::Tick;
use act_zero::*;
use async_trait::async_trait;

use super::ServiceBroker;

pub(crate) type ActionName = String;
pub(crate) type EventName = String;
pub(crate) type NodeName = String;

pub(crate) struct Registry {
    actions: HashMap<EventName, QueueSet<NodeName>>,
    events: HashMap<EventName, QueueSet<NodeName>>,
    nodes: HashMap<NodeName, Node>,
}

impl Registry {
    pub(crate) fn new() -> Self {
        Self {
            actions: HashMap::new(),
            events: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    pub(crate) fn get_all_nodes_for_event(&self, event_name: &str) -> Option<Vec<NodeName>> {
        let event_nodes = self.events.get(event_name)?;

        Some(event_nodes.iter().cloned().collect())
    }

    pub(crate) fn get_node_name_for_event(&mut self, event_name: &str) -> Option<NodeName> {
        let event_nodes = self.events.get_mut(event_name)?;
        event_nodes.get_round_robin()
    }

    pub(crate) fn get_node_name_for_action(&mut self, action_name: &str) -> Option<NodeName> {
        let action_nodes = self.actions.get_mut(action_name)?;
        action_nodes.get_round_robin()
    }

    pub(crate) fn add_or_update_node(
        &mut self,
        broker: Addr<ServiceBroker>,
        heartbeat_timeout: u32,
        info: InfoMessage,
    ) {
        // get or insert node from/into registry
        let node: &mut Node = match self.nodes.get_mut(&info.sender) {
            Some(node) => node,
            None => {
                let node = Node::new(broker, heartbeat_timeout, &info);

                self.nodes.insert(info.sender.clone(), node);
                self.nodes
                    .get_mut(&info.sender)
                    .expect("present because just added the node")
            }
        };

        // get event_names  from info message
        let event_names = info
            .services
            .iter()
            .flat_map(|service| service.events.keys());
        for event_name in event_names {
            match self.events.get_mut(event_name) {
                // event present from another node, add node_name to event's node_names set
                Some(node_names) => {
                    node_names.insert(node.name.clone());
                }

                // first instance of event, create event name entry with node_name
                None => {
                    self.events
                        .insert(event_name.clone(), qset![node.name.clone()]);
                }
            }

            // insert event into node's events set
            node.events.insert(event_name.clone());
        }

        // get action_names from info message
        let action_names = info
            .services
            .iter()
            .flat_map(|service| service.actions.keys());

        for action_name in action_names {
            match self.actions.get_mut(action_name) {
                // action present from another node, add node_name to action's node_names set
                Some(node_names) => {
                    node_names.insert(node.name.clone());
                }

                // first instance of action, create action name entry with node_name
                None => {
                    self.actions
                        .insert(action_name.clone(), qset![node.name.clone()]);
                }
            }

            // insert action into node's actions set
            node.actions.insert(action_name.clone());
        }
    }

    pub(crate) fn remove_node(&mut self, node_name: NodeName) -> Option<()> {
        let node = self.nodes.remove(&node_name)?;

        for event_name in node.events {
            let node_names_left_for_event = self.events.get_mut(&event_name).map(|node_names| {
                node_names.remove(&node_name);
                node_names
            });

            // if the event doesn't have any associated nodes remove the event entirely
            if let Some(0) = node_names_left_for_event.map(|node_names| node_names.len()) {
                self.events.remove(&event_name);
            }
        }

        for action_name in node.actions {
            let node_names_left_for_action = self
                .actions
                .get_mut(&action_name)
                // go through the node's actions and remove node from each action
                .map(|node_names| {
                    node_names.remove(&node_name);
                    node_names
                });

            // if the action doesn't have any associated nodes remove the action entirely
            if let Some(0) = node_names_left_for_action.map(|node_names| node_names.len()) {
                self.actions.remove(&action_name);
            }
        }

        Some(())
    }

    pub(crate) fn update_node(&mut self, heartbeat: HeartbeatMessage) -> Option<()> {
        let node = self.nodes.get_mut(&heartbeat.sender)?;
        node.cpu = Some(heartbeat.cpu);

        send!(node.node_watcher_pid.received_heartbeat());

        Some(())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct Node {
    node_watcher_pid: Addr<NodeWatcher>,

    pub(crate) name: NodeName,
    pub(crate) cpu: Option<f32>,

    pub(crate) ip_list: Vec<String>,
    pub(crate) hostname: String,
    pub(crate) client: Client,
    pub(crate) instance_id: String,
    pub(crate) events: HashSet<EventName>,
    pub(crate) actions: HashSet<ActionName>,
}

impl Node {
    fn new(broker: Addr<ServiceBroker>, heartbeat_timeout: u32, info: &InfoMessage) -> Self {
        let node_watcher =
            NodeWatcher::new(info.sender.clone(), heartbeat_timeout, broker.downgrade());

        Self {
            node_watcher_pid: spawn_actor(node_watcher),
            name: info.sender.clone(),
            cpu: None,
            ip_list: info.ip_list.clone(),
            hostname: info.hostname.clone(),
            client: info.client.clone(),
            instance_id: info.instance_id.clone(),
            events: hashset![],
            actions: hashset![],
        }
    }
}

#[async_trait]
impl Actor for NodeWatcher {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        self.pid = pid.downgrade();

        // // Start the timer
        self.timer.set_timeout_for_weak(
            self.pid.clone(),
            Duration::from_secs(self.heartbeat_timeout as u64),
        );

        Produces::ok(())
    }
}

#[async_trait]
impl Tick for NodeWatcher {
    async fn tick(&mut self) -> ActorResult<()> {
        if self.timer.tick() {
            let now = Instant::now();

            if now.duration_since(self.last_heartbeat).as_secs() >= self.heartbeat_timeout as u64 {
                // haven't received a heartbeat recently
                send!(self.broker.missed_heartbeat(self.node_name.clone()))
            } else {
                // reschedule timer
                self.timer.set_timeout_for_weak(
                    self.pid.clone(),
                    Duration::from_secs(self.heartbeat_timeout as u64),
                );
            }
        }
        Produces::ok(())
    }
}

struct NodeWatcher {
    node_name: NodeName,
    pid: WeakAddr<Self>,
    broker: WeakAddr<ServiceBroker>,
    timer: Timer,

    heartbeat_timeout: u32,
    last_heartbeat: Instant,
}

impl NodeWatcher {
    pub(crate) fn new(
        name: NodeName,
        heartbeat_timeout: u32,
        broker: WeakAddr<ServiceBroker>,
    ) -> Self {
        Self {
            node_name: name,
            pid: WeakAddr::detached(),
            broker,
            timer: Timer::default(),

            heartbeat_timeout,
            last_heartbeat: Instant::now(),
        }
    }

    pub(crate) async fn received_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now()
    }
}
