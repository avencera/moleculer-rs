use std::error::Error;

use moleculer::{
    config::{ConfigBuilder, Transporter},
    service::{Context, Event, EventBuilder, Service},
    ServiceBroker,
};
use serde_json::json;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let config = ConfigBuilder::default()
        .transporter(Transporter::nats("nats://localhost:4222"))
        .build();

    let ask_node_for_answer = EventBuilder::new("askNodeForAnswer")
        .add_callback(ask_node_for_answer)
        .build();

    let greeter_service = Service::new("rustGreeter").add_event(ask_node_for_answer);

    let service_broker = ServiceBroker::new(config).add_service(greeter_service);
    service_broker.start().await;

    Ok(())
}

fn ask_node_for_answer(ctx: Context<Event>) -> Result<(), Box<dyn Error>> {
    tokio::spawn(async move {
        let a = 10;
        let b = 78;

        let response = ctx
            .broker
            .call("greeter.math.add.js", json!({"a": a, "b": b}))
            .await;

        let answer = response.unwrap().to_string();

        println!("The answer to the question {} + {} is {}", a, b, answer);
    });
    Ok(())
}
