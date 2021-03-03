use std::borrow::Cow;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

fn random_string(take: usize) -> String {
    let mut s = String::with_capacity(take);

    for char in random_string_iter(take) {
        s.push(char)
    }

    s
}

fn random_string_iter(take: usize) -> impl Iterator<Item = char> {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(take)
        .map(char::from)
}

pub fn gen_node_id() -> String {
    let random_string_length = 6;

    let hostname = hostname::get()
        .map(|s| Cow::Owned(s.to_string_lossy().to_string()))
        .unwrap_or_else(|_| Cow::Borrowed("unknown_host_name"));

    let pid = std::process::id().to_string();

    let mut node_id =
        String::with_capacity("rust-".len() + hostname.len() + pid.len() + random_string_length);

    node_id.push_str("rust-");
    node_id.push_str(&hostname);
    node_id.push('.');
    node_id.push_str(&pid);
    node_id.push('-');

    random_string_iter(random_string_length).for_each(|char| node_id.push(char));

    node_id
}
