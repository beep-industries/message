use crate::domain::{health::port::HealthRepository, message::ports::MessageRepository, attachment::port::AttachmentRepository, outbox::ports::OutboxEventRepository};

#[derive(Clone)]

pub struct Service<S, H, A, O>
where
    S: MessageRepository,
    H: HealthRepository,
    A: AttachmentRepository,
    O: OutboxEventRepository,
{
    pub(crate) message_repository: S,
    pub(crate) health_repository: H,
    pub(crate) attachment_repository: A,
    pub(crate) outbox_repository: O,
}

impl<S, H, A, O> Service<S, H, A, O>
where
    S: MessageRepository,
    H: HealthRepository,
    A: AttachmentRepository,
    O: OutboxEventRepository,
{
    pub fn new(message_repository: S, health_repository: H, attachment_repository: A, outbox_repository: O) -> Self {
        Self {
            message_repository,
            health_repository,
            attachment_repository,
            outbox_repository,
        }
    }
}
