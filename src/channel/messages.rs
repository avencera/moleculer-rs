pub mod incoming {
    use std::collections::HashMap;

    use serde::Deserialize;

    use crate::{config::Client, service::Service};

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
}

pub mod outgoing {
    use std::{collections::HashMap, time::SystemTime};

    use crate::{config::Client, service::Service};

    use super::incoming::PingMessage;
    use serde::Serialize;

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

        services: Vec<Service>,
        ip_list: &'a Vec<String>,
        hostname: &'a str,
        client: &'a Client,

        #[serde(rename = "instanceID")]
        instance_id: &'a str,

        config: HashMap<String, String>,
        metadata: HashMap<String, String>,
    }
}
