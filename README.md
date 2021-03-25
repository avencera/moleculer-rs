# moleculer-rs

![Build Status](https://github.com/primcloud/moleculer-rs/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/moleculer.svg)](https://crates.io/crates/moleculer)
[![Documentation](https://docs.rs/moleculer/badge.svg)](https://docs.rs/moleculer)
[![Rust 1.47+](https://img.shields.io/badge/rust-1.47+-orange.svg)](https://www.rust-lang.org)
![](https://img.shields.io/badge/unsafe-forbidden-brightgreen.svg)

Inspired and compatible with [Moleculer JS](https://github.com/moleculerjs/moleculer)

You can currently do all the basics of `emit`, `broadcast` and `call`.

However it only works with the `NATS` transporter and `JSON` serializer/deserializer.

## Getting Started

Available on crates: [crates.io/moleculer](https://crates.io/crates/moleculer)

Documentation available at: [docs.rs/moleculer](https://docs.rs/moleculer/)

```toml
moleculer = "0.3.3"
```

Simple example showing how to receive an event, and responding to a request, for more check the [examples folder](https://github.com/primcloud/moleculer-rs/tree/master/examples)

```rust
use std::error::Error;
use moleculer::{
    config::{ConfigBuilder, Transporter},
    service::{Context, Event, EventBuilder, Service},
    ServiceBroker,
};
use serde::Deserialize;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    // build config
    let config = ConfigBuilder::default().transporter("nats://localhost:4222")
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

// callback for math action
fn math_add(ctx: Context<Action>) -> Result<(), Box<dyn Error>> {
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

## Current Status

Usable but missing some options

### What it does

- Is discoverable by other moleculer clients
- Only NATS transporter
- Only JSON serialization/deserialization
- Can `emit` and `broadcast` events
- Can `call` to send request and wait for response ([#20](https://github.com/primcloud/moleculer-rs/pull/20))
- Can respond to events from other molecular clients using callbacks (see: [simple event example](https://github.com/primcloud/moleculer-rs/blob/master/examples/simple_event.rs))
- Can create actions, and respond to requests ([#19](https://github.com/primcloud/moleculer-rs/pull/19))

### What its missing:

- Tests [#17](https://github.com/primcloud/moleculer-rs/issues/17)
- Better error handling on actions [#24](https://github.com/primcloud/moleculer-rs/issues/24)
- Support for different serializer/deserializer [#23](https://github.com/primcloud/moleculer-rs/issues/23)
- Support for `Bulkhead`, `CircuitBreaker` and `RetryPolicy`
- Support for tracing
- Support for different transporters other than NATS
- Support for different loggers
