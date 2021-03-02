use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    namespace: String,
    #[serde(rename = "nodeID")]
    node_id: String,
    logger: Logger,
    log_level: log::Level,
    transporter: Transporter,
    request_timeout: i32,
    retry_policy: RetryPolicy,
    context_params_cloning: bool,
    dependency_internal: u32,
    max_call_level: u32,
    heartbeat_interval: u32,
    heartbeat_timeout: u32,
    tracking: Tracking,
    disable_balancer: bool,
    registry: Registry,
    circuit_breaker: CircuitBreaker,
    bulkhead: Bulkhead,
    transit: Transit,
    serializer: Serializer,
    meta_data: HashMap<String, String>,
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
