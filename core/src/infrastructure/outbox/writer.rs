use mongodb::{
    Collection, Database,
    bson::{DateTime as BsonDateTime, doc, to_bson},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    domain::common::CoreError,
    infrastructure::outbox::event::{MessageRouter, OutboxEventRecord},
};

const OUTBOX_COLLECTION: &str = "outbox_messages";

#[derive(Debug, Serialize)]
struct OutboxDocument {
    #[serde(rename = "_id")]
    id: Uuid,
    exchange_name: String,
    routing_key: String,
    payload: mongodb::bson::Bson,
    status: String,
    created_at: BsonDateTime,
}

pub async fn write_outbox_event<TPayload, TRouter>(
    db: &Database,
    event: &OutboxEventRecord<TPayload, TRouter>,
) -> Result<Uuid, CoreError>
where
    TPayload: Serialize + Send + Sync,
    TRouter: MessageRouter + Send + Sync,
{
    let payload = to_bson(&event.payload)
        .map_err(|e| CoreError::SerializationError { msg: e.to_string() })?;

    let doc = OutboxDocument {
        id: event.id,
        exchange_name: event.router.exchange_name().to_string(),
        routing_key: event.router.routing_key().to_string(),
        payload,
        status: "READY".to_string(),
        created_at: BsonDateTime::now(),
    };

    let collection: Collection<OutboxDocument> = db.collection(OUTBOX_COLLECTION);

    collection
        .insert_one(doc)
        .await
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    Ok(event.id)
}
