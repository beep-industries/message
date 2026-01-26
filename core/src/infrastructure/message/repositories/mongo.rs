use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    Collection, Database,
    bson::Document,
    bson::{Bson, doc},
    options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument},
};

use mongodb::bson::Binary;
use mongodb::bson::spec::BinarySubtype;

use crate::{
    domain::{
        common::{CoreError, GetPaginated, TotalPaginatedElements},
        message::{
            entities::{InsertMessageInput, Message, MessageId, UpdateMessageInput},
            events::{
                create_message_event_from_domain,
                delete_message_event_from_domain, 
                event_to_bytes, update_message_event_from_domain,
            },
            ports::MessageRepository,
        },
    },
    infrastructure::{MessageRoutingInfo, outbox::OutboxEventRecord, write_outbox_event},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct MongoMessageRepository {
    collection: Collection<Message>,
    pub db: Database,
    routing_info: MessageRoutingInfo,
}

impl MongoMessageRepository {
    pub fn new(db: &Database, routing_info: MessageRoutingInfo) -> Self {
        Self {
            collection: db.collection::<Message>("messages"),
            db: db.clone(),
            routing_info,
        }
    }

    fn pagination_options(pagination: &GetPaginated) -> FindOptions {
        let limit = pagination.limit.min(50) as i64;
        let page = pagination.page.max(1); // Ensure page is at least 1
        let skip = ((page - 1) * pagination.limit) as u64;

        FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(skip)
            .limit(limit)
            .build()
    }
}

#[async_trait::async_trait]
impl MessageRepository for MongoMessageRepository {
    async fn insert(&self, input: InsertMessageInput) -> Result<Message, CoreError> {
        let now = Utc::now();

        let message = Message {
            id: input.id,
            channel_id: input.channel_id,
            author_id: input.author_id,
            content: input.content,
            reply_to_message_id: input.reply_to_message_id,
            attachments: input.attachments,
            is_pinned: false,
            created_at: now,
            updated_at: None,
        };

        // Serialize the message to a BSON document so we can ensure `created_at` is stored as a BSON datetime
        let bson = mongodb::bson::to_bson(&message)
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        if let Bson::Document(mut doc) = bson {
            // convert uuid fields to binary representation so deserialization to `Message` (which
            // expects UUID bytes) works consistently
            doc.insert(
                "_id",
                Bson::Binary(Binary {
                    subtype: BinarySubtype::Generic,
                    bytes: message.id.0.as_bytes().to_vec(),
                }),
            );
            doc.insert(
                "channel_id",
                Bson::Binary(Binary {
                    subtype: BinarySubtype::Generic,
                    bytes: message.channel_id.0.as_bytes().to_vec(),
                }),
            );
            doc.insert(
                "author_id",
                Bson::Binary(Binary {
                    subtype: BinarySubtype::Generic,
                    bytes: message.author_id.0.as_bytes().to_vec(),
                }),
            );

            // attachments is an array of documents with `id` that should also be binary
            if let Some(bson_arr) = doc.get_mut("attachments") {
                if let Bson::Array(arr) = bson_arr {
                    for item in arr.iter_mut() {
                        if let Bson::Document(adoc) = item {
                            if let Some(Bson::String(s)) = adoc.get("id") {
                                // parse string uuid and insert binary
                                if let Ok(u) = Uuid::parse_str(s) {
                                    adoc.insert(
                                        "id",
                                        Bson::Binary(Binary {
                                            subtype: BinarySubtype::Generic,
                                            bytes: u.as_bytes().to_vec(),
                                        }),
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // store created_at as RFC3339 string to match serde's default chrono serialization
            doc.insert("created_at", Bson::String(now.to_rfc3339()));

            let raw_coll = self.db.collection::<Document>("messages");
            raw_coll
                .insert_one(doc)
                .await
                .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

            // Write outbox event for message creation with dynamic routing key
            let event = create_message_event_from_domain(
                message.id.0,
                message.channel_id.0,
                message.author_id.0,
                message.content.clone(),
                message.reply_to_message_id.map(|id| id.0),
                message.attachments.clone(),
            );
            let event_bytes = event_to_bytes(&event)
                .map_err(|e| CoreError::SerializationError { msg: e.to_string() })?;
            let routing_info =
                MessageRoutingInfo::new(self.routing_info.exchange.clone(), "message.created");
            let outbox_record = OutboxEventRecord::new(routing_info, event_bytes);
            write_outbox_event(&self.db, &outbox_record).await?;

            tracing::info!(
                message_id = %message.id,
                "Message create and outbox event written"
            );
        } else {
            return Err(CoreError::DatabaseError {
                msg: "Failed to convert message to BSON document".into(),
            });
        }

        Ok(message)
    }

    async fn find_by_id(&self, id: &MessageId) -> Result<Option<Message>, CoreError> {
        let collection = self.collection.clone();
        let id = *id;

        let id_bson = Bson::Binary(Binary {
            subtype: BinarySubtype::Generic,
            bytes: id.0.as_bytes().to_vec(),
        });

        collection
            .find_one(doc! { "_id": id_bson })
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })
    }

    async fn list(
        &self,
        channel_id: &crate::domain::message::entities::ChannelId,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Message>, TotalPaginatedElements), CoreError> {
        let collection = self.collection.clone();
        let options = Self::pagination_options(pagination);

        // build filter by channel_id
        let channel_bson = Bson::Binary(Binary {
            subtype: BinarySubtype::Generic,
            bytes: channel_id.0.as_bytes().to_vec(),
        });
        let filter = doc! { "channel_id": channel_bson };

        let total = collection
            .count_documents(filter.clone())
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        let mut cursor = collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        let mut messages = Vec::new();
        while let Some(message) = cursor
            .try_next()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?
        {
            messages.push(message);
        }

        Ok((messages, total))
    }

    async fn search_messages(
        &self,
        channel_id: &crate::domain::message::entities::ChannelId,
        query: &str,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Message>, TotalPaginatedElements), CoreError> {
        let collection = self.collection.clone();
        let options = Self::pagination_options(pagination);

        // build filter by channel_id and content regex (case-insensitive)
        let channel_bson = Bson::Binary(Binary {
            subtype: BinarySubtype::Generic,
            bytes: channel_id.0.as_bytes().to_vec(),
        });

        let filter = doc! {
            "channel_id": channel_bson,
            "content": { "$regex": query, "$options": "i" }
        };

        let total = collection
            .count_documents(filter.clone())
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        let mut cursor = collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        let mut messages = Vec::new();
        while let Some(message) = cursor
            .try_next()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?
        {
            messages.push(message);
        }

        Ok((messages, total))
    }

    async fn update(&self, input: UpdateMessageInput) -> Result<Message, CoreError> {
        let collection = self.collection.clone();

        let channel_id = match self.find_by_id(&input.id).await? {
            Some(msg) => msg.channel_id,
            None => {
                return Err(CoreError::MessageNotFound { id: input.id });
            }
        };

        let mut set = doc! {
            // store updated_at as RFC3339 string to match how `created_at` is serialized
            "updated_at": Utc::now().to_rfc3339()
        };

        if let Some(ref content) = input.content {
            set.insert("content", content);
        }

        if let Some(is_pinned) = input.is_pinned {
            set.insert("is_pinned", is_pinned);
        }

        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        let id_bson = Bson::Binary(Binary {
            subtype: BinarySubtype::Generic,
            bytes: input.id.0.as_bytes().to_vec(),
        });

        let updated = collection
            .find_one_and_update(doc! { "_id": id_bson }, doc! { "$set": set })
            .with_options(options)
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        let content_for_event = input.content.clone().unwrap_or_default();
        let is_pinned = input.is_pinned.unwrap_or(false);
        let event = update_message_event_from_domain(
            input.id,
            channel_id,
            content_for_event,
            is_pinned,
            vec![], //empty vector
        );

        let event_bytes = event_to_bytes(&event)
            .map_err(|e| CoreError::SerializationError { msg: e.to_string() })?;
        let routing_info =
            MessageRoutingInfo::new(self.routing_info.exchange.clone(), "message.updated");
        let outbox_record = OutboxEventRecord::new(routing_info, event_bytes);
        write_outbox_event(&self.db, &outbox_record).await?;

        updated.ok_or(CoreError::MessageNotFound { id: input.id })
    }

    async fn delete(&self, id: &MessageId) -> Result<(), CoreError> {
        let collection = self.collection.clone();
        let id = *id;

        // get channel_id for outbox event
        let channel_id = match self.find_by_id(&id).await? {
            Some(msg) => msg.channel_id,
            None => {
                return Err(CoreError::MessageNotFound { id });
            }
        };

        let id_bson = Bson::Binary(Binary {
            subtype: BinarySubtype::Generic,
            bytes: id.0.as_bytes().to_vec(),
        });

        let result = collection
            .delete_one(doc! { "_id": id_bson })
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        if result.deleted_count == 0 {
            return Err(CoreError::MessageNotFound { id });
        }

        let event = delete_message_event_from_domain(id, channel_id);
        let event_bytes = event_to_bytes(&event)
            .map_err(|e| CoreError::SerializationError { msg: e.to_string() })?;
        let routing_info =
            MessageRoutingInfo::new(self.routing_info.exchange.clone(), "message.deleted");
        let outbox_record = OutboxEventRecord::new(routing_info, event_bytes);
        write_outbox_event(&self.db, &outbox_record).await?;

        tracing::info!(
            message_id = %id,
            "Message delete and outbox event written"
        );

        Ok(())
    }
}
