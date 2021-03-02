use log;
use serde::{Deserialize, Serialize};

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
    #[serde(rename = "transit.maxQueueSize")]
    transit_max_queue_size: u32,
    #[serde(rename = "transit.maxChunkSize")]
    transit_max_chunk_size: u32,
    #[serde(rename = "transit.disableReconnect")]
    transit_disable_reconnect: bool,
    #[serde(rename = "transit.disableVersionCheck")]
    transit_disable_version_check: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Logger {
    Console,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Transporter {
    Nats,
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
#[serde(rename_all = "camelCase")]
pub struct Registry {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CircuitBreaker {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bulkhead {}
