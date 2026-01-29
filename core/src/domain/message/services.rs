use crate::{domain::{
    attachment::port::AttachmentRepository, common::{CoreError, GetPaginated, TotalPaginatedElements, services::Service}, health::port::HealthRepository, message::{
        entities::{InsertMessageInput, Message, MessageId, UpdateMessageInput},
        ports::{MessageRepository, MessageService},
    }
}, infrastructure::outbox::entities::MessageOutboxEventRouting};

use crate::infrastructure::outbox::{MessageRoutingInfo, OutboxEventRecord};
use crate::domain::outbox::ports::OutboxEventRepository;
use crate::domain::message::events::{create_message_event_from_domain, event_to_bytes};

#[async_trait::async_trait]
impl<S, H, A, O> MessageService for Service<S, H, A, O>
where
    S: MessageRepository,
    H: HealthRepository,
    A: AttachmentRepository,
    O: OutboxEventRepository,
{
    async fn create_message(&self, input: InsertMessageInput) -> Result<Message, CoreError> {
        // Validate message content is not empty
        if input.content.trim().is_empty() {
            return Err(CoreError::InvalidMessageName);
        }

        // @TODO Authorization: Check if the user has permission to create messages

        // Create the message via repository
        let message = self.message_repository.insert(input).await?;

        // Outbox event logic moved here
        // Convert Vec<AttachmentId> to Vec<Attachment> with empty URLs (or fetch if needed)
        let attachments: Vec<crate::domain::message::entities::Attachment> = message.attachments.iter().cloned().map(|id| crate::domain::message::entities::Attachment { id, url: String::new() }).collect();
        let event = create_message_event_from_domain(
            message.id.0,
            message.channel_id.0,
            message.author_id.0,
            message.content.clone(),
            message.reply_to_message_id.map(|id| id.0),
            attachments,
        );
        let event_bytes = event_to_bytes(&event)
            .map_err(|e| CoreError::SerializationError { msg: e.to_string() })?;
        let routing_info = MessageRoutingInfo::new("notifications", "message.created");
        let outbox_record = OutboxEventRecord::new(routing_info, event_bytes);
        self.outbox_repository.write_event(&outbox_record, MessageOutboxEventRouting::Create).await?;

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
        channel_id: &crate::domain::message::entities::ChannelId,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Message>, TotalPaginatedElements), CoreError> {
        // @TODO Authorization: Filter messages by visibility based on user permissions

        let (messages, total) = self.message_repository.list(channel_id, pagination).await?;

        Ok((messages, total))
    }

    async fn search_messages(
        &self,
        channel_id: &crate::domain::message::entities::ChannelId,
        query: &str,
        pagination: &GetPaginated,
    ) -> Result<(Vec<Message>, TotalPaginatedElements), CoreError> {
        // @TODO Authorization: Filter messages by visibility based on user permissions

        let (messages, total) = self
            .message_repository
            .search_messages(channel_id, query, pagination)
            .await?;

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

        let event = create_message_event_from_domain(
            updated_message.id.0,
            updated_message.channel_id.0,
            updated_message.author_id.0,
            updated_message.content.clone(),
            updated_message.reply_to_message_id.map(|id| id.0),
            updated_message
                .attachments
                .iter()
                .cloned()
                .map(|id| crate::domain::message::entities::Attachment { id, url: String::new() })
                .collect(),
        );
        let event_bytes = event_to_bytes(&event)
            .map_err(|e| CoreError::SerializationError { msg: e.to_string() })?;
        let routing_info = MessageRoutingInfo::new("notifications", "message.updated");
        let outbox_record = OutboxEventRecord::new(routing_info, event_bytes);
        self.outbox_repository.write_event(&outbox_record, MessageOutboxEventRouting::Update).await?;

        Ok(updated_message)
    }

    async fn delete_message(&self, message_id: &MessageId) -> Result<(), CoreError> {
        // Check if message exists
        let existing_message = self.message_repository.find_by_id(message_id).await?;

        if existing_message.clone().is_none() {
            return Err(CoreError::MessageNotFound {
                id: message_id.clone(),
            });
        }

        // @TODO Authorization: Verify user is the message owner or has admin privileges

        // Delete the message
        self.message_repository.delete(message_id).await?;

        let existing_message = existing_message.unwrap();

        let event = create_message_event_from_domain(
            message_id.0,
            existing_message.channel_id.0,
            existing_message.author_id.0,
            existing_message.content.clone(),
            existing_message.reply_to_message_id.map(|id| id.0),
            existing_message
                .attachments
                .iter()
                .cloned()
                .map(|id| crate::domain::message::entities::Attachment { id, url: String::new() })
                .collect(),
        );
        let event_bytes = event_to_bytes(&event)
            .map_err(|e| CoreError::SerializationError { msg: e.to_string() })?;
        let routing_info = MessageRoutingInfo::new("notifications", "message.deleted");
        let outbox_record = OutboxEventRecord::new(routing_info, event_bytes);
        self.outbox_repository.write_event(&outbox_record, MessageOutboxEventRouting::Delete).await?;


        Ok(())
    }
}
