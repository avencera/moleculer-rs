pub(crate) mod incoming {
    use std::collections::HashMap;

    use serde::Deserialize;
    use serde_json::Value;

    use crate::service::Service;

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Client {
        #[serde(rename = "type")]
        type_: String,
        version: String,
        lang_version: String,
    }

    #[derive(Deserialize, Debug)]
    pub(crate) struct PingMessage {
        pub(crate) ver: String,
        pub(crate) sender: String,
        pub(crate) id: String,
        pub(crate) time: i64,
    }

    #[derive(Deserialize, Debug)]
    pub(crate) struct HeartbeatMessage {
        pub(crate) ver: String,
        pub(crate) sender: String,
        pub(crate) cpu: f32,
    }

    #[derive(Deserialize, Debug)]
    pub(crate) struct DisconnectMessage {
        pub(crate) ver: String,
        pub(crate) sender: String,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct InfoMessage {
        pub(crate) ver: String,
        pub(crate) sender: String,

        pub(crate) services: Vec<Service>,
        pub(crate) ip_list: Vec<String>,
        pub(crate) hostname: String,
        pub(crate) client: Client,

        #[serde(rename = "instanceID")]
        pub(crate) instance_id: String,

        pub(crate) config: HashMap<String, String>,
        pub(crate) metadata: HashMap<String, String>,
    }

    #[derive(Deserialize, Debug)]
    pub(crate) struct DiscoverMessage {
        pub(crate) ver: String,
        pub(crate) sender: String,
    }

    #[derive(Deserialize, Debug)]
    pub(crate) struct EventMessage {
        pub(crate) id: String,
        pub(crate) sender: String,
        pub(crate) ver: String,

        pub(crate) event: String,

        #[serde(default)]
        pub(crate) data: Value,

        #[serde(default)]
        pub(crate) meta: Value,
        pub(crate) level: i32,

        #[serde(default)]
        pub(crate) tracing: Option<bool>,

        #[serde(rename = "parentID", default)]
        pub(crate) parent_id: Option<String>,

        #[serde(rename = "requestID", default)]
        pub(crate) request_id: Option<String>,

        #[serde(rename = "caller", default)]
        pub(crate) caller: Option<String>,

        #[serde(default)]
        pub(crate) stream: Option<bool>,

        #[serde(default)]
        pub(crate) seq: Option<i32>,

        #[serde(default)]
        pub(crate) groups: Option<Vec<String>>,

        #[serde(default)]
        pub(crate) broadcast: Option<bool>,
    }

    #[derive(Deserialize, Debug)]
    pub(crate) struct RequestMessage {
        pub(crate) id: String,
        pub(crate) sender: String,
        pub(crate) ver: String,

        pub(crate) action: String,

        #[serde(default)]
        pub(crate) params: Value,

        #[serde(default)]
        pub(crate) meta: Value,

        pub(crate) timeout: f32,
        pub(crate) level: i32,

        #[serde(default)]
        pub(crate) tracing: Option<bool>,

        #[serde(rename = "parentID", default)]
        pub(crate) parent_id: Option<String>,

        #[serde(rename = "requestID", default)]
        pub(crate) request_id: String,

        #[serde(rename = "caller", default)]
        pub(crate) caller: Option<String>,

        #[serde(default)]
        pub(crate) stream: Option<bool>,

        #[serde(default)]
        pub(crate) seq: Option<i32>,
    }
    #[derive(Deserialize, Debug)]
    pub(crate) struct ResponseMessage {
        pub(crate) id: String,
        pub(crate) sender: String,
        pub(crate) ver: String,

        #[serde(default)]
        pub(crate) data: Value,

        #[serde(default)]
        pub(crate) meta: Value,

        #[serde(default)]
        pub(crate) error: Option<crate::channels::messages::MoleculerError>,

        #[serde(default)]
        pub(crate) success: bool,
    }
}

pub(crate) mod outgoing {
    use std::{collections::HashMap, time::SystemTime};

    use super::incoming::PingMessage;
    use crate::{built_info, config::Config, service::Service};
    use serde::Serialize;
    use serde_json::{json, Value};
    use uuid::Uuid;

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Client {
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
    pub(crate) struct PongMessage<'a> {
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
    pub(crate) struct HeartbeatMessage<'a> {
        ver: &'static str,
        sender: &'a str,
        cpu: f32,
    }

    impl<'a> HeartbeatMessage<'a> {
        pub(crate) fn new(sender: &'a str, cpu: f32) -> Self {
            Self {
                ver: "4",
                sender,
                cpu,
            }
        }
    }

    #[derive(Serialize)]
    pub(crate) struct DisconnectMessage<'a> {
        ver: &'static str,
        sender: &'a str,
    }

    impl<'a> DisconnectMessage<'a> {
        pub(crate) fn new(sender: &'a str) -> Self {
            Self { ver: "4", sender }
        }
    }

    #[derive(Serialize)]
    pub(crate) struct DiscoverMessage<'a> {
        ver: &'static str,
        sender: &'a str,
    }

    impl<'a> DiscoverMessage<'a> {
        pub(crate) fn new(sender: &'a str) -> Self {
            Self { ver: "4", sender }
        }
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct InfoMessage<'a> {
        ver: &'static str,
        sender: &'a str,

        #[serde(rename = "instanceID")]
        instance_id: &'a str,
        services: &'a [Service],
        ip_list: &'a [String],
        hostname: &'a str,
        client: Client,

        config: HashMap<String, String>,
        metadata: HashMap<String, String>,
    }

    impl<'a> InfoMessage<'a> {
        pub(crate) fn new(config: &'a Config, services: &'a [Service]) -> Self {
            Self {
                ver: "4",
                sender: &config.node_id,

                instance_id: &config.instance_id,
                services,
                ip_list: &config.ip_list,
                hostname: &config.hostname,
                client: Client::new(),

                config: HashMap::new(),
                metadata: HashMap::new(),
            }
        }
    }

    #[derive(Serialize, Debug)]
    pub(crate) struct EventMessage<'a> {
        pub(crate) id: String,
        pub(crate) sender: &'a str,
        pub(crate) ver: &'static str,

        pub(crate) event: &'a str,

        #[serde(default)]
        pub(crate) data: Value,

        #[serde(default)]
        pub(crate) meta: Value,
        pub(crate) level: i32,

        #[serde(default)]
        pub(crate) tracing: Option<bool>,

        #[serde(rename = "parentID", default)]
        pub(crate) parent_id: &'a Option<String>,

        #[serde(rename = "requestID", default)]
        pub(crate) request_id: &'a Option<String>,

        #[serde(rename = "caller", default)]
        pub(crate) caller: &'a Option<String>,

        #[serde(default)]
        pub(crate) stream: Option<bool>,

        #[serde(default)]
        pub(crate) seq: Option<i32>,

        #[serde(default)]
        pub(crate) groups: Option<Vec<String>>,

        #[serde(default)]
        pub(crate) broadcast: Option<bool>,
    }

    impl<'a> EventMessage<'a> {
        pub(crate) fn new_for_emit(config: &'a Config, event: &'a str, params: Value) -> Self {
            Self {
                event,

                ver: "4",
                id: Uuid::new_v4().to_string(),
                sender: &config.node_id,
                data: params,
                meta: json!({}),
                level: 1,

                tracing: None,
                parent_id: &None,
                request_id: &None,

                caller: &None,
                stream: None,
                seq: None,
                groups: None,
                broadcast: Some(false),
            }
        }

        pub(crate) fn new_for_broadcast(config: &'a Config, event: &'a str, params: Value) -> Self {
            Self {
                broadcast: Some(true),
                ..EventMessage::new_for_emit(config, event, params)
            }
        }
    }

    #[derive(Serialize, Debug)]
    pub(crate) struct ResponseMessage<'a> {
        pub(crate) id: &'a str,
        pub(crate) sender: &'a str,
        pub(crate) ver: &'static str,

        #[serde(default)]
        pub(crate) data: Value,

        #[serde(default)]
        pub(crate) meta: Value,

        #[serde(default)]
        pub(crate) error: Option<crate::channels::messages::MoleculerError>,

        #[serde(default)]
        pub(crate) success: bool,
    }

    impl<'a> ResponseMessage<'a> {
        pub(crate) fn new(config: &'a Config, request_id: &'a str, params: Value) -> Self {
            Self {
                ver: "4",
                id: request_id,
                data: params,
                meta: Value::default(),
                sender: &config.node_id,
                success: true,
                error: None,
            }
        }
    }

    #[derive(Serialize, Debug)]
    pub(crate) struct RequestMessage<'a> {
        pub(crate) id: String,
        pub(crate) sender: &'a str,
        pub(crate) ver: &'static str,

        pub(crate) action: &'a str,

        #[serde(default)]
        pub(crate) params: Value,

        #[serde(default)]
        pub(crate) meta: Value,

        pub(crate) timeout: f32,
        pub(crate) level: i32,

        #[serde(default)]
        pub(crate) tracing: Option<bool>,

        #[serde(rename = "parentID", default)]
        pub(crate) parent_id: Option<&'a str>,

        #[serde(rename = "requestID", default)]
        pub(crate) request_id: String,

        #[serde(rename = "caller", default)]
        pub(crate) caller: Option<&'a str>,

        #[serde(default)]
        pub(crate) stream: Option<bool>,

        #[serde(default)]
        pub(crate) seq: Option<i32>,
    }

    impl<'a> RequestMessage<'a> {
        pub(crate) fn new(config: &'a Config, action_name: &'a str, params: Value) -> Self {
            let id = Uuid::new_v4();

            Self {
                ver: "4",
                sender: &config.node_id,
                id: id.to_string(),

                params,
                action: action_name,

                meta: serde_json::Value::default(),

                timeout: config.request_timeout as f32,
                level: 1,

                tracing: None,
                parent_id: None,

                request_id: id.to_string(),
                caller: None,

                stream: None,
                seq: None,
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub(crate) struct MoleculerError {
    message: String,
    code: i8,
    #[serde(rename = "type")]
    type_: String,
    data: serde_json::Value,
}
