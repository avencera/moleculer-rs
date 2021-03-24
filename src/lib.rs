/*!
Inspired and compatible with [Moleculer JS](https://github.com/moleculerjs/moleculer)

You can currently do all the basics of `emit`, `broadcast` and `call`.

However it only works with the `NATS` transporter and `JSON` serializer/deserializer.

## Getting Started

Simple example showing how to receive an event, for more check the [examples folder](https://github.com/primcloud/moleculer-rs/tree/master/examples)

```rust
#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    // build config
    let config = ConfigBuilder {
        transporter: Transporter::nats("nats://localhost:4222"),
        ..ConfigBuilder::default()
    }
    .build();

    // create the first event
    let print_hi = EventBuilder::new("printHi").add_callback(print_hi).build();

    // create the second event
    let print_name = EventBuilder::new("printName")
        .add_callback(print_name)
        .build();

    // create a service with events
    let greeter_service = Service::new("rustGreeter")
        .add_event(print_hi)
        .add_event(print_name);

    // create service broker with service
    let service_broker = ServiceBroker::new(config).add_service(greeter_service);

    // start the service broker
    service_broker.start().await;

    Ok(())
}

// callback for first event, will be called whenever "printHi" event is received
fn print_hi(_ctx: Context<Event>) -> Result<(), Box<dyn Error>> {
    println!("Hello from Rust");
    Ok(())
}

// callback for second event, will be called whenever "printName" event is received
fn print_name(ctx: Context<Event>) -> Result<(), Box<dyn Error>> {
    let msg: PrintNameMessage = serde_json::from_value(ctx.params)?;

    println!("Hello to: {} from Rust", msg.name);

    Ok(())
}

#[derive(Deserialize)]
struct PrintNameMessage {
    name: String,
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

#[derive(Clone)]
pub struct ServiceBroker {
    addr: Addr<broker::ServiceBroker>,
}

impl ServiceBroker {
    /// Create new service broker, takes [Config] struct
    pub fn new(config: Config) -> ServiceBroker {
        ServiceBroker {
            addr: spawn_actor(broker::ServiceBroker::new(config)),
        }
    }

    /// Add a service to the service broker
    pub fn add_service(self, service: Service) -> Self {
        send!(self.addr.add_service(service));
        self
    }

    /// Add all the services to the service broker at once
    /// Takes a vector of services and replaces any services the broker already had
    pub fn add_services(self, services: Vec<Service>) -> Self {
        send!(self.addr.add_services(services));
        self
    }

    /// Starts the service, this will run forever until your application closes
    pub async fn start(self) {
        self.addr.termination().await
    }

    /// Request/Response style call
    /// Call an action directly with params serialized into
    /// [serde_json::Value](https://docs.rs/serde_json/1.0.64/serde_json/value/index.html) and `await` on the result
    /// ```rust
    ///  let result = broker.call("math.add", json!{"a": 1, "b": c}).await?;
    /// ```
    pub async fn call<S: Into<String>>(self, action: S, params: Value) -> Result<Value, Error> {
        let (tx, rx) = oneshot::channel();

        send!(self.addr.call(action.into(), params, tx));
        let response_value = rx.await?;

        Ok(response_value)
    }

    /// Emits a balanced event to one of the nodes
    pub fn emit<S: Into<String>>(&self, event: S, params: Value) {
        send!(self.addr.emit(event.into(), params))
    }

    /// Emits an event to all the nodes that can handle the event
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
