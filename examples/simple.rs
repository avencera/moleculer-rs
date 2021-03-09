use moleculer_rs::{
    config::{ConfigBuilder, Transporter},
    service::{Context, EventBuilder, Service},
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let config = ConfigBuilder {
        transporter: Transporter::nats("nats://localhost:4222"),
        ..ConfigBuilder::default()
    }
    .build();

    let event = EventBuilder::new("print-hi").add_callback(print_hi).build();
    let greeter_service = Service::new("rustGreeter").add_event(event);

    let services = vec![greeter_service];

    moleculer_rs::start(config, services).await?;

    Ok(())
}

fn print_hi(_ctx: Context) {
    println!("PRINTING HI FROM RUST")
}
