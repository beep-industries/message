use async_trait::async_trait;
use mongodb::Database;

use crate::{domain::{common::CoreError, outbox::ports::OutboxEventRepository}, infrastructure::outbox::{MessageRouter, OutboxEventRecord, entities::MessageOutboxEventRouting}, write_outbox_event};

#[derive(Clone)]
pub struct MongoOutboxEventRepository {
    db: Database,
}

impl MongoOutboxEventRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl OutboxEventRepository for MongoOutboxEventRepository {
    async fn write_event<TRouter: MessageRouter + Send + Sync>(
        &self,
        event: &OutboxEventRecord<TRouter>,
        routing: MessageOutboxEventRouting
    ) -> Result<(), CoreError> {
        write_outbox_event(&self.db, routing.get_exchange(), routing.to_routing_key(), event)
            .await
            .map(|_| ())
    }
}
