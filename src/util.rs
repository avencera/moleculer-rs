use std::borrow::Cow;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

fn random_string_iter(take: usize) -> impl Iterator<Item = char> {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(take)
        .map(char::from)
}

pub(crate) fn gen_node_id() -> String {
    let random_string_length = 6;

    let pid = std::process::id().to_string();

    let hostname = hostname();

    let mut node_id = String::with_capacity(hostname.len() + pid.len() + random_string_length);

    node_id.push_str(&hostname);
    node_id.push('.');
    node_id.push_str(&pid);
    node_id.push('-');

    random_string_iter(random_string_length).for_each(|char| node_id.push(char));

    node_id.to_lowercase()
}

pub(crate) fn hostname() -> Cow<'static, str> {
    hostname::get()
        .map(|s| Cow::Owned(s.to_string_lossy().to_string().to_lowercase()))
        .unwrap_or_else(|_| Cow::Borrowed("unknown_host_name"))
}

pub(crate) fn ip_list() -> Vec<String> {
    get_if_addrs::get_if_addrs()
        .unwrap_or_default()
        .iter()
        .map(|interface| interface.addr.ip())
        .filter(|ip| ip.is_ipv4() && !ip.is_loopback())
        .map(|ip| ip.to_string())
        .collect()
}
