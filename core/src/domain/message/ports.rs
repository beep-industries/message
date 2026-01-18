use std::sync::{Arc, Mutex};

use crate::domain::{
    common::{CoreError, GetPaginated, TotalPaginatedElements},
    message::entities::{InsertMessageInput, Message, MessageId, UpdateMessageInput},
};

#[async_trait::async_trait]
pub trait MessageRepository: Send + Sync {
    async fn insert(&self, input: InsertMessageInput) -> Result<Message, CoreError>;
    async fn find_by_id(&self, id: &MessageId) -> Result<Option<Message>, CoreError>;
    async fn list(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Message>, TotalPaginatedElements), CoreError>;
    async fn update(&self, input: UpdateMessageInput) -> Result<Message, CoreError>;
    async fn delete(&self, id: &MessageId) -> Result<(), CoreError>;
}

/// A service for managing message operations in the application.
///
/// This trait defines the core business logic operations that can be performed on messages.
/// It follows the ports and adapters pattern, where this trait acts as a port that defines
/// the interface for message-related operations. Implementations of this trait will provide
/// the actual business logic while maintaining separation of concerns.
///
/// The trait requires `Send + Sync` to ensure thread safety in async contexts, making it
/// suitable for use in web messages and other concurrent applications
///
/// # Thread Safety
///
/// All implementations must be thread-safe (`Send + Sync`) to support concurrent access
/// in multi-threaded environments.
#[async_trait::async_trait]
pub trait MessageService: Send + Sync {
    /// Creates a new message with the provided input.
    ///
    /// This method performs business logic validation before delegating to the repository.
    /// It ensures that all required fields are present and valid, and that the user
    /// creating the message has the necessary permissions.
    ///
    /// # Arguments
    ///
    /// * `input` - The message creation input containing name, owner_id, and optional fields
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to:
    /// - `Ok(Message)` - The newly created message
    /// - `Err(CoreError)` - If validation fails or repository operation fails
    async fn create_message(&self, input: InsertMessageInput) -> Result<Message, CoreError>;

    /// Retrieves a message by its unique identifier.
    ///
    /// This method performs the core business logic for fetching a message, including
    /// any necessary authorization checks and data validation. The implementation
    /// should handle cases where the message doesn't exist gracefully.
    ///
    /// # Arguments
    ///
    /// * `message_id` - A reference to the unique identifier of the message to retrieve.
    ///   This should be a valid [`MessageId`] that represents an existing message.
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to:
    /// - `Ok(Message)` - The message was found and the user has permission to access it
    /// - `Err(CoreError::MessageNotFound)` - No message exists with the given ID
    /// - `Err(CoreError)` - Other errors such as database connectivity issues or authorization failures
    async fn get_message(&self, message_id: &MessageId) -> Result<Message, CoreError>;

    /// Lists messages with pagination support.
    ///
    /// This method retrieves a paginated list of messages. The implementation should
    /// apply visibility filters based on user permissions and authorization rules.
    ///
    /// # Arguments
    ///
    /// * `pagination` - Pagination parameters (page and limit)
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to:
    /// - `Ok((Vec<Message>, TotalPaginatedElements))` - List of messages and total count
    /// - `Err(CoreError)` - If repository operation fails
    async fn list_messages(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Message>, TotalPaginatedElements), CoreError>;

    /// Updates an existing message with the provided input.
    ///
    /// This method validates that the message exists and that the user has permission
    /// to update it before applying the changes. Only non-None fields in the input
    /// will be updated.
    ///
    /// # Arguments
    ///
    /// * `input` - The message update input containing the message ID and fields to update
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to:
    /// - `Ok(Message)` - The updated message
    /// - `Err(CoreError::MessageNotFound)` - No message exists with the given ID
    /// - `Err(CoreError)` - If validation fails or repository operation fails
    async fn update_message(&self, input: UpdateMessageInput) -> Result<Message, CoreError>;

    /// Deletes a message by its unique identifier.
    ///
    /// This method validates that the message exists and that the user has permission
    /// to delete it before removing it from the repository.
    ///
    /// # Arguments
    ///
    /// * `message_id` - A reference to the unique identifier of the message to delete
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to:
    /// - `Ok(())` - The message was successfully deleted
    /// - `Err(CoreError::MessageNotFound)` - No message exists with the given ID
    /// - `Err(CoreError)` - If repository operation fails
    async fn delete_message(&self, message_id: &MessageId) -> Result<(), CoreError>;
}

#[derive(Clone)]
pub struct MockMessageRepository {
    messages: Arc<Mutex<Vec<Message>>>,
}

impl MockMessageRepository {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl MessageRepository for MockMessageRepository {
    async fn find_by_id(&self, id: &MessageId) -> Result<Option<Message>, CoreError> {
        let messages = self.messages.lock().unwrap();

        let message = messages.iter().find(|s| &s.id == id).cloned();

        Ok(message)
    }

    async fn list(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Message>, TotalPaginatedElements), CoreError> {
        let messages = self.messages.lock().unwrap();
        let total = messages.len() as u64;

        let offset = ((pagination.page - 1) * pagination.limit) as usize;
        let limit = pagination.limit as usize;

        let paginated_messages: Vec<Message> =
            messages.iter().skip(offset).take(limit).cloned().collect();

        Ok((paginated_messages, total))
    }

    async fn insert(&self, input: InsertMessageInput) -> Result<Message, CoreError> {
        let mut messages = self.messages.lock().unwrap();

        let new_message = Message {
            id: input.id,
            channel_id: input.channel_id,
            author_id: input.author_id,
            content: input.content,
            reply_to_message_id: input.reply_to_message_id,
            attachments: input.attachments,
            is_pinned: false,

            created_at: chrono::Utc::now(),
            updated_at: None,
        };

        messages.push(new_message.clone());

        Ok(new_message)
    }

    async fn update(&self, input: UpdateMessageInput) -> Result<Message, CoreError> {
        let mut messages = self.messages.lock().unwrap();

        let message = messages
            .iter_mut()
            .find(|s| &s.id == &input.id)
            .ok_or_else(|| CoreError::MessageNotFound {
                id: input.id.clone(),
            })?;

        if let Some(content) = input.content {
            message.content = content;
        }
        if let Some(is_pinned) = input.is_pinned {
            message.is_pinned = is_pinned;
        }
        message.updated_at = Some(chrono::Utc::now());

        Ok(message.clone())
    }

    async fn delete(&self, id: &MessageId) -> Result<(), CoreError> {
        let mut messages = self.messages.lock().unwrap();

        let index = messages
            .iter()
            .position(|s| &s.id == id)
            .ok_or_else(|| CoreError::MessageNotFound { id: id.clone() })?;

        messages.remove(index);

        Ok(())
    }
}
