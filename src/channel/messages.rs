pub mod incoming {
    use std::collections::HashMap;

    use serde::Deserialize;
    use serde_json::Value;

    use crate::service::Service;

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Client {
        #[serde(rename = "type")]
        type_: String,
        version: String,
        lang_version: String,
    }

    #[derive(Deserialize)]
    pub struct PingMessage {
        pub ver: String,
        pub sender: String,
        pub id: String,
        pub time: i64,
    }

    #[derive(Deserialize)]
    pub struct HeartbeatMessage {
        pub ver: String,
        pub sender: String,
        pub cpu: f32,
    }

    #[derive(Deserialize)]
    pub struct DisconnectMessage {
        pub ver: String,
        pub sender: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InfoMessage {
        ver: String,
        sender: String,

        services: Vec<Service>,
        ip_list: Vec<String>,
        hostname: String,
        client: Client,

        #[serde(rename = "instanceID")]
        instance_id: String,

        config: HashMap<String, String>,
        metadata: HashMap<String, String>,
    }

    #[derive(Deserialize)]
    pub struct DiscoverMessage {
        pub ver: String,
        pub sender: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct EventMessage {
        pub id: String,
        pub sender: String,
        pub ver: String,

        pub event: String,

        #[serde(default)]
        pub data: Value,

        #[serde(default)]
        pub meta: Value,
        pub level: i32,

        #[serde(default)]
        pub tracing: Option<bool>,

        #[serde(rename = "parentID", default)]
        pub parent_id: Option<String>,

        #[serde(rename = "requestID", default)]
        pub request_id: Option<String>,

        #[serde(rename = "caller", default)]
        pub caller: Option<String>,

        #[serde(default)]
        pub stream: Option<bool>,

        #[serde(default)]
        pub seq: Option<i32>,

        #[serde(default)]
        pub groups: Option<Vec<String>>,

        #[serde(default)]
        pub broadcast: Option<bool>,
    }
}

pub mod outgoing {
    use std::{collections::HashMap, time::SystemTime};

    use super::incoming::PingMessage;
    use crate::{built_info, config::Config, service::Service};
    use serde::Serialize;

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Client {
        #[serde(rename = "type")]
        type_: &'static str,
        version: &'static str,
        lang_version: &'static str,
    }

    impl Client {
        fn new() -> Self {
            Self {
                type_: "rust",
                version: env!("CARGO_PKG_VERSION"),
                lang_version: built_info::RUSTC_VERSION,
            }
        }
    }

    #[derive(Serialize)]
    pub struct PongMessage<'a> {
        ver: String,
        sender: &'a str,
        id: String,
        time: i64,
        arrived: i64,
    }

    impl<'a> From<(PingMessage, &'a str)> for PongMessage<'a> {
        fn from(from: (PingMessage, &'a str)) -> Self {
            let (ping, node_id) = from;

            Self {
                ver: ping.ver,
                id: ping.id,
                sender: node_id,
                time: ping.time,
                arrived: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("now should always be before unix epoch")
                    .as_millis() as i64,
            }
        }
    }

    #[derive(Serialize)]
    pub struct HeartbeatMessage<'a> {
        ver: &'static str,
        sender: &'a str,
        cpu: f32,
    }

    impl<'a> HeartbeatMessage<'a> {
        pub fn new(sender: &'a str, cpu: f32) -> Self {
            Self {
                ver: "4",
                sender,
                cpu,
            }
        }
    }

    #[derive(Serialize)]
    pub struct DisconnectMessage<'a> {
        ver: &'static str,
        sender: &'a str,
    }

    impl<'a> DisconnectMessage<'a> {
        pub fn new(sender: &'a str) -> Self {
            Self { ver: "4", sender }
        }
    }

    #[derive(Serialize)]
    pub struct DiscoverMessage<'a> {
        ver: &'static str,
        sender: &'a str,
    }

    impl<'a> DiscoverMessage<'a> {
        pub fn new(sender: &'a str) -> Self {
            Self { ver: "4", sender }
        }
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InfoMessage<'a> {
        ver: &'static str,
        sender: &'a str,

        #[serde(rename = "instanceID")]
        instance_id: &'a str,
        services: &'a Vec<Service>,
        ip_list: &'a Vec<String>,
        hostname: &'a str,
        client: Client,

        config: HashMap<String, String>,
        metadata: HashMap<String, String>,
    }

    impl<'a> InfoMessage<'a> {
        pub fn new(config: &'a Config) -> Self {
            Self {
                ver: "4",
                sender: &config.node_id,

                instance_id: &config.instance_id,
                services: &config.services,
                ip_list: &config.ip_list,
                hostname: &config.hostname,
                client: Client::new(),

                config: HashMap::new(),
                metadata: HashMap::new(),
            }
        }
    }
}
