/*!
Create Config struct using [ConfigBuilder] with global settings for you micro-service.

```rust
let config = ConfigBuilder {
    transporter: Transporter::nats("nats://localhost:4222"),
    ..ConfigBuilder::default()
}
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

// #[derive(Debug)]
// pub struct ConfigBuilder {
//     pub namespace: String,
//     pub node_id: String,
//     pub logger: Logger,
//     pub log_level: log::Level,
//     pub transporter: Transporter,
//     pub request_timeout: i32,
//     pub retry_policy: RetryPolicy,
//     pub context_params_cloning: bool,
//     pub dependency_internal: u32,
//     pub max_call_level: u32,
//     pub heartbeat_interval: u32,
//     pub heartbeat_timeout: u32,
//     pub tracking: Tracking,
//     pub disable_balancer: bool,
//     pub registry: Registry,
//     pub circuit_breaker: CircuitBreaker,
//     pub bulkhead: Bulkhead,
//     pub transit: Transit,
//     pub serializer: Serializer,
//     pub meta_data: HashMap<String, String>,
// }

// impl ConfigBuilder {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     pub fn build(self) -> Config {
//         Config {
//             namespace: self.namespace,
//             node_id: self.node_id,
//             logger: self.logger,
//             log_level: self.log_level,
//             transporter: self.transporter,
//             request_timeout: self.request_timeout,
//             retry_policy: self.retry_policy,
//             context_params_cloning: self.context_params_cloning,
//             dependency_internal: self.dependency_internal,
//             max_call_level: self.max_call_level,
//             heartbeat_interval: self.heartbeat_interval,
//             heartbeat_timeout: self.heartbeat_timeout,
//             tracking: self.tracking,
//             disable_balancer: self.disable_balancer,
//             registry: self.registry,
//             circuit_breaker: self.circuit_breaker,
//             bulkhead: self.bulkhead,
//             transit: self.transit,
//             serializer: self.serializer,
//             meta_data: self.meta_data,

//             hostname: util::hostname().into_owned(),
//             instance_id: Uuid::new_v4().to_string(),
//             ip_list: get_if_addrs::get_if_addrs()
//                 .unwrap_or_default()
//                 .iter()
//                 .map(|interface| interface.addr.ip())
//                 .filter(|ip| ip.is_ipv4() && !ip.is_loopback())
//                 .map(|ip| ip.to_string())
//                 .collect(),
//         }
//     }
// }

// impl Default for ConfigBuilder {
//     fn default() -> Self {
//         Self {
//             namespace: "".to_string(),
//             node_id: util::gen_node_id(),
//             logger: Logger::Console,
//             log_level: log::Level::Info,
//             transporter: Transporter::Nats("nats://localhost:4222".to_string()),
//             // 5 minute timeout
//             request_timeout: 1000 * 60 * 5,
//             retry_policy: RetryPolicy::default(),
//             context_params_cloning: false,
//             dependency_internal: 1000,
//             max_call_level: 0,
//             heartbeat_interval: 5,
//             heartbeat_timeout: 15,
//             tracking: Tracking::default(),
//             disable_balancer: false,
//             registry: Registry::Local,
//             circuit_breaker: CircuitBreaker::default(),
//             bulkhead: Bulkhead::default(),
//             transit: Transit::default(),
//             serializer: Serializer::JSON,
//             meta_data: HashMap::new(),
//         }
//     }
// }

#[derive(Serialize, Debug, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(pattern = "owned")]
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
    #[builder(default = "Transporter::Nats(\"nats://localhost:4222\".to_string())")]
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
            Channel::Event => format!("{}.EVENT.{}", mol(&config), &config.node_id),
            Channel::Request => format!("{}.REQ.{}", mol(&config), &config.node_id),
            Channel::Response => format!("{}.RES.{}", mol(&config), &config.node_id),
            Channel::Discover => format!("{}.DISCOVER", mol(&config)),
            Channel::DiscoverTargeted => format!("{}.DISCOVER.{}", mol(&config), &config.node_id),
            Channel::Info => format!("{}.INFO", mol(&config)),
            Channel::InfoTargeted => format!("{}.INFO.{}", mol(&config), &config.node_id),
            Channel::Heartbeat => format!("{}.HEARTBEAT", mol(&config)),
            Channel::Ping => format!("{}.PING", mol(&config)),
            Channel::PingTargeted => format!("{}.PING.{}", mol(&config), &config.node_id),
            Channel::PongPrefix => format!("{}.PONG", mol(&config)),
            Channel::Pong => format!("{}.PONG.{}", mol(&config), &config.node_id),
            Channel::Disconnect => format!("{}.DISCONNECT", mol(&config)),
        }
    }

    pub(crate) fn external_channel<S>(&self, config: &Config, node_name: S) -> String
    where
        S: AsRef<str> + Display,
    {
        match self {
            Channel::Event => format!("{}.EVENT.{}", mol(&config), node_name),
            Channel::Response => format!("{}.RES.{}", mol(&config), node_name),
            Channel::Request => format!("{}.REQ.{}", mol(&config), node_name),
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
