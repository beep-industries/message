use axum::middleware::from_extractor_with_state;
use communities_core::create_repositories;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

// tracing macros are used fully-qualified to keep imports explicit where needed

use crate::{
    Config,
    http::{
        health::routes::health_routes,
        server::{
            ApiError, AppState, middleware::auth::AuthMiddleware,
            middleware::auth::entities::AuthValidator,
        },
    },
    message_routes,
};

#[derive(OpenApi)]
#[openapi(info(
    title = "Beep communities openapi",
    contact(name = "communities-core@beep.ovh"),
    description = "API documentation for the Communities service",
    version = "0.0.1"
))]
struct ApiDoc;
pub struct App {
    config: Config,
    pub state: AppState,
    pub auth_validator: AuthValidator,
    app_router: axum::Router,
    health_router: axum::Router,
}

impl App {
    #[tracing::instrument(skip(config))]
    pub async fn new(config: Config) -> Result<Self, ApiError> {
        tracing::debug!("Creating repositories...");
        let state: AppState =
            create_repositories(&config.database.mongo_uri, &config.database.mongo_db_name)
                .await
                .map_err(|e| ApiError::StartupError {
                    msg: format!("Failed to create repositories: {}", e),
                })?
                .into();
        let auth_validator = AuthValidator::new(config.clone().jwt.secret_key);
        let (app_router, mut api) = OpenApiRouter::<AppState>::new()
            .merge(message_routes())
            // Add application routes here
            .route_layer(from_extractor_with_state::<AuthMiddleware, AuthValidator>(
                auth_validator.clone(),
            ))
            .split_for_parts();

        // Override API documentation info
        let custom_info = ApiDoc::openapi();
        api.info = custom_info.info;

        let openapi_json = api.to_pretty_json().map_err(|e| ApiError::StartupError {
            msg: format!("Failed to generate OpenAPI spec: {}", e),
        })?;

        let app_router = app_router
            .with_state(state.clone())
            .merge(Scalar::with_url("/scalar", api));
        // Write OpenAPI spec to file in development environment
        if matches!(config.environment, crate::config::Environment::Development) {
            std::fs::write("openapi.json", &openapi_json).map_err(|e| ApiError::StartupError {
                msg: format!("Failed to write OpenAPI spec to file: {}", e),
            })?;
        }

        let health_router = axum::Router::new()
            .merge(health_routes())
            .with_state(state.clone());
        Ok(Self {
            config,
            state,
            auth_validator,
            app_router,
            health_router,
        })
    }

    pub fn app_router(&self) -> axum::Router {
        self.app_router.clone()
    }

    #[tracing::instrument(skip(self))]
    pub async fn start(&self) -> Result<(), ApiError> {
        let health_addr = format!("0.0.0.0:{}", self.config.clone().message.health_port);
        let api_addr = format!("0.0.0.0:{}", self.config.clone().message.api_port);
        // Create TCP listeners for both messages
        let health_listener = tokio::net::TcpListener::bind(&health_addr)
            .await
            .map_err(|_| ApiError::StartupError {
                msg: format!("Failed to bind health message: {}", health_addr),
            })?;
        let api_listener = tokio::net::TcpListener::bind(&api_addr)
            .await
            .map_err(|_| ApiError::StartupError {
                msg: format!("Failed to bind API message: {}", api_addr),
            })?;

    tracing::info!(api_addr = %api_addr, health_addr = %health_addr, "Starting HTTP listeners");
    // Run both listeners concurrently
        tokio::try_join!(
            axum::serve(health_listener, self.health_router.clone()),
            axum::serve(api_listener, self.app_router.clone())
        )
        .expect("Failed to start messages");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn shutdown(&self) {
        self.state.shutdown().await;
    }
}

pub trait AppBuilder {
    fn build(config: Config) -> impl Future<Output = Result<App, ApiError>>;
    fn with_state(self, state: AppState) -> impl Future<Output = Result<App, ApiError>>;
}

impl AppBuilder for App {
    async fn build(config: Config) -> Result<App, ApiError> {
        App::new(config).await
    }

    async fn with_state(mut self, state: AppState) -> Result<App, ApiError> {
        self.state = state;
        Ok(self)
    }
}
