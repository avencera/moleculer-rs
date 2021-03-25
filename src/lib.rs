/*!
Inspired and compatible with [Moleculer JS](https://github.com/moleculerjs/moleculer)

You can currently do all the basics of `emit`, `broadcast` and `call`.

However it only works with the `NATS` transporter and `JSON` serializer/deserializer.

## Getting Started

Simple example showing how to receive an event, and responding to a request, for more check the
[examples folder](https://github.com/primcloud/moleculer-rs/tree/master/examples).

```rust
use std::error::Error;
use serde::Deserialize;

use moleculer::{
    config::{ConfigBuilder, Transporter},
    service::{EventBuilder, Service, ActionBuilder},
    EventContext, ActionContext, ServiceBroker,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    // build config
    let config = ConfigBuilder::default().transporter(Transporter::nats("nats://localhost:4222"))
    .build();

    // create the first event
    let print_hi = EventBuilder::new("printHi").add_callback(print_hi).build();

    // create the second event
    let print_name = EventBuilder::new("printName")
        .add_callback(print_name)
        .build();

    // create math action
    let math_action = ActionBuilder::new("mathAdd").add_callback(math_add).build();

    // create a service with events and actions
    let greeter_service = Service::new("rustGreeter")
        .add_event(print_hi)
        .add_event(print_name)
        .add_action(math_action);

    // create service broker with service
    let service_broker = ServiceBroker::new(config).add_service(greeter_service);

    // start the service broker
    service_broker.start().await;

    Ok(())
}


// callback for first event, will be called whenever "printHi" event is received
fn print_hi(_ctx: EventContext) -> Result<(), Box<dyn Error>> {
    println!("Hello from Rust");
    Ok(())
}

// callback for second event, will be called whenever "printName" event is received
fn print_name(ctx: EventContext) -> Result<(), Box<dyn Error>> {
    let msg: PrintNameMessage = serde_json::from_value(ctx.params)?;

    println!("Hello to: {} from Rust", msg.name);

    Ok(())
}

// callback for math action
fn math_add(ctx: ActionContext) -> Result<(), Box<dyn Error>> {
    // get message decode using serde
    let msg: ActionMessage = serde_json::from_value(ctx.params.clone())?;
    let answer = msg.a + msg.b;

    // serialize reply using serde and send reply
    let _ = ctx.reply(answer.into());

    Ok(())
}

#[derive(Deserialize)]
struct PrintNameMessage {
    name: String,
}

#[derive(Deserialize)]
struct ActionMessage {
    a: i32,
    b: i32,
}
```
*/

mod data_structures;
mod util;

pub mod config;
pub mod service;

mod broker;
mod channels;
mod nats;

use act_zero::runtimes::tokio::spawn_actor;
use act_zero::*;
use config::Config;
use serde_json::Value;
use service::Service;
use thiserror::Error;
use tokio::sync::oneshot::{self, error};

#[doc(hidden)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("Timeout reached waiting for response")]
    ReceiveError(#[from] error::RecvError),

    #[error("Unknown error")]
    UnknownError,
}

#[allow(dead_code)]
pub(crate) mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// The struct used to interact with moleculer.
/// Use [`emit()`][Self::emit()], [`broadcast()`][Self::broadcast()] and [`call()`][Self::call()] functions.
/// ```rust, ignore
/// // emit an event
/// broker.emit("printHi", json!{{}});
///
/// // broadcast an event
/// broker.broadcast("printHi", json!{{}});
///
/// // call an action
/// let result = broker.call("math.add", json!{"a": 1, "b": c}).await?;
/// ```
#[derive(Clone)]
pub struct ServiceBroker {
    addr: Addr<broker::ServiceBroker>,
}

/// An alias to [service::Context\<service::Event>][service::Context].
/// In all contexts [`emit()`][service::Context::emit()], [`broadcast()`][service::Context::broadcast()]
/// and [`call()`][service::Context::call()] are available.
pub type EventContext = service::Context<service::Event>;

/// An alias to [service::Context\<service::Action>][service::Context].
/// Send a response to a request using [`reply()`][service::Context::reply()].
pub type ActionContext = service::Context<service::Action>;

impl ServiceBroker {
    /// Create new service broker, takes [Config] struct.
    pub fn new(config: Config) -> ServiceBroker {
        ServiceBroker {
            addr: spawn_actor(broker::ServiceBroker::new(config)),
        }
    }

    /// Add a service to the service broker.
    pub fn add_service(self, service: Service) -> Self {
        send!(self.addr.add_service(service));
        self
    }

    /// Add all the services to the service broker at once.
    /// Takes a vector of services and replaces any services the broker already had.
    pub fn add_services(self, services: Vec<Service>) -> Self {
        send!(self.addr.add_services(services));
        self
    }

    /// Starts the service, this will run forever until your application exits.
    pub async fn start(self) {
        self.addr.termination().await
    }

    /// Request/Response style call
    /// Call an action directly with params serialized into
    /// [serde_json::Value](https://docs.rs/serde_json/1.0.64/serde_json/value/index.html) and `await` on the result
    pub async fn call<S: Into<String>>(self, action: S, params: Value) -> Result<Value, Error> {
        let (tx, rx) = oneshot::channel();

        send!(self.addr.call(action.into(), params, tx));
        let response_value = rx.await?;

        Ok(response_value)
    }

    /// Emits a balanced event to one of the nodes.
    pub fn emit<S: Into<String>>(&self, event: S, params: Value) {
        send!(self.addr.emit(event.into(), params))
    }

    /// Emits an event to all the nodes that can handle the event.
    pub fn broadcast<S: Into<String>>(&self, event: S, params: Value) {
        send!(self.addr.broadcast(event.into(), params))
    }
}

#[doc(hidden)]
impl From<Addr<broker::ServiceBroker>> for ServiceBroker {
    fn from(addr: Addr<broker::ServiceBroker>) -> Self {
        Self { addr }
    }
}
