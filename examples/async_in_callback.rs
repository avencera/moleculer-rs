use std::{error::Error, time::Duration};

use moleculer_rs::{
    config::{ConfigBuilder, Transporter},
    service::{Context, EventBuilder, Service},
    ServiceBroker,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let config = ConfigBuilder {
        transporter: Transporter::nats("nats://localhost:4222"),
        ..ConfigBuilder::default()
    }
    .build();

    let print_async = EventBuilder::new("printAsync")
        .add_callback(print_async)
        .build();

    let print_normal = EventBuilder::new("printNormal")
        .add_callback(print_normal)
        .build();

    let greeter_service = Service::new("asyncGreeter")
        .add_event(print_normal)
        .add_event(print_async);

    let service_broker = ServiceBroker::new(config).add_service(greeter_service);
    service_broker.start().await;

    Ok(())
}

fn print_async(_ctx: Context) -> Result<(), Box<dyn Error>> {
    println!("Hello from sync");
    futures_lite::future::block_on(async { hello_from_async().await });
    println!("Bye from sync");
    Ok(())
}

fn print_normal(_ctx: Context) -> Result<(), Box<dyn Error>> {
    println!("Hello from normal");
    Ok(())
}

async fn hello_from_async() {
    sleep(Duration::from_secs(5)).await;
    println!("Hello from async")
}
