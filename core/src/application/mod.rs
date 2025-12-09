use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    domain::common::{CoreError, services::Service},
    infrastructure::{
        MessageRoutingInfo, health::repositories::postgres::PostgresHealthRepository,
        message::repositories::postgres::PostgresMessageRepository,
    },
};

/// Concrete service type with PostgreSQL repositories (using MockMemberRepository until issue #68 is implemented)
pub type CommunitiesService = Service<PostgresMessageRepository, PostgresHealthRepository>;

#[derive(Clone)]
pub struct CommunitiesRepositories {
    pool: PgPool,
    pub message_repository: PostgresMessageRepository,
    pub health_repository: PostgresHealthRepository,
}

pub async fn create_repositories(
    pg_connection_options: PgConnectOptions,
    message_routing_infos: MessageRoutingInfos,
) -> Result<CommunitiesRepositories, CoreError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_connection_options)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;
    let message_repository = PostgresMessageRepository::new(
        pool.clone(),
        message_routing_infos.delete_message,
        message_routing_infos.create_message,
    );
    let health_repository = PostgresHealthRepository::new(pool.clone());
    Ok(CommunitiesRepositories {
        pool,
        message_repository,
        health_repository,
    })
}

impl Into<CommunitiesService> for CommunitiesRepositories {
    fn into(self) -> CommunitiesService {
        Service::new(self.message_repository, self.health_repository)
    }
}

impl CommunitiesRepositories {
    pub async fn shutdown_pool(&self) {
        let _ = &self.pool.close().await;
    }
}

impl CommunitiesService {
    pub async fn shutdown_pool(&self) {
        self.message_repository.pool.close().await;
    }
}

/// Configuration for message routing information across different event types.
///
/// This struct holds the routing configuration for various outbox events
/// that need to be published to a message broker. Each field represents
/// the routing information (exchange name and routing key) for a specific
/// type of domain event.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct MessageRoutingInfos {
    /// Routing information for message creation events
    pub create_message: MessageRoutingInfo,
    /// Routing information for message deletion events
    pub delete_message: MessageRoutingInfo,
}
