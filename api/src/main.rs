use api::app::App;
use api::http::server::ApiError;

use api::config::Config;
use clap::Parser;

use tracing::{info, trace};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    // Initialize tracing subscriber with JSON output for Loki
    // Use RUST_LOG environment variable to control log level
    // Examples: RUST_LOG=debug, RUST_LOG=api=debug, RUST_LOG=api::http::server::middleware::auth=trace
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .init();

    // Load .env file as fallback (only sets variables that aren't already in the environment)
    // System environment variables always take priority
    if let Ok(path) = dotenvy::dotenv() {
        info!("Loaded .env file from: {:?}", path);
    } else {
        info!("No .env file found, using system environment variables");
    }

    let config: Config = Config::parse();
    trace!("...config and env vars loaded.");
    let app = App::new(config).await?;
    info!("Starting the service");
    app.start().await?;
    Ok(())
}
