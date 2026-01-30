use mongodb::{
    Collection, Database,
    bson::{DateTime as BsonDateTime, doc},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    domain::common::CoreError,
    infrastructure::{outbox::{event::{MessageRouter, OutboxEventRecord}}},
};

const OUTBOX_COLLECTION: &str = "outbox_messages";

use mongodb::bson::Binary;
#[derive(Debug, Serialize)]
struct OutboxDocument {
    #[serde(rename = "_id")]
    id: Uuid,
    exchange_name: String,
    routing_key: String,
    payload: Binary, // store as BSON binary
    status: String,
    created_at: BsonDateTime,
}

pub async fn write_outbox_event<TRouter>(
    db: &Database,
    exchange: &str,
    routing_key: &str,
    event: &OutboxEventRecord<TRouter>,
) -> Result<Uuid, CoreError>
where
    TRouter: MessageRouter + Send + Sync,
{
    let doc = OutboxDocument {
        id: event.id,
        exchange_name: exchange.to_string(),
        routing_key: routing_key.to_string(),
        payload: Binary {
            subtype: mongodb::bson::spec::BinarySubtype::Generic,
            bytes: event.payload.clone(),
        },
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
