use std::error::Error;

use moleculer_rs::{
    config::{ConfigBuilder, Transporter},
    service::{Context, Event, EventBuilder, Service},
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

    let print_hi = EventBuilder::new("printHi").add_callback(print_hi).build();

    let print_name = EventBuilder::new("printName")
        .add_callback(print_name)
        .build();

    let greeter_service = Service::new("rustGreeter")
        .add_event(print_hi)
        .add_event(print_name);

    let service_broker = ServiceBroker::new(config).add_service(greeter_service);
    service_broker.start().await;

    Ok(())
}

fn print_hi(_ctx: Context<Event>) -> Result<(), Box<dyn Error>> {
    println!("Hello from Rust");
    Ok(())
}

fn print_name(ctx: Context<Event>) -> Result<(), Box<dyn Error>> {
    let msg: PrintNameMessage = serde_json::from_value(ctx.params)?;

    println!("Hello to: {} from Rust", msg.name);

    Ok(())
}

#[derive(Deserialize)]
struct PrintNameMessage {
    name: String,
}
