use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ActionCallback = fn(Context) -> Option<Bytes>;
pub type EventCallback = fn(Context) -> ();

#[derive(Debug)]
pub struct Action {
    name: String,
    params: HashMap<String, String>,
    callback: ActionCallback,
}

#[derive(Debug)]
pub struct Event {
    name: String,
    params: HashMap<String, String>,
    callback: EventCallback,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    name: String,
    version: Option<i32>,
    #[serde(skip)]
    actions: HashMap<String, Action>,
    #[serde(skip)]
    events: HashMap<String, Event>,
}

impl Service {
    pub fn new<S: Into<String>>(name: S) -> Service {
        Service {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn version(mut self, version: i32) -> Service {
        self.version = Some(version);
        self
    }

    pub fn action(mut self, action: Action) -> Service {
        self.actions.insert(action.name.clone(), action);
        self
    }

    pub fn event(mut self, event: Event) -> Service {
        self.events.insert(event.name.clone(), event);
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Emit,
    Broadcast,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    id: String,
    broker: String,
    #[serde(rename = "nodeID")]
    node_id: String,
    action: Option<String>,

    event: Option<String>,
    event_name: Option<String>,
    event_type: Option<EventType>,
    event_groups: Vec<String>,

    caller: String,
    #[serde(rename = "requestID")]
    request_id: String,
    #[serde(rename = "parentID")]
    parent_id: String,

    params: Bytes,
    meta: Bytes,
    locals: Bytes,

    level: i32,
}
