use api::app::App;
use api::http::server::ApiError;
use dotenv::dotenv;

use api::config::Config;
use clap::Parser;

use tracing::{info, trace};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    // Initialize tracing subscriber with environment filter
    // Use RUST_LOG environment variable to control log level
    // Examples: RUST_LOG=debug, RUST_LOG=api=debug, RUST_LOG=api::http::server::middleware::auth=trace
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
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
