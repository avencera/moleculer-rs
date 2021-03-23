# moleculer-rs — work in progress

Inspired and compatible with [Moleculer JS](https://github.com/moleculerjs/moleculer)

You can currently do all the basics of `emit`, `broadcast` and `call`.

However it only works with the `NATS` transporter and `JSON` serializer/deserializer.

## Getting Started

Available on crates: [crates.io/moleculer](https://crates.io/crates/moleculer)

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

## Current Status

### What it does

- Is discoverable by other moleculer clients
- Only NATS transporter
- Only JSON serialization/deserialization
- Can `emit` and `broadcast` events
- Can respond to events from other molecular clients using callbacks (see: [simple event example](https://github.com/primcloud/moleculer-rs/blob/master/examples/simple_event.rs))
- Can create actions, and respond to requests ([#19](https://github.com/primcloud/moleculer-rs/pull/19))
- Can `call` to send request and wait for response ([#20](https://github.com/primcloud/moleculer-rs/pull/20))

### Big missing pieces:

- Documentation [#16](https://github.com/primcloud/moleculer-rs/issues/16)
- Tests [#17](https://github.com/primcloud/moleculer-rs/issues/17)
