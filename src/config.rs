use crate::util;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub namespace: String,
    #[serde(rename = "nodeID")]
    pub node_id: String,
    pub logger: Logger,
    pub log_level: log::Level,
    pub transporter: Transporter,
    pub request_timeout: i32,
    pub retry_policy: RetryPolicy,
    pub context_params_cloning: bool,
    pub dependency_internal: u32,
    pub max_call_level: u32,
    pub heartbeat_interval: u32,
    pub heartbeat_timeout: u32,
    pub tracking: Tracking,
    pub disable_balancer: bool,
    pub registry: Registry,
    pub circuit_breaker: CircuitBreaker,
    pub bulkhead: Bulkhead,
    pub transit: Transit,
    pub serializer: Serializer,
    pub meta_data: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Logger {
    Console,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Transporter {
    Nats(String),
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

#[derive(Serialize, Deserialize, Debug)]
pub enum Serializer {
    JSON,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            namespace: "".to_string(),
            node_id: util::gen_node_id(),
            logger: Logger::Console,
            log_level: log::Level::Info,
            transporter: Transporter::Nats("nats://localhost:4222".to_string()),
            request_timeout: 0,
            retry_policy: RetryPolicy::default(),
            context_params_cloning: false,
            dependency_internal: 1000,
            max_call_level: 0,
            heartbeat_interval: 5,
            heartbeat_timeout: 15,
            tracking: Tracking::default(),
            disable_balancer: false,
            registry: Registry::Local,
            circuit_breaker: CircuitBreaker::default(),
            bulkhead: Bulkhead::default(),
            transit: Transit::default(),
            serializer: Serializer::JSON,
            meta_data: HashMap::new(),
        }
    }
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

#[derive(Error, Debug)]
pub enum SerializeError {
    #[error("Unable to serialize to json: {0}")]
    JSON(serde_json::error::Error),
}

impl Config {
    pub fn serialize<T: Serialize>(&self, msg: T) -> Result<Vec<u8>, SerializeError> {
        match self.serializer {
            Serializer::JSON => serde_json::to_vec(&msg).map_err(SerializeError::JSON),
        }
    }
}
