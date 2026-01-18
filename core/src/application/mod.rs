use mongodb::{Client as MongoClient, options::ClientOptions};

use crate::{
    domain::common::{CoreError, services::Service},
    infrastructure::{
        MessageRoutingInfo,
    health::repositories::mongo::MongoHealthRepository,
        message::repositories::mongo::MongoMessageRepository,
    },
};

/// Concrete service type
pub type CommunitiesService = Service<MongoMessageRepository, MongoHealthRepository>;

#[derive(Clone)]
pub struct CommunitiesRepositories {
    pub message_repository: MongoMessageRepository,
    pub health_repository: MongoHealthRepository,
}

#[tracing::instrument(skip(mongo_uri, mongo_db_name))]
pub async fn create_repositories(
    mongo_uri: &str,
    mongo_db_name: &str,
) -> Result<CommunitiesRepositories, CoreError> {
    tracing::info!(db = %mongo_db_name, "creating mongodb client");
    let mongo_options = ClientOptions::parse(mongo_uri)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    let mongo_client = MongoClient::with_options(mongo_options)
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    let mongo_db = mongo_client.database(mongo_db_name);

    let message_repository = MongoMessageRepository::new(&mongo_db);

    let health_repository = MongoHealthRepository::new(&mongo_db);

    tracing::info!("repositories created");

    Ok(CommunitiesRepositories {
        message_repository,
        health_repository,
    })
}

impl From<CommunitiesRepositories> for CommunitiesService {
    fn from(repos: CommunitiesRepositories) -> Self {
        Service::new(repos.message_repository, repos.health_repository)
    }
}

impl CommunitiesRepositories {
    pub async fn shutdown(&self) {
        tracing::info!("closing Mongo DB connection");
        // MongoDB driver shuts down automatically
    }
}

impl CommunitiesService {
    pub async fn shutdown(&self) {
        tracing::info!("closing Mongo DB connection");
        // MongoDB driver shuts down automatically
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
