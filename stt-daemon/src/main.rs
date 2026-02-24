mod config;
mod provider;
mod server;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::load()?;
    info!(
        "Loaded config: provider={}, model={}",
        config.engine.provider, config.engine.model_path
    );

    let provider = provider::from_config(&config.engine)?;
    info!("Model loaded and ready");

    server::run(config.server, provider).await
}
