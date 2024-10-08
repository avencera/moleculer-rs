use std::error::Error;

use moleculer::{
    config::{ConfigBuilder, Transporter},
    service::{ActionBuilder, Service},
    ActionContext, ServiceBroker,
};
use serde::Deserialize;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let config = ConfigBuilder::default()
        .transporter(Transporter::nats("nats://localhost:4222"))
        .build();

    let math_action = ActionBuilder::new("mathAdd").add_callback(math_add).build();
    let greeter_service = Service::new("rustMath").add_action(math_action);

    let service_broker = ServiceBroker::new(config).add_service(greeter_service);
    service_broker.start().await;

    Ok(())
}
fn math_add(ctx: ActionContext) -> Result<(), Box<dyn Error>> {
    // get message decode using serde
    let msg: ActionMessage = serde_json::from_value(ctx.params.clone())?;
    let answer = msg.a + msg.b;

    // serialize reply using serde and send
    ctx.reply(answer.into());

    Ok(())
}

#[derive(Deserialize)]
struct ActionMessage {
    a: i32,
    b: i32,
}
