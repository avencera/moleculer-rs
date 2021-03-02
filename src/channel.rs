use std::borrow::Cow;

use crate::config::Config;

pub fn event(config: &Config) -> String {
    format!("{}.EVENT.{}", mol(&config), &config.node_id)
}

pub fn event_balanced(config: &Config, event_name: &str) -> String {
    format!("{}.EVENTB.{}", mol(&config), event_name)
}

pub fn request(config: &Config) -> String {
    format!("{}.REQ.{}", mol(&config), &config.node_id)
}

pub fn request_balanced(config: &Config, action_name: &str) -> String {
    format!("{}.REQB.{}", mol(&config), action_name)
}

pub fn response(config: &Config) -> String {
    format!("{}.REQ.{}", mol(&config), &config.node_id)
}

pub fn discover(config: &Config) -> String {
    format!("{}.DISCOVER", mol(&config))
}

pub fn discover_targeted(config: &Config) -> String {
    format!("{}.DISCOVER.{}", mol(&config), &config.node_id)
}

pub fn info(config: &Config) -> String {
    format!("{}.INFO", mol(&config))
}

pub fn info_targeted(config: &Config) -> String {
    format!("{}.INFO.{}", mol(&config), &config.node_id)
}

pub fn heartbeat(config: &Config) -> String {
    format!("{}.HEARTBEAT", mol(&config))
}

pub fn ping(config: &Config) -> String {
    format!("{}.PING", mol(&config))
}

pub fn ping_targeted(config: &Config) -> String {
    format!("{}.PING.{}", mol(&config), &config.node_id)
}

pub fn pong(config: &Config) -> String {
    format!("{}.PONG.{}", mol(&config), &config.node_id)
}

pub fn disconnect(config: &Config) -> String {
    format!("{}.DISCONNECT", mol(&config))
}

fn mol(config: &Config) -> Cow<str> {
    if config.namespace == "" {
        Cow::Borrowed("MOL")
    } else {
        Cow::Owned(format!("MOL-{}", &config.namespace))
    }
}
