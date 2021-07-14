/*!
Create Config struct using [ConfigBuilder] with global settings for you micro-service.

```rust
use moleculer::config::{ConfigBuilder, Transporter};

let config = ConfigBuilder::default()
    .transporter(Transporter::nats("nats://localhost:4222"))
    .build();
```
*/

use crate::util;
use derive_builder::Builder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::{borrow::Cow, fmt::Display};
use strum::{EnumIter, IntoEnumIterator};
use thiserror::Error;
use uuid::Uuid;

#[derive(Serialize, Debug, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(pattern = "owned")]
#[builder(build_fn(name = "build_private", private))]
#[builder(setter(into, strip_option))]
pub struct Config {
    #[builder(default = "\"\".to_string()")]
    pub(crate) namespace: String,
    #[serde(rename = "nodeID")]
    #[builder(default = "util::gen_node_id()")]
    pub(crate) node_id: String,
    #[builder(default = "Logger::Console")]
    pub(crate) logger: Logger,
    #[builder(default = "log::Level::Info")]
    pub(crate) log_level: log::Level,
    #[builder(default = "Transporter::nats(\"nats://localhost:4222\")")]
    pub(crate) transporter: Transporter,
    #[builder(default = "1000 * 60 * 5")]
    pub(crate) request_timeout: i32,
    #[builder(default)]
    pub(crate) retry_policy: RetryPolicy,
    #[builder(default = "false")]
    pub(crate) context_params_cloning: bool,
    #[builder(default = "1000")]
    pub(crate) dependency_internal: u32,
    #[builder(default = "0")]
    pub(crate) max_call_level: u32,
    #[builder(default = "5")]
    pub(crate) heartbeat_interval: u32,
    #[builder(default = "15")]
    pub(crate) heartbeat_timeout: u32,
    #[builder(default)]
    pub(crate) tracking: Tracking,
    #[builder(default = "false")]
    pub(crate) disable_balancer: bool,
    #[builder(default = "Registry::Local")]
    pub(crate) registry: Registry,
    #[builder(default)]
    pub(crate) circuit_breaker: CircuitBreaker,
    #[builder(default)]
    pub(crate) bulkhead: Bulkhead,
    #[builder(default)]
    pub(crate) transit: Transit,
    #[builder(default = "Serializer::JSON")]
    pub(crate) serializer: Serializer,
    #[builder(default)]
    pub(crate) meta_data: HashMap<String, String>,

    #[builder(setter(skip), default = "util::ip_list()")]
    pub(crate) ip_list: Vec<String>,
    #[builder(setter(skip), default = "util::hostname().into_owned()")]
    pub(crate) hostname: String,
    #[builder(setter(skip), default = "Uuid::new_v4().to_string()")]
    pub(crate) instance_id: String,
}

impl ConfigBuilder {
    pub fn build(self) -> Config {
        self.build_private()
            .expect("will always work because all fields have defaults")
    }
}

#[derive(Debug, Serialize)]
pub enum Logger {
    Console,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Transporter {
    Nats(String),
}

impl Transporter {
    /// Create a NATS transporter with address, ex:
    /// `Transporter::nats("nats://localhost:4222")`
    pub fn nats<S: Into<String>>(nats_address: S) -> Self {
        Self::Nats(nats_address.into())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RetryPolicy {
    enabled: bool,
    retries: u32,
    delay: u32,
    max_delay: u32,
    factor: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tracking {
    enabled: bool,
    shutdown_timeout: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Serializer {
    JSON,
}

impl Serializer {
    pub(crate) fn serialize<T: Serialize>(&self, msg: T) -> Result<Vec<u8>, SerializeError> {
        match self {
            Serializer::JSON => serde_json::to_vec(&msg).map_err(SerializeError::JSON),
        }
    }

    pub(crate) fn deserialize<T: DeserializeOwned>(
        &self,
        msg: &[u8],
    ) -> Result<T, DeserializeError> {
        match self {
            Serializer::JSON => serde_json::from_slice(msg).map_err(DeserializeError::JSON),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Registry {
    Local,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CircuitBreaker {
    enabled: bool,
    threshold: f32,
    min_request_count: u32,
    window_time: u32,
    half_open_time: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bulkhead {
    enabled: bool,
    concurrency: u32,
    max_queue_size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Transit {
    max_queue_size: u32,
    max_chunk_size: u32,
    disable_reconnect: bool,
    disable_version_check: bool,
    packet_log_filter: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            retries: 5,
            delay: 100,
            max_delay: 2000,
            factor: 2,
        }
    }
}

impl Default for Tracking {
    fn default() -> Self {
        Self {
            enabled: false,
            shutdown_timeout: 10000,
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: 0.5,
            min_request_count: 20,
            window_time: 60,
            half_open_time: 10000,
        }
    }
}

impl Default for Bulkhead {
    fn default() -> Self {
        Self {
            enabled: false,
            concurrency: 3,
            max_queue_size: 10,
        }
    }
}

impl Default for Transit {
    fn default() -> Self {
        Transit {
            max_queue_size: 50_000,
            max_chunk_size: 256,
            disable_reconnect: false,
            disable_version_check: false,
            packet_log_filter: vec![],
        }
    }
}

#[derive(EnumIter, Debug, PartialEq, Hash, Eq, Clone)]
pub(crate) enum Channel {
    Event,
    Request,
    Response,
    Discover,
    DiscoverTargeted,
    Info,
    InfoTargeted,
    Heartbeat,
    Ping,
    PongPrefix,
    Pong,
    PingTargeted,
    Disconnect,
}

impl Channel {
    pub(crate) fn build_hashmap(config: &Config) -> HashMap<Channel, String> {
        Channel::iter()
            .map(|channel| (channel.clone(), channel.channel_to_string(config)))
            .collect()
    }

    pub(crate) fn channel_to_string(&self, config: &Config) -> String {
        match self {
            Channel::Event => format!("{}.EVENT.{}", mol(config), &config.node_id),
            Channel::Request => format!("{}.REQ.{}", mol(config), &config.node_id),
            Channel::Response => format!("{}.RES.{}", mol(config), &config.node_id),
            Channel::Discover => format!("{}.DISCOVER", mol(config)),
            Channel::DiscoverTargeted => format!("{}.DISCOVER.{}", mol(config), &config.node_id),
            Channel::Info => format!("{}.INFO", mol(config)),
            Channel::InfoTargeted => format!("{}.INFO.{}", mol(config), &config.node_id),
            Channel::Heartbeat => format!("{}.HEARTBEAT", mol(config)),
            Channel::Ping => format!("{}.PING", mol(config)),
            Channel::PingTargeted => format!("{}.PING.{}", mol(config), &config.node_id),
            Channel::PongPrefix => format!("{}.PONG", mol(config)),
            Channel::Pong => format!("{}.PONG.{}", mol(config), &config.node_id),
            Channel::Disconnect => format!("{}.DISCONNECT", mol(config)),
        }
    }

    pub(crate) fn external_channel<S>(&self, config: &Config, node_name: S) -> String
    where
        S: AsRef<str> + Display,
    {
        match self {
            Channel::Event => format!("{}.EVENT.{}", mol(config), node_name),
            Channel::Response => format!("{}.RES.{}", mol(config), node_name),
            Channel::Request => format!("{}.REQ.{}", mol(config), node_name),
            _ => unreachable!(),
        }
    }
}

#[derive(Error, Debug)]
pub(crate) enum SerializeError {
    #[error("Unable to serialize to json: {0}")]
    JSON(serde_json::error::Error),
}

#[derive(Error, Debug)]
pub(crate) enum DeserializeError {
    #[error("Unable to deserialize from json: {0}")]
    JSON(serde_json::error::Error),
}

fn mol(config: &Config) -> Cow<str> {
    if config.namespace.is_empty() {
        Cow::Borrowed("MOL")
    } else {
        Cow::Owned(format!("MOL-{}", &config.namespace))
    }
}
