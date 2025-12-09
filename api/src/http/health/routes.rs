use axum::{Router, routing::get};

use crate::http::{health::health_check, message::AppState};

pub fn health_routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}
