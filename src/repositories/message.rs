use crate::domain::{Message, MessageUpdate, SearchResult};
use crate::ports::{MessageRepository, RepoResult, RepositoryError};
use async_trait::async_trait;
use chrono::Utc;
use futures_util::stream::TryStreamExt;
use mongodb::{
    Client, Collection,
    bson::{Bson, doc, to_bson},
    options::ClientOptions,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct MongoRepo {
    col: Collection<Message>,
}

impl MongoRepo {
    pub async fn from_env() -> Result<Self, RepositoryError> {
        let mongo_url = std::env::var("MONGO_URL")
            .map_err(|e| RepositoryError::Other(format!("MONGO_URL missing: {}", e)))?;

        let options = ClientOptions::parse(&mongo_url).await.map_err(|e| {
            RepositoryError::Other(format!("failed to parse client options: {}", e))
        })?;

        let client = Client::with_options(options.clone())
            .map_err(|e| RepositoryError::Other(format!("failed to create client: {}", e)))?;

        let db_name = options
            .default_database
            .clone()
            .unwrap_or_else(|| "message".to_string());

        let db = client.database(&db_name);
        let col = db.collection::<Message>("messages");

        Ok(Self { col })
    }
}

#[async_trait]
impl MessageRepository for MongoRepo {
    async fn get(&self, channel: &str, id: Uuid) -> RepoResult<Message> {
        let id_bson = to_bson(&id).map_err(|e| RepositoryError::Other(e.to_string()))?;
        let filter =
            doc! { "id": id_bson, "channel_id": channel, "deleted_at": { "$eq": Bson::Null } };
        match self.col.find_one(filter, None).await {
            Ok(Some(msg)) => Ok(msg),
            Ok(None) => Err(RepositoryError::NotFound),
            Err(e) => Err(RepositoryError::Other(e.to_string())),
        }
    }

    async fn list(
        &self,
        channel: &str,
        limit: Option<u32>,
        before: Option<Uuid>,
    ) -> RepoResult<(Vec<Message>, Option<Uuid>)> {
        let mut filter = doc! { "channel_id": channel, "deleted_at": { "$eq": Bson::Null } };

        if let Some(before_id) = before {
            // find the message to get its created_at as cursor
            let before_bson =
                to_bson(&before_id).map_err(|e| RepositoryError::Other(e.to_string()))?;
            if let Ok(Some(cursor_msg)) = self.col.find_one(doc! { "id": before_bson }, None).await
            {
                let created_bson = to_bson(&cursor_msg.created_at)
                    .map_err(|e| RepositoryError::Other(e.to_string()))?;
                filter.insert("created_at", doc! { "$lt": created_bson });
            }
        }

        let limit_val = limit.unwrap_or(50) as i64;

        let find_opts = mongodb::options::FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(Some(limit_val))
            .build();

        let mut cursor = self
            .col
            .find(filter, find_opts)
            .await
            .map_err(|e| RepositoryError::Other(e.to_string()))?;
        let mut items = Vec::new();
        while let Some(m) = cursor
            .try_next()
            .await
            .map_err(|e| RepositoryError::Other(e.to_string()))?
        {
            items.push(m);
        }

        let next_before = if items.len() as i64 == limit_val && !items.is_empty() {
            Some(items.last().unwrap().id)
        } else {
            None
        };

        Ok((items, next_before))
    }

    async fn update(&self, id: Uuid, update: MessageUpdate) -> RepoResult<Message> {
        let edited_bson =
            to_bson(&Utc::now()).map_err(|e| RepositoryError::Other(e.to_string()))?;
        let mut set_doc = doc! { "edited_at": edited_bson };
        if let Some(content) = update.content {
            set_doc.insert("content", content);
        }
        if let Some(notify) = update.notify {
            set_doc.insert(
                "notify",
                to_bson(&notify).map_err(|e| RepositoryError::Other(e.to_string()))?,
            );
        }

        let update_doc = doc! { "$set": set_doc };

        let find_opts = mongodb::options::FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        let id_bson = to_bson(&id).map_err(|e| RepositoryError::Other(e.to_string()))?;
        match self
            .col
            .find_one_and_update(doc! { "id": id_bson }, update_doc, find_opts)
            .await
        {
            Ok(Some(m)) => Ok(m),
            Ok(None) => Err(RepositoryError::NotFound),
            Err(e) => Err(RepositoryError::Other(e.to_string())),
        }
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let deleted_bson =
            to_bson(&Utc::now()).map_err(|e| RepositoryError::Other(e.to_string()))?;
        let update_doc = doc! { "$set": { "deleted_at": deleted_bson } };
        let id_bson = to_bson(&id).map_err(|e| RepositoryError::Other(e.to_string()))?;
        match self
            .col
            .update_one(doc! { "id": id_bson }, update_doc, None)
            .await
        {
            Ok(res) => {
                if res.matched_count == 0 {
                    Err(RepositoryError::NotFound)
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(RepositoryError::Other(e.to_string())),
        }
    }

    async fn pin(&self, id: Uuid) -> RepoResult<()> {
        let id_bson = to_bson(&id).map_err(|e| RepositoryError::Other(e.to_string()))?;
        let update_doc = doc! { "$set": { "pinned": true } };
        match self
            .col
            .update_one(doc! { "id": id_bson }, update_doc, None)
            .await
        {
            Ok(res) => {
                if res.matched_count == 0 {
                    Err(RepositoryError::NotFound)
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(RepositoryError::Other(e.to_string())),
        }
    }

    async fn list_pins(
        &self,
        channel: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> RepoResult<(Vec<Message>, usize)> {
        let limit_val = limit.unwrap_or(50) as i64;
        let skip_val = offset.unwrap_or(0) as u64;

        let filter =
            doc! { "channel_id": channel, "pinned": true, "deleted_at": { "$eq": Bson::Null } };

        let find_opts = mongodb::options::FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(Some(limit_val))
            .skip(Some(skip_val))
            .build();

        let mut cursor = self
            .col
            .find(filter.clone(), find_opts)
            .await
            .map_err(|e| RepositoryError::Other(e.to_string()))?;
        let mut items = Vec::new();
        while let Some(m) = cursor
            .try_next()
            .await
            .map_err(|e| RepositoryError::Other(e.to_string()))?
        {
            items.push(m);
        }

        let total = self
            .col
            .count_documents(filter, None)
            .await
            .map_err(|e| RepositoryError::Other(e.to_string()))? as usize;

        Ok((items, total))
    }

    async fn search(
        &self,
        channel: &str,
        q: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        _in_docs: Option<bool>,
    ) -> RepoResult<(Vec<SearchResult>, usize)> {

        let limit_val = limit.unwrap_or(50) as i64;
        let skip_val = offset.unwrap_or(0) as u64;

        let filter = doc! {
            "channel_id": channel,
            "deleted_at": { "$eq": Bson::Null },
            "content": { "$regex": q, "$options": "i" }
        };

        let find_opts = mongodb::options::FindOptions::builder()
            .limit(Some(limit_val))
            .skip(Some(skip_val))
            .build();

        let mut cursor = self
            .col
            .find(filter.clone(), find_opts)
            .await
            .map_err(|e| RepositoryError::Other(e.to_string()))?;
        let mut items = Vec::new();
        while let Some(m) = cursor
            .try_next()
            .await
            .map_err(|e| RepositoryError::Other(e.to_string()))?
        {
            items.push(SearchResult {
                r#type: "message".to_string(),
                id: m.id.to_string(),
                channel_id: m.channel_id.clone(),
                snippet: m.content.chars().take(200).collect(),
                score: 1.0,
                message_id: Some(m.id),
                metadata: serde_json::Value::Null,
            });
        }

        let total = self
            .col
            .count_documents(filter, None)
            .await
            .map_err(|e| RepositoryError::Other(e.to_string()))? as usize;

        Ok((items, total))
    }
}

// Make MongoRepo available to other modules
pub use MongoRepo as Repo;
