use crate::domain::{
    common::{CoreError, GetPaginated, TotalPaginatedElements, services::Service},
    health::port::HealthRepository,
    message::{
        entities::{InsertMessageInput, Message, MessageId, UpdateMessageInput},
        ports::{MessageRepository, MessageService},
    },
};

#[async_trait::async_trait]
impl<S, H> MessageService for Service<S, H>
where
    S: MessageRepository,
    H: HealthRepository,
{
    async fn create_message(&self, input: InsertMessageInput) -> Result<Message, CoreError> {
        // Validate message content is not empty
        if input.content.trim().is_empty() {
            return Err(CoreError::InvalidMessageName);
        }

        // @TODO Authorization: Check if the user has permission to create messages

        // Create the message via repository
        let message = self.message_repository.insert(input).await?;

        Ok(message)
    }

    async fn get_message(&self, message_id: &MessageId) -> Result<Message, CoreError> {
        // @TODO Authorization: Check if the user has permission to access the message

        let message = self.message_repository.find_by_id(message_id).await?;

        match message {
            Some(message) => Ok(message),
            None => Err(CoreError::MessageNotFound {
                id: message_id.clone(),
            }),
        }
    }

    async fn list_messages(
        &self,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Message>, TotalPaginatedElements), CoreError> {
        // @TODO Authorization: Filter messages by visibility based on user permissions

        let (messages, total) = self.message_repository.list(pagination).await?;

        Ok((messages, total))
    }

    async fn update_message(&self, input: UpdateMessageInput) -> Result<Message, CoreError> {
        // Check if message exists
        let existing_message = self.message_repository.find_by_id(&input.id).await?;

        if existing_message.is_none() {
            return Err(CoreError::MessageNotFound {
                id: input.id.clone(),
            });
        }

        // @TODO Authorization: Verify user is the message owner or has admin privileges

        // Update the message
        let updated_message = self.message_repository.update(input).await?;

        Ok(updated_message)
    }

    async fn delete_message(&self, message_id: &MessageId) -> Result<(), CoreError> {
        // Check if message exists
        let existing_message = self.message_repository.find_by_id(message_id).await?;

        if existing_message.is_none() {
            return Err(CoreError::MessageNotFound {
                id: message_id.clone(),
            });
        }

        // @TODO Authorization: Verify user is the message owner or has admin privileges

        // Delete the message
        self.message_repository.delete(message_id).await?;

        Ok(())
    }
}
