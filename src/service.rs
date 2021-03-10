use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::channel::messages::incoming::EventMessage;

pub type EventContext = EventMessage;

pub type ActionCallback = fn(Context) -> Option<Bytes>;
pub type EventCallback = fn(EventContext) -> ();

#[derive(Serialize, Deserialize, Debug)]
pub struct Action {
    name: String,
    params: HashMap<String, String>,
    #[serde(skip)]
    callback: Option<ActionCallback>,
}

#[derive(Default, Debug)]
pub struct EventBuilder {
    name: String,
    params: HashMap<String, String>,
    callback: Option<EventCallback>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    name: String,
    params: HashMap<String, String>,
    #[serde(skip)]
    pub callback: Option<EventCallback>,
}

impl EventBuilder {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    pub fn add_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = params;
        self
    }

    pub fn add_callback(mut self, callback: EventCallback) -> Self {
        self.callback = Some(callback);
        self
    }

    pub fn build(self) -> Event {
        Event {
            name: self.name,
            params: self.params,
            callback: self.callback,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    name: String,
    version: Option<i32>,

    settings: HashMap<String, String>,
    metadata: HashMap<String, String>,

    actions: HashMap<String, Action>,
    pub(crate) events: HashMap<String, Event>,
}

impl Service {
    pub fn new<S: Into<String>>(name: S) -> Service {
        Service {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn set_version(mut self, version: i32) -> Service {
        self.version = Some(version);
        self
    }

    pub fn add_action(mut self, action: Action) -> Service {
        self.actions.insert(action.name.clone(), action);
        self
    }

    pub fn add_event(mut self, event: Event) -> Service {
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
