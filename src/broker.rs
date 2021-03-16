use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use act_zero::*;
use async_trait::async_trait;
use log::error;

use crate::{
    channels::{
        self,
        messages::{
            incoming::{Client, EventMessage, InfoMessage},
            outgoing::{self},
        },
        ChannelSupervisor,
    },
    config::{self, Channel, DeserializeError, Serializer},
    service::{Context, Event, Service},
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ChannelError(#[from] channels::Error),

    #[error("Unable to deserialize to EventContext: {0}")]
    EventDeserializeFail(#[from] config::DeserializeError),

    #[error("Unable to find event '{0}' in registry")]
    EventNotFound(String),

    #[error("Unable to find callback function for event '{0}'")]
    EventCallbackNotFound(String),

    #[error("Call back function failed to complete: {0}")]
    EventCallbackFailed(String),
}

pub struct ServiceBroker {
    pub(crate) namespace: String,
    pub(crate) node_id: String,
    pub(crate) instance_id: String,
    pub(crate) serializer: Serializer,

    pub(crate) services: Vec<Service>,

    pub(crate) events: InternalEvents,

    pub(crate) external_events: ExternalEvents,

    pid: Addr<Self>,
    channel_supervisor: Addr<ChannelSupervisor>,
    config: Arc<config::Config>,
}

pub struct InternalEvents(HashMap<String, Event>);

impl InternalEvents {
    fn new() -> Self {
        InternalEvents(HashMap::new())
    }

    fn get(&self, key: &str) -> Option<&Event> {
        self.0.get(key)
    }
}

impl From<&Vec<Service>> for InternalEvents {
    fn from(services: &Vec<Service>) -> Self {
        InternalEvents(
            services
                .iter()
                .flat_map(|service| service.events.clone())
                .collect(),
        )
    }
}

pub type EventName = String;
pub type NodeName = String;
pub struct ExternalEvents {
    events: HashMap<EventName, HashSet<NodeName>>,
    nodes: HashMap<NodeName, Node>,
}

impl ExternalEvents {
    fn new() -> Self {
        Self {
            events: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    fn add_new_events(&mut self, info: InfoMessage) {
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

#[async_trait]
impl Actor for ServiceBroker {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        self.pid = pid.clone();

        let channel_supervisor = channels::start_supervisor(pid, Arc::clone(&self.config))
            .await
            .map_err(Error::ChannelError)?;

        send!(self.pid.broadcast_info());
        send!(channel_supervisor.broadcast_discover());

        self.channel_supervisor = channel_supervisor.clone();

        self.pid
            .send_fut(async move { channels::listen_for_disconnect(channel_supervisor).await });

        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        log::error!("ServiceBroker Actor Error: {:?}", error);
        // do not stop on actor error
        false
    }
}
impl ServiceBroker {
    pub(crate) fn new(config: config::Config) -> Self {
        Self {
            namespace: config.namespace.clone(),
            node_id: config.node_id.clone(),
            instance_id: config.instance_id.clone(),
            serializer: config.serializer.clone(),

            services: vec![],

            events: InternalEvents::new(),
            external_events: ExternalEvents::new(),

            pid: Addr::detached(),
            channel_supervisor: Addr::detached(),
            config: Arc::new(config),
        }
    }

    // private
    async fn broadcast_info(&self) -> ActorResult<()> {
        self.publish_info_to_channel(Channel::Info.channel_to_string(&self.config))
            .await
    }

    pub(crate) async fn handle_info_message(&mut self, info: InfoMessage) {
        self.external_events.add_new_events(info);
    }

    pub(crate) async fn add_service(&mut self, service: Service) {
        self.services.push(service);
        self.events = (&self.services).into();
    }

    pub(crate) async fn add_services(&mut self, services: Vec<Service>) {
        for service in services {
            self.services.push(service);
        }
    }

    pub(crate) async fn publish_info_to_channel(&self, channel: String) -> ActorResult<()> {
        let info = outgoing::InfoMessage::new(&self.config, &self.services);
        send!(self
            .channel_supervisor
            .publish_to_channel(channel, self.serializer.serialize(info)?));

        Produces::ok(())
    }

    pub(crate) async fn handle_incoming_event(
        &self,
        event_message: Result<EventMessage, DeserializeError>,
    ) -> ActorResult<()> {
        let event_message = event_message?;

        let event = self
            .events
            .get(&event_message.event)
            .ok_or_else(|| Error::EventNotFound(event_message.event.clone()))?;

        let callback = event
            .callback
            .clone()
            .ok_or_else(|| Error::EventCallbackNotFound(event_message.event.clone()))?;

        let event_context = Context::new(event_message, self.pid.clone().into());

        callback(event_context).map_err(|err| Error::EventCallbackFailed(err.to_string()))?;

        Produces::ok(())
    }
}
