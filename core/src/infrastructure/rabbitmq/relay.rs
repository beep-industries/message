use futures::TryStreamExt;
use mongodb::{
    Collection, Database,
    bson::{Bson, Document, doc},
    options::FindOptions,
};
use std::sync::Arc;
use tokio::time::{Duration, interval};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{domain::common::CoreError, infrastructure::rabbitmq::publisher::RabbitMqPublisher};

/// Service that relays messages from the outbox to RabbitMQ
pub struct OutboxRelayService {
    db: Database,
    publisher: Arc<RabbitMqPublisher>,
    poll_interval: Duration,
}

impl OutboxRelayService {
    /// Create a new outbox relay service
    pub fn new(db: Database, publisher: Arc<RabbitMqPublisher>) -> Self {
        Self {
            db,
            publisher,
            poll_interval: Duration::from_secs(1),
        }
    }

    /// Start the relay service (long-running task)
    pub async fn start(&self) {
        info!("Starting outbox relay service");
        let mut ticker = interval(self.poll_interval);

        loop {
            ticker.tick().await;

            if let Err(e) = self.process_pending_messages().await {
                error!("Error processing outbox messages: {}", e);
            }
        }
    }

    /// Process all pending messages in the outbox
    async fn process_pending_messages(&self) -> Result<(), CoreError> {
        let collection: Collection<Document> = self.db.collection("outbox_messages");

        // Find all READY messages
        let filter = doc! { "status": "READY" };
        let options = FindOptions::builder()
            .sort(doc! { "created_at": 1 })
            .limit(100)
            .build();

        let mut cursor = collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to query outbox: {}", e),
            })?;

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| CoreError::DatabaseError {
                msg: format!("Failed to read outbox document: {}", e),
            })?
        {
            if let Err(e) = self.process_single_message(&collection, doc).await {
                error!("Failed to process outbox message: {}", e);
            }
        }

        Ok(())
    }

    /// Process a single outbox message
    async fn process_single_message(
        &self,
        collection: &Collection<Document>,
        doc: Document,
    ) -> Result<(), CoreError> {
        // Extract _id which is stored as UUID (Binary in MongoDB)
        let id_bson = doc.get("_id").ok_or_else(|| CoreError::DatabaseError {
            msg: "Missing _id in outbox document".to_string(),
        })?;

        // Convert BSON to UUID - handle both Binary and String representations
        let id = match id_bson {
            Bson::Binary(bin) => {
                Uuid::from_slice(&bin.bytes).map_err(|e| CoreError::DatabaseError {
                    msg: format!("Invalid UUID in _id: {}", e),
                })?
            }
            Bson::String(s) => Uuid::parse_str(s).map_err(|e| CoreError::DatabaseError {
                msg: format!("Invalid UUID string in _id: {}", e),
            })?,
            _ => {
                return Err(CoreError::DatabaseError {
                    msg: format!("Unexpected _id type: {:?}", id_bson),
                });
            }
        };

        let exchange_name = doc
            .get("exchange_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::DatabaseError {
                msg: format!("Missing exchange_name in outbox document {}", id),
            })?;

        let routing_key = doc
            .get("routing_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::DatabaseError {
                msg: format!("Missing routing_key in outbox document {}", id),
            })?;

        let payload = doc.get("payload").ok_or_else(|| CoreError::DatabaseError {
            msg: format!("Missing payload in outbox document {}", id),
        })?;

        // Extract protobuf bytes from BSON binary
        let payload_bytes = match payload {
            Bson::Binary(bin) => bin.bytes.clone(),
            _ => {
                // Prevent endless retries for legacy/non-binary payloads
                let update = doc! {
                    "$set": {
                        "status": "FAILED",
                        "failed_at": mongodb::bson::DateTime::now(),
                    }
                };
                let _ = collection.update_one(doc! { "_id": id_bson }, update).await;
                return Err(CoreError::SerializationError {
                    msg: format!("Expected binary payload in outbox document {}", id),
                });
            }
        };

        // Ensure exchange exists
        if let Err(e) = self.publisher.declare_exchange(exchange_name).await {
            warn!("Failed to declare exchange {}: {}", exchange_name, e);
        }

        // Publish to RabbitMQ
        match self
            .publisher
            .publish(exchange_name, routing_key, payload_bytes)
            .await
        {
            Ok(_) => {
                // Mark as SENT - use the original _id value for the query
                let update = doc! {
                    "$set": {
                        "status": "SENT",
                    }
                };

                collection
                    .update_one(doc! { "_id": id_bson }, update)
                    .await
                    .map_err(|e| CoreError::DatabaseError {
                        msg: format!("Failed to update outbox status: {}", e),
                    })?;

                info!("Successfully published outbox message {}", id);
            }
            Err(e) => {
                // Mark as FAILED - use the original _id value for the query
                let update = doc! {
                    "$set": {
                        "status": "FAILED",
                        "failed_at": mongodb::bson::DateTime::now(),
                    }
                };

                collection
                    .update_one(doc! { "_id": id_bson }, update)
                    .await
                    .map_err(|e| CoreError::DatabaseError {
                        msg: format!("Failed to update outbox status to FAILED: {}", e),
                    })?;

                error!("Failed to publish outbox message {}: {}", id, e);
            }
        }

        Ok(())
    }
}
