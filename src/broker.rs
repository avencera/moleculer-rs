mod registry;

use std::{collections::HashMap, sync::Arc};

use act_zero::*;
use async_trait::async_trait;
use log::{error, warn};
use serde_json::Value;

use crate::{
    channels::messages::{
        incoming::{
            DisconnectMessage, EventMessage, HeartbeatMessage, InfoMessage, RequestMessage,
        },
        outgoing::{self},
    },
    service::Action,
};

use crate::{
    channels::{self, ChannelSupervisor},
    config::{self, Channel, DeserializeError, Serializer},
    service::{Context, Event, Service},
};

use thiserror::Error;

use self::registry::Registry;

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

    #[error("Unable to find action '{0}' in registry")]
    ActionNotFound(String),

    #[error("Unable to find callback function for action '{0}'")]
    ActionCallbackNotFound(String),

    #[error("Call back function failed to complete: {0}")]
    ActionCallbackFailed(String),

    #[error("Node not found for ('{0}') event or action")]
    NodeNotFound(String),
}

#[allow(dead_code)]
pub struct ServiceBroker {
    pub(crate) namespace: String,
    pub(crate) node_id: String,
    pub(crate) instance_id: String,
    pub(crate) serializer: Serializer,
    pub(crate) services: Vec<Service>,

    pub(crate) events: Events,
    pub(crate) actions: Actions,

    pub(crate) registry: Registry,

    pid: Addr<Self>,
    channel_supervisor: Addr<ChannelSupervisor>,
    config: Arc<config::Config>,
}

pub struct Events(HashMap<String, Event>);
pub struct Actions(HashMap<String, Action>);

impl Events {
    fn new() -> Self {
        Events(HashMap::new())
    }

    fn get(&self, key: &str) -> Option<&Event> {
        self.0.get(key)
    }
}

impl From<&Vec<Service>> for Events {
    fn from(services: &Vec<Service>) -> Self {
        Events(
            services
                .iter()
                .flat_map(|service| service.events.clone())
                .collect(),
        )
    }
}

impl Actions {
    fn new() -> Self {
        Actions(HashMap::new())
    }

    fn get(&self, key: &str) -> Option<&Action> {
        self.0.get(key)
    }
}

impl From<&Vec<Service>> for Actions {
    fn from(services: &Vec<Service>) -> Self {
        Actions(
            services
                .iter()
                .flat_map(|service| service.actions.clone())
                .collect(),
        )
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

            registry: Registry::new(),
            events: Events::new(),
            actions: Actions::new(),

            pid: Addr::detached(),
            channel_supervisor: Addr::detached(),
            config: Arc::new(config),
        }
    }

    // exposed publicly via crate::ServiceBroker
    pub(crate) async fn emit(&mut self, event_name: String, params: Value) -> ActorResult<()> {
        let node_name = self
            .registry
            .get_node_name_for_event(&event_name)
            .ok_or_else(|| Error::NodeNotFound(event_name.clone()))?;

        let node_event_channel = Channel::Event.external_channel(&self.config, node_name);

        let message = outgoing::EventMessage::new_for_emit(&self.config, &event_name, params);

        send!(self
            .channel_supervisor
            .publish_to_channel(node_event_channel, serde_json::to_vec(&message)?));

        Produces::ok(())
    }

    pub(crate) async fn broadcast(&self, event_name: String, params: Value) -> ActorResult<()> {
        let node_names = self
            .registry
            .get_all_nodes_for_event(&event_name)
            .ok_or_else(|| Error::NodeNotFound(event_name.clone()))?;

        let message = outgoing::EventMessage::new_for_broadcast(&self.config, &event_name, params);

        for node_name in node_names {
            let node_event_channel = Channel::Event.external_channel(&self.config, node_name);

            send!(self
                .channel_supervisor
                .publish_to_channel(node_event_channel, serde_json::to_vec(&message)?));
        }

        Produces::ok(())
    }

    pub(crate) async fn reply(&self, node: String, id: String, reply: Value) -> ActorResult<()> {
        let message = outgoing::ResponseMessage::new(&self.config, &id, reply);

        let reply_channel = Channel::Response.external_channel(&self.config, node);

        send!(self
            .channel_supervisor
            .publish_to_channel(reply_channel, serde_json::to_vec(&message)?));

        Produces::ok(())
    }

    // private

    pub(crate) async fn handle_info_message(&mut self, info: InfoMessage) {
        if self.node_id != info.sender {
            self.registry
                .reconcile_node(self.pid.clone(), self.config.heartbeat_timeout, info);
        }
    }

    pub(crate) async fn handle_disconnect_message(&mut self, disconnect: DisconnectMessage) {
        if self.node_id != disconnect.sender {
            self.registry.remove_node_with_events(disconnect.sender);
        }
    }

    pub(crate) async fn missed_heartbeat(&mut self, node_name: String) {
        warn!(
            "Node {} expectedly disconnected (missed heartbeat)",
            &node_name
        );
        self.registry.remove_node_with_events(node_name);
    }

    pub(crate) async fn handle_heartbeat_message(&mut self, heartbeat: HeartbeatMessage) {
        if self.node_id != heartbeat.sender {
            self.registry.update_node(heartbeat);
        }
    }

    pub(crate) async fn add_service(&mut self, service: Service) {
        self.services.push(service);
        self.events = (&self.services).into();
        self.actions = (&self.services).into();
    }

    pub(crate) async fn add_services(&mut self, services: Vec<Service>) {
        for service in services {
            self.add_service(service).await;
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

        let event_context = Context::<Event>::new(event_message, self.pid.clone().into());

        callback(event_context).map_err(|err| Error::EventCallbackFailed(err.to_string()))?;

        Produces::ok(())
    }

    pub(crate) async fn handle_incoming_request(
        &self,
        request_message: Result<RequestMessage, DeserializeError>,
    ) -> ActorResult<()> {
        let request_message = request_message?;

        let request = self
            .actions
            .get(&request_message.action)
            .ok_or_else(|| Error::ActionNotFound(request_message.action.clone()))?;

        let callback = request
            .callback
            .clone()
            .ok_or_else(|| Error::ActionCallbackNotFound(request_message.action.clone()))?;

        let request_context = Context::<Action>::new(request_message, self.pid.clone().into());

        callback(request_context).map_err(|err| Error::ActionCallbackFailed(err.to_string()))?;

        Produces::ok(())
    }

    async fn broadcast_info(&self) -> ActorResult<()> {
        self.publish_info_to_channel(Channel::Info.channel_to_string(&self.config))
            .await
    }
}
