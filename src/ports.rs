use crate::domain::*;
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("not found")]
    NotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("not implemented")]
    NotImplemented,
    #[error("other: {0}")]
    Other(String),
}

pub type RepoResult<T> = Result<T, RepositoryError>;

#[async_trait]
pub trait MessageRepository: Send + Sync + 'static {
    async fn get(&self, channel: &str, id: Uuid) -> RepoResult<Message>;

    async fn list(
        &self,
        channel: &str,
        limit: Option<u32>,
        before: Option<Uuid>,
    ) -> RepoResult<(Vec<Message>, Option<Uuid>)>;

    async fn update(&self, id: Uuid, update: MessageUpdate) -> RepoResult<Message>;

    async fn delete(&self, id: Uuid) -> RepoResult<()>;

    async fn pin(&self, id: Uuid) -> RepoResult<()>;

    async fn list_pins(
        &self,
        channel: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> RepoResult<(Vec<Message>, usize)>;

    async fn search(
        &self,
        channel: &str,
        q: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        in_docs: Option<bool>,
    ) -> RepoResult<(Vec<SearchResult>, usize)>;
}
