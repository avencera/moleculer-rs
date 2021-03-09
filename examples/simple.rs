use moleculer_rs::{
    config::{ConfigBuilder, Transporter},
    service::Service,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let config = ConfigBuilder {
        transporter: Transporter::Nats("nats://localhost:4222".to_string()),
        ..ConfigBuilder::default()
    }
    .build();

    let greeter_service = Service::new("greeter");

    let services = vec![greeter_service];

    moleculer_rs::start(config, services).await?;

    Ok(())
}
