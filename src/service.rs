//! Create [Service] struct with [Events][Event] and [Actions][Action]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, marker::PhantomData};

use crate::{
    channels::messages::incoming::{EventMessage, RequestMessage},
    ServiceBroker,
};

pub type Callback<T> = fn(Context<T>) -> Result<(), Box<dyn std::error::Error>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    name: String,
    #[serde(default)]
    params: Option<Value>,
    #[serde(skip)]
    pub callback: Option<Callback<Action>>,
}

#[derive(Default, Debug)]
pub struct EventBuilder {
    name: String,
    params: Option<Value>,
    callback: Option<Callback<Event>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    name: String,
    #[serde(default)]
    params: Option<Value>,
    #[serde(skip)]
    pub callback: Option<Callback<Event>>,
}

impl EventBuilder {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    pub fn add_params(mut self, params: Value) -> Self {
        self.params = Some(params);
        self
    }

    pub fn add_callback(mut self, callback: Callback<Event>) -> Self {
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

#[derive(Default, Debug)]
pub struct ActionBuilder {
    name: String,
    params: Option<Value>,
    callback: Option<Callback<Action>>,
}

impl ActionBuilder {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    pub fn add_params(mut self, params: Value) -> Self {
        self.params = Some(params);
        self
    }

    pub fn add_callback(mut self, callback: Callback<Action>) -> Self {
        self.callback = Some(callback);
        self
    }

    pub fn build(self) -> Action {
        Action {
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

    #[serde(default)]
    #[serde(skip_deserializing)]
    settings: HashMap<String, String>,
    #[serde(default)]
    metadata: Option<Value>,

    pub actions: HashMap<String, Action>,
    pub events: HashMap<String, Event>,
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

pub struct Context<T> {
    phantom: PhantomData<T>,

    pub id: String,
    pub broker: ServiceBroker,
    pub node_id: String,
    pub action: Option<String>,

    pub event_name: Option<String>,
    pub event_type: Option<EventType>,
    pub event_groups: Vec<String>,

    pub caller: Option<String>,
    pub request_id: Option<String>,
    pub parent_id: Option<String>,

    pub params: Value,
    pub meta: Value,
    pub locals: Option<Value>,

    pub level: i32,
}

impl Context<Event> {
    pub(crate) fn new(event_message: EventMessage, service_broker: ServiceBroker) -> Self {
        let event_type = if event_message.broadcast.unwrap_or(false) {
            EventType::Broadcast
        } else {
            EventType::Emit
        };

        Self {
            phantom: PhantomData,

            broker: service_broker,
            id: event_message.id,
            params: event_message.data,

            action: None,

            event_type: Some(event_type),
            event_name: Some(event_message.event),
            event_groups: vec![],

            node_id: event_message.sender,
            caller: event_message.caller,
            parent_id: event_message.parent_id,
            request_id: event_message.request_id,

            meta: event_message.meta,
            level: event_message.level,

            locals: None,
        }
    }
}

impl Context<Action> {
    pub(crate) fn new(request_message: RequestMessage, service_broker: ServiceBroker) -> Self {
        Self {
            phantom: PhantomData,

            broker: service_broker,
            id: request_message.request_id.clone(),
            params: request_message.params,

            action: Some(request_message.action),

            event_type: None,
            event_name: None,
            event_groups: vec![],

            node_id: request_message.sender,
            caller: request_message.caller,
            parent_id: request_message.parent_id,
            request_id: Some(request_message.request_id),

            meta: request_message.meta,
            level: 1,

            locals: None,
        }
    }

    pub fn reply(&self, params: Value) {
        act_zero::send!(self
            .broker
            .addr
            .reply(self.node_id.clone(), self.id.clone(), params));
    }
}

impl<T> Context<T> {
    pub fn emit<S: Into<String>>(&self, event: S, params: Value) {
        self.broker.emit(event, params)
    }

    pub fn broadcast<S: Into<String>>(&self, event: S, params: Value) {
        self.broker.broadcast(event, params)
    }
}
