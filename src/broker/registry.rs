use maplit::hashset;
use std::collections::{HashMap, HashSet};

use crate::channels::messages::incoming::{Client, InfoMessage};

pub type EventName = String;
pub type NodeName = String;

pub(crate) struct Registry {
    events: HashMap<EventName, HashSet<NodeName>>,
    nodes: HashMap<NodeName, Node>,
}

impl Registry {
    pub(crate) fn new() -> Self {
        Self {
            events: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    pub(crate) fn add_new_node_with_events(&mut self, info: InfoMessage) {
        // get or insert node from/into registry
        let node: &mut Node = match self.nodes.get_mut(&info.sender) {
            Some(node) => node,
            None => {
                let node = Node::from(&info);
                self.nodes.insert(info.sender.clone(), node);
                self.nodes
                    .get_mut(&info.sender)
                    .expect("present because just added the node")
            }
        };

        // get event_names from info message
        let event_names = info
            .services
            .iter()
            .flat_map(|service| service.events.keys().clone());

        for event_name in event_names {
            match self.events.get_mut(event_name) {
                // event present from another node, add node_name to event's node_names set
                Some(node_names) => {
                    node_names.insert(node.name.clone());
                }

                // first instance of event, create event name entry with node_name
                None => {
                    self.events
                        .insert(event_name.clone(), hashset![node.name.clone()]);
                }
            }

            // insert event into node's events set
            node.events.insert(event_name.clone());
        }
    }

    pub(crate) fn remove_node_with_events(&mut self, node_name: NodeName) -> Option<()> {
        let node = self.nodes.remove(&node_name)?;

        for event_name in node.events {
            let node_names_left = self
                .events
                .get_mut(&event_name)
                // go through the node's events and remove node from each event
                .and_then(|node_names| {
                    node_names.remove(&node_name);
                    Some(node_names)
                });

            // if the event doesn't have any associated nodes remove the event entirely
            if let Some(0) = node_names_left.map(|node_names| node_names.len()) {
                self.events.remove(&event_name);
            }
        }

        Some(())
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) name: NodeName,
    pub(crate) cpu: Option<f32>,
    pub(crate) ip_list: Vec<String>,
    pub(crate) hostname: String,
    pub(crate) client: Client,
    pub(crate) instance_id: String,
    pub(crate) events: HashSet<EventName>,
}

impl From<&InfoMessage> for Node {
    fn from(info: &InfoMessage) -> Self {
        Self {
            name: info.sender.clone(),
            cpu: None,
            ip_list: info.ip_list.clone(),
            hostname: info.hostname.clone(),
            client: info.client.clone(),
            instance_id: info.instance_id.clone(),
            events: hashset![],
        }
    }
}
