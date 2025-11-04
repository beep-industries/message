use crate::domain::*;
use crate::ports::{MessageRepository, RepoResult};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct MessageService {
    repo: Arc<dyn MessageRepository>,
}

impl MessageService {
    pub fn new(repo: Arc<dyn MessageRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_message(&self, channel: &str, id: Uuid) -> RepoResult<Message> {
        self.repo.get(channel, id).await
    }

    pub async fn list_messages(
        &self,
        channel: &str,
        limit: Option<u32>,
        before: Option<Uuid>,
    ) -> RepoResult<(Vec<Message>, Option<Uuid>)> {
        self.repo.list(channel, limit, before).await
    }

    pub async fn update_message(&self, id: Uuid, update: MessageUpdate) -> RepoResult<Message> {
        self.repo.update(id, update).await
    }

    pub async fn delete_message(&self, id: Uuid) -> RepoResult<()> {
        self.repo.delete(id).await
    }

    pub async fn pin_message(&self, id: Uuid) -> RepoResult<()> {
        self.repo.pin(id).await
    }

    pub async fn list_pins(
        &self,
        channel: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> RepoResult<(Vec<Message>, usize)> {
        self.repo.list_pins(channel, limit, offset).await
    }

    pub async fn search(
        &self,
        channel: &str,
        q: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        in_docs: Option<bool>,
    ) -> RepoResult<(Vec<SearchResult>, usize)> {
        self.repo.search(channel, q, limit, offset, in_docs).await
    }
}
