use async_trait::async_trait;

use crate::domain::common::CoreError;
use crate::infrastructure::outbox::OutboxEventRecord;

use crate::infrastructure::outbox::MessageRouter;
use crate::infrastructure::outbox::entities::MessageOutboxEventRouting;

#[async_trait]
pub trait OutboxEventRepository: Send + Sync {
    async fn write_event<TRouter: MessageRouter + Send + Sync>(
        &self,
        event: &OutboxEventRecord<TRouter>,
        routing_key: MessageOutboxEventRouting,
    ) -> Result<(), CoreError>;
}

pub struct MockOutboxEventRepository;

impl MockOutboxEventRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl OutboxEventRepository for MockOutboxEventRepository {
    async fn write_event<TRouter: MessageRouter + Send + Sync>(
        &self,
        _event: &OutboxEventRecord<TRouter>,
        _routing: MessageOutboxEventRouting,
    ) -> Result<(), CoreError> {
        Ok(())
    }
}