use api::app::App;
use api::http::server::ApiError;
use dotenv::dotenv;

use api::config::Config;
use clap::Parser;

use tracing::{info, trace};

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    // Initialize logger
    tracing_subscriber::fmt::init();

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
