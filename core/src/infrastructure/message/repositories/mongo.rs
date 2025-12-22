use std::future::Future;

use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    Collection, Database,
    bson::{Bson, DateTime as BsonDateTime, doc},
    options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument},
};

use crate::domain::{
    common::{CoreError, GetPaginated, TotalPaginatedElements},
    message::{
        entities::{InsertMessageInput, Message, MessageId, UpdateMessageInput},
        ports::MessageRepository,
    },
};

#[derive(Clone)]
pub struct MongoMessageRepository {
    collection: Collection<Message>,
}

impl MongoMessageRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Message>("messages"),
        }
    }

    fn pagination_options(pagination: &GetPaginated) -> FindOptions {
        let limit = pagination.limit.min(50) as i64;
        let skip = ((pagination.page - 1) * pagination.limit) as u64;

        FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(skip)
            .limit(limit)
            .build()
    }
}

impl MessageRepository for MongoMessageRepository {
    fn insert(
        &self,
        input: InsertMessageInput,
    ) -> impl Future<Output = Result<Message, CoreError>> + Send {
        let collection = self.collection.clone();

        async move {
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

            collection
                .insert_one(&message)
                .await
                .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

            Ok(message)
        }
    }

    fn find_by_id(
        &self,
        id: &MessageId,
    ) -> impl Future<Output = Result<Option<Message>, CoreError>> + Send {
        let collection = self.collection.clone();
        let id = *id;

        async move {
            collection
                .find_one(doc! { "_id": Bson::from(id.0) })
                .await
                .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })
        }
    }

    fn list(
        &self,
        pagination: &GetPaginated,
    ) -> impl Future<Output = Result<(Vec<Message>, TotalPaginatedElements), CoreError>> + Send
    {
        let collection = self.collection.clone();
        let options = Self::pagination_options(pagination);

        async move {
            let filter = doc! {};

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
    }

    fn update(
        &self,
        input: UpdateMessageInput,
    ) -> impl Future<Output = Result<Message, CoreError>> + Send {
        let collection = self.collection.clone();

        async move {
            let mut set = doc! {
                "updated_at": BsonDateTime::now()
            };

            if let Some(content) = input.content {
                set.insert("content", content);
            }

            if let Some(is_pinned) = input.is_pinned {
                set.insert("is_pinned", is_pinned);
            }

            let options = FindOneAndUpdateOptions::builder()
                .return_document(ReturnDocument::After)
                .build();

            let updated = collection
                .find_one_and_update(doc! { "_id": Bson::from(input.id.0) }, doc! { "$set": set })
                .with_options(options)
                .await
                .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

            updated.ok_or(CoreError::MessageNotFound { id: input.id })
        }
    }

    fn delete(&self, id: &MessageId) -> impl Future<Output = Result<(), CoreError>> + Send {
        let collection = self.collection.clone();
        let id = *id;

        async move {
            let result = collection
                .delete_one(doc! { "_id": Bson::from(id.0) })
                .await
                .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

            if result.deleted_count == 0 {
                return Err(CoreError::MessageNotFound { id });
            }

            Ok(())
        }
    }
}
