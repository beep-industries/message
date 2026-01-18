use api::app::App;
use api::http::server::ApiError;
use dotenv::dotenv;

use api::config::Config;
use clap::Parser;

use tracing::{info, trace};

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    // Initialize tracing subscriber with environment filter and a default level
    // Initialize a basic tracing subscriber. Using a simple default level (INFO).
    // For more advanced filtering (RUST_LOG) we can switch to EnvFilter when desired.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    // Load environment variables from .env file
    trace!("loading env vars and config file...");
    dotenv().ok();

    let mut config: Config = Config::parse();
    config.load_routing().map_err(|e| ApiError::StartupError {
        msg: format!("Failed to load routing config: {}", e),
    })?;
    trace!("...config and env vars loaded.");
    let app = App::new(config).await?;
    info!("Starting the service");
    app.start().await?;
    Ok(())
}
