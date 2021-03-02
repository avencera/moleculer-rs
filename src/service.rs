use bytes::Bytes;
use std::collections::HashMap;

pub type ActionCallback = fn(Context) -> Option<Bytes>;
pub type EventCallback = fn(Context) -> ();

pub struct Action {
    name: String,
    callback: ActionCallback,
}

pub struct Event {
    name: String,
    callback: EventCallback,
}

#[derive(Default)]
struct Service {
    name: String,
    version: Option<i32>,
    actions: HashMap<String, ActionCallback>,
    events: HashMap<String, EventCallback>,
}

impl Service {
    pub fn new<S: Into<String>>(name: S) -> Service {
        Service {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn version(mut self, version: i32) -> Service {
        self.version = Some(version);
        self
    }

    pub fn action(mut self, action: Action) -> Service {
        self.actions.insert(action.name, action.callback);
        self
    }

    pub fn event(mut self, event: Event) -> Service {
        self.events.insert(event.name, event.callback);
        self
    }
}

pub enum EventType {
    Emit,
    Broadcast,
}

pub struct Context {
    id: String,
    broker: String,
    nodeID: String,
    action: Option<String>,

    event: Option<String>,
    eventName: Option<String>,
    eventType: Option<EventType>,
    eventGroups: Vec<String>,

    caller: String,
    requestID: String,
    parentID: String,

    params: Bytes,
    meta: Bytes,
    locals: Bytes,

    level: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_func(_ctx: Context) -> Option<Bytes> {
        None
    }

    fn create_service() -> Service {
        let mut service = Service::new("my-service");

        service
    }

    #[test]
    fn creates_service() {
        create_service();
        assert!(true)
    }
}
