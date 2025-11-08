use crate::domain::ports::message_repository::MessageRepository;

pub mod message;

pub struct Service<M>
where
    M: MessageRepository,
{
    pub(crate) message_repository: M,
}

impl<M> Service<M>
where
    M: MessageRepository,
{
    pub fn new(message_repository: M) -> Self {
        Self { message_repository }
    }
}
