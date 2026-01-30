use mongodb::{Client as MongoClient, options::ClientOptions};

use crate::{
    domain::common::{CoreError, services::Service},
    infrastructure::{
        attachments::repositories::reqwest::ReqwestAttachmentRepository,
        health::repositories::mongo::MongoHealthRepository,
        message::repositories::mongo::MongoMessageRepository,
        outbox::mongo::MongoOutboxEventRepository,
    },
};

/// Concrete service type
pub type MessagesService = Service<
    MongoMessageRepository,
    MongoHealthRepository,
    ReqwestAttachmentRepository,
    MongoOutboxEventRepository,
>;

#[derive(Clone)]
pub struct MessageRepositories {
    pub message_repository: MongoMessageRepository,
    pub health_repository: MongoHealthRepository,
    pub attachment_repository: ReqwestAttachmentRepository,
    pub outbox_repository: MongoOutboxEventRepository,
}

#[tracing::instrument(skip(mongo_uri, mongo_db_name))]
pub async fn create_repositories(
    mongo_uri: &str,
    mongo_db_name: &str,
    client_url: &String,
) -> Result<MessageRepositories, CoreError> {
    tracing::info!(db = %mongo_db_name, "creating mongodb client");
    let mongo_options = ClientOptions::parse(mongo_uri)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    let mongo_client = MongoClient::with_options(mongo_options)
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    let mongo_db = mongo_client.database(mongo_db_name);

    let message_repository = MongoMessageRepository::new(&mongo_db);

    let health_repository = MongoHealthRepository::new(&mongo_db);

    let attachment_repository = ReqwestAttachmentRepository::new(client_url);

    let outbox_repository = MongoOutboxEventRepository::new(mongo_db.clone());

    tracing::info!("repositories created");

    Ok(MessageRepositories {
        message_repository,
        health_repository,
        attachment_repository,
        outbox_repository,
    })
}

impl From<MessageRepositories> for MessagesService {
    fn from(repos: MessageRepositories) -> Self {
        Service::new(
            repos.message_repository,
            repos.health_repository,
            repos.attachment_repository,
            repos.outbox_repository,
        )
    }
}

impl MessageRepositories {
    pub async fn shutdown(&self) {
        tracing::info!("closing Mongo DB connection");
        // MongoDB driver shuts down automatically
    }
}

impl MessagesService {
    pub async fn shutdown(&self) {
        tracing::info!("closing Mongo DB connection");
        // MongoDB driver shuts down automatically
    }
}
