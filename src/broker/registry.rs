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

    pub(crate) fn add_new_events(&mut self, info: InfoMessage) {
        // get or insert node
        let node = self
            .nodes
            .entry(info.sender.clone())
            .or_insert_with(|| Node::from(&info));

        // get event_names from info message
        let event_names = info
            .services
            .iter()
            .flat_map(|service| service.events.keys().clone());

        // insert event names into event HashSet
        for event_name in event_names {
            match self.events.get_mut(event_name) {
                Some(values) => {
                    values.insert(node.name.clone());
                }
                None => {
                    let mut values = HashSet::new();
                    values.insert(node.name.clone());
                    self.events.insert(node.name.clone(), values);
                }
            }
        }
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
        }
    }
}
