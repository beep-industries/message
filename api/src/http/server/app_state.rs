use messages_core::{MessagesService, application::MessageRepositories};
use std::sync::Arc;

use crate::http::server::authorization::DynAuthz;

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    pub service: MessagesService,
    pub authz: DynAuthz,
}

impl AppState {
    /// Create a new AppState with the given service and authorization client
    pub fn new(service: MessagesService, authz: DynAuthz) -> Self {
        Self { service, authz }
    }

    /// Shutdown the underlying database pool
    pub async fn shutdown(&self) {
        self.service.shutdown().await
    }
}

impl From<MessageRepositories> for AppState {
    fn from(repositories: MessageRepositories) -> Self {
        // Fallback: create a permissive dummy authz client so code using `From`
        // doesn't break. Most callers should construct AppState::new with a
        // real authz client.
        let service = MessagesService::new(
            repositories.message_repository,
            repositories.health_repository,
            repositories.attachment_repository,
            repositories.outbox_repository,
        );
        let authz = Arc::new(crate::http::server::authorization::DummyAuthz::new());
        AppState { service, authz }
    }
}
