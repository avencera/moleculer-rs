use moleculer_rs::config::{Config, Transporter};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let config = Config {
        transporter: Transporter::Nats("nats://localhost:4222".to_string()),
        ..Config::default()
    };

    moleculer_rs::start(config).await?;

    Ok(())
}
