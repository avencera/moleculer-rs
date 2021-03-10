use std::error::Error;

use moleculer_rs::{
    config::{ConfigBuilder, Transporter},
    service::{Context, EventBuilder, Service},
    ServiceBroker,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let config = ConfigBuilder {
        transporter: Transporter::nats("nats://localhost:4222"),
        ..ConfigBuilder::default()
    }
    .build();

    let mut service_broker = ServiceBroker::new(config);

    let reply_with_name = EventBuilder::new("replyWithName")
        .add_callback(reply_with_name)
        .build();

    let greeter_service = Service::new("rustGreeter").add_event(reply_with_name);

    service_broker = service_broker.add_service(greeter_service);
    service_broker.start().await;

    Ok(())
}

fn reply_with_name(ctx: Context) -> Result<(), Box<dyn Error>> {
    let msg: PrintNameMessage = serde_json::from_value(ctx.params.clone())?;

    ctx.emit("testWithParam", serde_json::to_vec(&msg)?);

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct PrintNameMessage {
    name: String,
}
