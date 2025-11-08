use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::{
    Collection,
    bson::{Bson, doc, to_bson},
    options::FindOptions,
};

use crate::domain::{
    CoreError,
    entities::{ChannelId, Message, MessageId},
    ports::message_repository::MessageRepository,
};

pub struct MongoMessageRepository {
    col: Collection<Message>,
}

impl MessageRepository for MongoMessageRepository {
    async fn get(&self, channel_id: &ChannelId, id: &MessageId) -> Result<Message, CoreError> {
        let id_bson = to_bson(id).map_err(|e| CoreError::Unknown {
            message: e.to_string(),
        })?;
        let filter = doc! {
            "id": id_bson, "channel_id": channel_id.to_string(), "deleted_at": { "$eq": Bson::Null }
        };

        match self.col.find_one(filter).await {
            Ok(Some(msg)) => Ok(msg),
            Ok(None) => Err(CoreError::MessageNotFound {
                message_id: id.clone(),
            }),
            Err(e) => Err(CoreError::Unknown {
                message: e.to_string(),
            }),
        }
    }

    async fn delete(&self, message_id: &MessageId) -> Result<(), CoreError> {
        let deleted_bson = to_bson(&Utc::now()).map_err(|e| CoreError::Unknown {
            message: e.to_string(),
        })?;

        let update_doc = doc! { "$set": { "deleted_at": deleted_bson }};

        let id_bson = to_bson(message_id).map_err(|e| CoreError::Unknown {
            message: e.to_string(),
        })?;

        match self
            .col
            .update_one(doc! { "id": id_bson }, update_doc)
            .await
        {
            Ok(res) => {
                if res.matched_count == 0 {
                    return Err(CoreError::MessageNotFound {
                        message_id: message_id.clone(),
                    });
                }

                Ok(())
            }
            Err(e) => Err(CoreError::Unknown {
                message: e.to_string(),
            }),
        }
    }

    async fn list(
        &self,
        channel: &ChannelId,
        limit: Option<u32>,
        before: Option<&MessageId>,
    ) -> Result<(Vec<Message>, Option<MessageId>), CoreError> {
        let mut filter = doc! {
            "channel_id": channel.to_string(),
            "deleted_at": { "$eq": Bson::Null }
        };

        // Handle cursor-based pagination with "before"
        if let Some(before_id) = before {
            let before_bson = to_bson(before_id).map_err(|e| CoreError::Unknown {
                message: e.to_string(),
            })?;

            // Find the message to get its created_at timestamp
            if let Ok(Some(cursor_msg)) = self.col.find_one(doc! { "id": before_bson }).await {
                let created_bson =
                    to_bson(&cursor_msg.created_at).map_err(|e| CoreError::Unknown {
                        message: e.to_string(),
                    })?;
                filter.insert("created_at", doc! { "$lt": created_bson });
            }
        }

        let limit_val = limit.unwrap_or(50) as i64;
        let find_opts = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(Some(limit_val))
            .build();

        let mut cursor = self
            .col
            .find(filter)
            .with_options(find_opts)
            .await
            .map_err(|e| CoreError::Unknown {
                message: e.to_string(),
            })?;

        let mut items = Vec::new();
        while let Some(message) = cursor.try_next().await.map_err(|e| CoreError::Unknown {
            message: e.to_string(),
        })? {
            items.push(message);
        }

        let next_before = if items.len() as i64 == limit_val && !items.is_empty() {
            Some(items.last().unwrap().id.clone())
        } else {
            None
        };

        Ok((items, next_before))
    }

    async fn list_pinned_messages(
        &self,
        channel_id: &ChannelId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<(Vec<Message>, usize), CoreError> {
        let limit_val = limit.unwrap_or(50) as i64;
        let skip_val = offset.unwrap_or(0) as u64;

        let filter = doc! {
            "channel_id": channel_id.to_string(),
            "pinned": true,
            "deleted_at": { "$eq": Bson::Null }
        };

        let find_opts = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(Some(limit_val))
            .skip(Some(skip_val))
            .build();

        let mut cursor = self
            .col
            .find(filter.clone())
            .with_options(find_opts)
            .await
            .map_err(|e| CoreError::Unknown {
                message: e.to_string(),
            })?;

        let mut items = Vec::new();
        while let Some(message) = cursor.try_next().await.map_err(|e| CoreError::Unknown {
            message: e.to_string(),
        })? {
            // No conversion needed - cursor returns Message directly!
            items.push(message);
        }

        // Get total count of pinned messages
        let total = self
            .col
            .count_documents(filter)
            .await
            .map_err(|e| CoreError::Unknown {
                message: e.to_string(),
            })? as usize;

        Ok((items, total))
    }

    async fn pin_message(&self, message_id: &MessageId) -> Result<(), CoreError> {
        let id_bson = to_bson(message_id).map_err(|e| CoreError::Unknown {
            message: e.to_string(),
        })?;

        let update_doc = doc! { "$set": { "pinned": true } };

        match self
            .col
            .update_one(doc! { "id": id_bson }, update_doc)
            .await
        {
            Ok(res) => {
                if res.matched_count == 0 {
                    return Err(CoreError::MessageNotFound {
                        message_id: message_id.clone(),
                    });
                }
                Ok(())
            }
            Err(e) => Err(CoreError::Unknown {
                message: e.to_string(),
            }),
        }
    }
}
