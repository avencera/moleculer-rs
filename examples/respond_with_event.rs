use std::error::Error;

use moleculer::{
    config::{ConfigBuilder, Transporter},
    service::{Context, Event, EventBuilder, Service},
    ServiceBroker,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let config = ConfigBuilder::default()
        .transporter(Transporter::nats("nats://localhost:4222"))
        .build();

    let emit_hi = EventBuilder::new("emitHi").add_callback(emit_hi).build();

    let broadcast_name = EventBuilder::new("broadcastName")
        .add_callback(broadcast_name)
        .build();

    let greeter_service = Service::new("rustGreeter")
        .add_event(emit_hi)
        .add_event(broadcast_name);

    let service_broker = ServiceBroker::new(config).add_service(greeter_service);
    service_broker.start().await;

    Ok(())
}

fn emit_hi(ctx: Context<Event>) -> Result<(), Box<dyn Error>> {
    println!("Received emitHi in rust");
    ctx.broker.emit("test", serde_json::json!({}));

    Ok(())
}

fn broadcast_name(ctx: Context<Event>) -> Result<(), Box<dyn Error>> {
    let msg: PrintNameMessage = serde_json::from_value(ctx.params)?;
    println!("Received broadcastName in rust");
    ctx.broker
        .broadcast("testWithParam", serde_json::to_value(&msg)?);

    Ok(())
}

#[derive(Deserialize, Serialize)]
struct PrintNameMessage {
    name: String,
}
