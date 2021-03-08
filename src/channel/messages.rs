pub mod incoming {
    use serde::Deserialize;

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
}

pub mod outgoing {
    use std::time::SystemTime;

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
}
