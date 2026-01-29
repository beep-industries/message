
use crate::{
    Service,
    domain::{
        attachment::port::{AttachmentRepository, AttachmentService},
        common::CoreError,
        health::port::HealthRepository,
        message::{
            entities::Attachment,
            ports::MessageRepository,
        },
        outbox::ports::OutboxEventRepository,
    },
};

#[async_trait::async_trait]
impl<S, H, A, O> AttachmentService for Service<S, H, A, O>
where
    S: MessageRepository,
    H: HealthRepository,
    A: AttachmentRepository,
    O: OutboxEventRepository,
{
    async fn create_attachment(&self) -> Result<Attachment, CoreError> {
        // @TODO Authorization: Check if the user has permission to create messages

        // Create the message via repository
        let attachment = self.attachment_repository.post_attachment().await?;

        Ok(attachment)
    }

    async fn get_attachment(&self, id: String) -> Result<Attachment, CoreError> {
        let attachment = self.attachment_repository.get_attachment(id).await?;
        Ok(attachment)
    }
}
