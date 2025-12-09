use sqlx::{PgPool, query_as};

use crate::{
    domain::{
        common::{CoreError, GetPaginated, TotalPaginatedElements},
        message::{
            entities::{
                DeleteMessageEvent, InsertMessageInput, Message, MessageId, UpdateMessageInput,
            },
            ports::MessageRepository,
        },
    },
    infrastructure::{MessageRoutingInfo, outbox::OutboxEventRecord},
};

#[derive(Clone)]
pub struct PostgresMessageRepository {
    pub(crate) pool: PgPool,
    delete_message_router: MessageRoutingInfo,
    create_message_router: MessageRoutingInfo,
}

impl PostgresMessageRepository {
    pub fn new(
        pool: PgPool,
        delete_message_router: MessageRoutingInfo,
        create_message_router: MessageRoutingInfo,
    ) -> Self {
        Self {
            pool,
            delete_message_router,
            create_message_router,
        }
    }
}

impl MessageRepository for PostgresMessageRepository {
    async fn find_by_id(&self, id: &MessageId) -> Result<Option<Message>, CoreError> {
        let message = query_as!(
            Message,
            r#"
            SELECT id, name, banner_url, picture_url, description, owner_id, 
                   visibility as "visibility: _", created_at, updated_at
            FROM messages
            WHERE id = $1
            "#,
            id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CoreError::MessageNotFound { id: id.clone() })?;

        Ok(message)
    }

    async fn list(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Message>, TotalPaginatedElements), CoreError> {
        let offset = (pagination.page - 1) * pagination.limit;
        let limit = std::cmp::min(pagination.limit, 50) as i64;

        // Get total count of public messages only
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE visibility = 'public'")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        // Get paginated public messages only
        let messages = query_as!(
            Message,
            r#"
            SELECT id, name, banner_url, picture_url, description, owner_id,
                   visibility as "visibility: _", created_at, updated_at
            FROM messages
            WHERE visibility = 'public'
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        Ok((messages, total as u64))
    }

    async fn insert(&self, input: InsertMessageInput) -> Result<Message, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| CoreError::FailedToInsertMessage {
                name: input.name.clone(),
            })?;

        // Insert the message into the database
        let message = query_as!(
            Message,
            r#"
            INSERT INTO messages (name, owner_id, picture_url, banner_url, description, visibility)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, banner_url, picture_url, description, owner_id, 
                      visibility as "visibility: _", created_at, updated_at
            "#,
            input.name,
            input.owner_id.0,
            input.picture_url,
            input.banner_url,
            input.description,
            input.visibility as _
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| CoreError::FailedToInsertMessage {
            name: input.name.clone(),
        })?;

        // Write the create event to the outbox table for eventual processing
        let create_message_event =
            OutboxEventRecord::new(self.create_message_router.clone(), input.clone());
        create_message_event.write(&mut *tx).await?;

        tx.commit()
            .await
            .map_err(|_| CoreError::FailedToInsertMessage { name: input.name })?;

        Ok(message)
    }

    async fn update(&self, input: UpdateMessageInput) -> Result<Message, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        // First, fetch the current message to get existing values
        let current = query_as!(
            Message,
            r#"
            SELECT id, name, banner_url, picture_url, description, owner_id, 
                   visibility as "visibility: _", created_at, updated_at
            FROM messages
            WHERE id = $1
            "#,
            input.id.0
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| CoreError::MessageNotFound {
            id: input.id.clone(),
        })?
        .ok_or_else(|| CoreError::MessageNotFound {
            id: input.id.clone(),
        })?;

        // Apply updates, falling back to current values if not provided
        let new_name = input.name.as_ref().unwrap_or(&current.name);
        let new_picture_url = input.picture_url.as_ref().or(current.picture_url.as_ref());
        let new_banner_url = input.banner_url.as_ref().or(current.banner_url.as_ref());
        let new_description = input.description.as_ref().or(current.description.as_ref());
        let new_visibility = input.visibility.as_ref().unwrap_or(&current.visibility);

        // Update the message in the database
        let message = query_as!(
            Message,
            r#"
            UPDATE messages
            SET name = $1, picture_url = $2, banner_url = $3, description = $4, visibility = $5
            WHERE id = $6
            RETURNING id, name, banner_url, picture_url, description, owner_id, 
                      visibility as "visibility: _", created_at, updated_at
            "#,
            new_name,
            new_picture_url,
            new_banner_url,
            new_description,
            new_visibility as _,
            input.id.0
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| CoreError::MessageNotFound {
            id: input.id.clone(),
        })?;

        tx.commit()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        Ok(message)
    }

    async fn delete(&self, id: &MessageId) -> Result<(), CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        // Delete the message inside the database
        let result = sqlx::query(r#"DELETE FROM messages WHERE id = $1"#)
            .bind(id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::MessageNotFound { id: id.clone() });
        }

        // Write the delete event to the outbox table
        // for eventual processing
        let event = DeleteMessageEvent { id: id.clone() };
        let delete_message_event =
            OutboxEventRecord::new(self.delete_message_router.clone(), event);
        delete_message_event.write(&mut *tx).await?;

        tx.commit()
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        Ok(())
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn test_insert_message_writes_row_and_outbox(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::message::entities::{InsertMessageInput, MessageVisibility, OwnerId};
    use crate::infrastructure::outbox::MessageRouter;
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.created".to_string(),
    );

    let repository = PostgresMessageRepository::new(
        pool.clone(),
        MessageRoutingInfo::default(),
        create_router.clone(),
    );

    let owner_id = OwnerId(Uuid::new_v4());
    let input = InsertMessageInput {
        name: "my test message".to_string(),
        owner_id: owner_id.clone(),
        picture_url: Some("https://example.com/pic.png".to_string()),
        banner_url: Some("https://example.com/banner.png".to_string()),
        description: Some("a description".to_string()),
        visibility: MessageVisibility::Public,
    };

    // Act: insert message
    let created = repository.insert(input.clone()).await?;

    // Assert: returned fields
    assert_eq!(created.name, input.name);
    assert_eq!(created.owner_id, owner_id);
    assert_eq!(created.picture_url, input.picture_url);
    assert_eq!(created.banner_url, input.banner_url);
    assert_eq!(created.description, input.description);
    assert_eq!(created.visibility, input.visibility);
    // id should be set and created_at present
    assert!(created.updated_at.is_none());

    // Assert: it can be fetched back
    let fetched = repository.find_by_id(&created.id).await?;
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, created.name);

    // Assert: an outbox message was written with expected routing and payload
    // Note: payload is the serialized InsertMessageInput
    use sqlx::Row;
    let row = sqlx::query(
        r#"
        SELECT exchange_name, routing_key, payload
        FROM outbox_messages
        WHERE exchange_name = $1 AND routing_key = $2
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(create_router.exchange_name())
    .bind(create_router.routing_key())
    .fetch_one(&pool)
    .await
    .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    let exchange_name: String = row
        .try_get("exchange_name")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    let routing_key: String = row
        .try_get("routing_key")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    assert_eq!(exchange_name, create_router.exchange_name());
    assert_eq!(routing_key, create_router.routing_key());

    // Validate the payload JSON contains the message name and owner_id
    let payload: serde_json::Value = row
        .try_get("payload")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    assert_eq!(
        payload.get("name").and_then(|v| v.as_str()),
        Some(created.name.as_str())
    );
    // OwnerId is a newtype around Uuid and serializes to the inner value
    let owner_str = owner_id.0.to_string();
    assert_eq!(
        payload.get("owner_id").and_then(|v| v.as_str()),
        Some(owner_str.as_str())
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_find_by_id_returns_none_for_nonexistent(pool: PgPool) -> Result<(), CoreError> {
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.created".to_string(),
    );
    let delete_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.deleted".to_string(),
    );

    let repository = PostgresMessageRepository::new(pool.clone(), delete_router, create_router);

    // Try to find a message with a random UUID that doesn't exist
    let nonexistent_id = MessageId(Uuid::new_v4());
    let result = repository.find_by_id(&nonexistent_id).await?;

    // Assert: should return None
    assert!(result.is_none());

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_nonexistent_returns_error(pool: PgPool) -> Result<(), CoreError> {
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.created".to_string(),
    );
    let delete_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.deleted".to_string(),
    );

    let repository = PostgresMessageRepository::new(pool.clone(), delete_router, create_router);

    // Try to delete a message with a random UUID that doesn't exist
    let nonexistent_id = MessageId(Uuid::new_v4());
    let result = repository.delete(&nonexistent_id).await;

    // Assert: should return MessageNotFound error
    assert!(result.is_err());
    match result {
        Err(CoreError::MessageNotFound { id }) => {
            assert_eq!(id, nonexistent_id);
        }
        _ => panic!("Expected MessageNotFound error"),
    }

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_message_removes_row_and_outbox(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::message::entities::{InsertMessageInput, MessageVisibility, OwnerId};
    use crate::infrastructure::outbox::MessageRouter;
    use sqlx::Row;
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.created".to_string(),
    );
    let delete_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.deleted".to_string(),
    );

    let repository =
        PostgresMessageRepository::new(pool.clone(), delete_router.clone(), create_router);

    // Arrange: insert a message first
    let owner_id = OwnerId(Uuid::new_v4());
    let input = InsertMessageInput {
        name: "to delete".to_string(),
        owner_id: owner_id.clone(),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: MessageVisibility::Private,
    };
    let created = repository.insert(input).await?;

    // Act: delete it
    repository.delete(&created.id).await?;

    // Assert: it's gone
    let fetched = repository.find_by_id(&created.id).await?;
    assert!(fetched.is_none());

    // Assert: an outbox message for delete was written
    let row = sqlx::query(
        r#"
        SELECT exchange_name, routing_key, payload
        FROM outbox_messages
        WHERE routing_key = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(delete_router.routing_key())
    .fetch_one(&pool)
    .await
    .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    let exchange_name: String = row
        .try_get("exchange_name")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    let routing_key: String = row
        .try_get("routing_key")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;
    assert_eq!(exchange_name, delete_router.exchange_name());
    assert_eq!(routing_key, delete_router.routing_key());

    let payload: serde_json::Value = row
        .try_get("payload")
        .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

    // Payload should be { "id": "<uuid>" }
    let id_str = created.id.0.to_string();
    assert_eq!(
        payload.get("id").and_then(|v| v.as_str()),
        Some(id_str.as_str())
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_message_updates_fields(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::message::entities::{
        InsertMessageInput, MessageVisibility, OwnerId, UpdateMessageInput,
    };
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.created".to_string(),
    );

    let repository =
        PostgresMessageRepository::new(pool.clone(), MessageRoutingInfo::default(), create_router);

    // Arrange: insert a message first
    let owner_id = OwnerId(Uuid::new_v4());
    let input = InsertMessageInput {
        name: "original name".to_string(),
        owner_id: owner_id.clone(),
        picture_url: Some("https://example.com/old.png".to_string()),
        banner_url: Some("https://example.com/old-banner.png".to_string()),
        description: Some("old description".to_string()),
        visibility: MessageVisibility::Public,
    };
    let created = repository.insert(input).await?;

    // Act: update the message
    let update_input = UpdateMessageInput {
        id: created.id.clone(),
        name: Some("updated name".to_string()),
        picture_url: Some("https://example.com/new.png".to_string()),
        banner_url: None,
        description: Some("new description".to_string()),
        visibility: Some(MessageVisibility::Private),
    };
    let updated = repository.update(update_input.clone()).await?;

    // Assert: returned message has updated fields
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "updated name");
    assert_eq!(
        updated.picture_url,
        Some("https://example.com/new.png".to_string())
    );
    assert_eq!(
        updated.banner_url,
        Some("https://example.com/old-banner.png".to_string())
    ); // unchanged
    assert_eq!(updated.description, Some("new description".to_string()));
    assert_eq!(updated.visibility, MessageVisibility::Private);
    assert!(updated.updated_at.is_some());

    // Assert: it can be fetched back with updates
    let fetched = repository.find_by_id(&created.id).await?;
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.name, "updated name");
    assert_eq!(
        fetched.picture_url,
        Some("https://example.com/new.png".to_string())
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_nonexistent_message_returns_error(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::message::entities::UpdateMessageInput;
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.created".to_string(),
    );

    let repository =
        PostgresMessageRepository::new(pool.clone(), MessageRoutingInfo::default(), create_router);

    // Try to update a message with a random UUID that doesn't exist
    let nonexistent_id = MessageId(Uuid::new_v4());
    let update_input = UpdateMessageInput {
        id: nonexistent_id.clone(),
        name: Some("new name".to_string()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: None,
    };
    let result = repository.update(update_input).await;

    // Assert: should return MessageNotFound error
    assert!(result.is_err());
    match result {
        Err(CoreError::MessageNotFound { id }) => {
            assert_eq!(id, nonexistent_id);
        }
        _ => panic!("Expected MessageNotFound error"),
    }

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_message_with_no_fields_returns_unchanged(
    pool: PgPool,
) -> Result<(), CoreError> {
    use crate::domain::message::entities::{
        InsertMessageInput, MessageVisibility, OwnerId, UpdateMessageInput,
    };
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.created".to_string(),
    );

    let repository =
        PostgresMessageRepository::new(pool.clone(), MessageRoutingInfo::default(), create_router);

    // Arrange: insert a message first
    let owner_id = OwnerId(Uuid::new_v4());
    let input = InsertMessageInput {
        name: "test message".to_string(),
        owner_id: owner_id.clone(),
        picture_url: Some("https://example.com/pic.png".to_string()),
        banner_url: None,
        description: None,
        visibility: MessageVisibility::Public,
    };
    let created = repository.insert(input).await?;

    // Act: update with no fields
    let update_input = UpdateMessageInput {
        id: created.id.clone(),
        name: None,
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: None,
    };
    let result = repository.update(update_input).await?;

    // Assert: returned message is unchanged
    assert_eq!(result.id, created.id);
    assert_eq!(result.name, created.name);
    assert_eq!(result.picture_url, created.picture_url);

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_messages_with_pagination(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::common::GetPaginated;
    use crate::domain::message::entities::{InsertMessageInput, MessageVisibility, OwnerId};
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.created".to_string(),
    );

    let repository =
        PostgresMessageRepository::new(pool.clone(), MessageRoutingInfo::default(), create_router);

    // Arrange: insert multiple messages
    let owner_id = OwnerId(Uuid::new_v4());
    for i in 1..=5 {
        let input = InsertMessageInput {
            name: format!("Message {}", i),
            owner_id: owner_id.clone(),
            picture_url: None,
            banner_url: None,
            description: Some(format!("Description {}", i)),
            visibility: MessageVisibility::Public,
        };
        repository.insert(input).await?;
    }

    // Act: list first page with 2 items
    let pagination = GetPaginated { page: 1, limit: 2 };
    let (messages, total) = repository.list(&pagination).await?;

    // Assert: correct pagination
    assert_eq!(total, 5);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].name, "Message 5"); // Most recent first
    assert_eq!(messages[1].name, "Message 4");

    // Act: list second page
    let pagination = GetPaginated { page: 2, limit: 2 };
    let (messages, total) = repository.list(&pagination).await?;

    // Assert: correct pagination
    assert_eq!(total, 5);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].name, "Message 3");
    assert_eq!(messages[1].name, "Message 2");

    // Act: list third page
    let pagination = GetPaginated { page: 3, limit: 2 };
    let (messages, total) = repository.list(&pagination).await?;

    // Assert: correct pagination
    assert_eq!(total, 5);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].name, "Message 1");

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_messages_filters_only_public(pool: PgPool) -> Result<(), CoreError> {
    use crate::domain::common::GetPaginated;
    use crate::domain::message::entities::{InsertMessageInput, MessageVisibility, OwnerId};
    use uuid::Uuid;

    let create_router = MessageRoutingInfo::new(
        "message.exchange".to_string(),
        "message.created".to_string(),
    );

    let repository =
        PostgresMessageRepository::new(pool.clone(), MessageRoutingInfo::default(), create_router);

    // Arrange: insert messages with mixed visibility
    let owner_id = OwnerId(Uuid::new_v4());

    // Create 3 public messages
    for i in 1..=3 {
        let input = InsertMessageInput {
            name: format!("Public Message {}", i),
            owner_id: owner_id.clone(),
            picture_url: None,
            banner_url: None,
            description: Some(format!("Public description {}", i)),
            visibility: MessageVisibility::Public,
        };
        repository.insert(input).await?;
    }

    // Create 2 private messages
    for i in 1..=2 {
        let input = InsertMessageInput {
            name: format!("Private Message {}", i),
            owner_id: owner_id.clone(),
            picture_url: None,
            banner_url: None,
            description: Some(format!("Private description {}", i)),
            visibility: MessageVisibility::Private,
        };
        repository.insert(input).await?;
    }

    // Act: list all messages
    let pagination = GetPaginated { page: 1, limit: 10 };
    let (messages, total) = repository.list(&pagination).await?;

    // Assert: returns only public messages (per security requirements)
    assert_eq!(
        total, 3,
        "Should return total count of public messages only"
    );
    assert_eq!(
        messages.len(),
        3,
        "Should return only public messages in the page"
    );

    // Verify all returned messages are public
    let all_public = messages
        .iter()
        .all(|s| s.visibility == MessageVisibility::Public);

    assert!(all_public, "All returned messages should be public");

    // Verify ordering (most recent first)
    assert_eq!(messages[0].name, "Public Message 3");
    assert_eq!(messages[1].name, "Public Message 2");
    assert_eq!(messages[2].name, "Public Message 1");

    Ok(())
}
