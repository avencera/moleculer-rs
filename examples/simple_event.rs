use std::error::Error;

use moleculer_rs::{
    config::{ConfigBuilder, Transporter},
    service::{EventBuilder, EventContext, Service},
    ServiceBroker,
};
use serde::Deserialize;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let config = ConfigBuilder {
        transporter: Transporter::nats("nats://localhost:4222"),
        ..ConfigBuilder::default()
    }
    .build();

    let print_hi = EventBuilder::new("rustGreeter.printHi")
        .add_callback(print_hi)
        .build();

    let print_name = EventBuilder::new("rustGreeter.printName")
        .add_callback(print_name)
        .build();

    let greeter_service = Service::new("rustGreeter")
        .add_event(print_hi)
        .add_event(print_name);

    let services = vec![greeter_service];

    let service_broker = ServiceBroker::new(config, services);
    service_broker.start().await;

    Ok(())
}

fn print_hi(_ctx: EventContext) -> Result<(), Box<dyn Error>> {
    println!("Hello from Rust");
    Ok(())
}

fn print_name(ctx: EventContext) -> Result<(), Box<dyn Error>> {
    let msg: PrintNameMessage = serde_json::from_value(ctx.data)?;

    println!("Hello to: {} from Rust", msg.name);

    Ok(())
}

#[derive(Deserialize)]
struct PrintNameMessage {
    name: String,
}
