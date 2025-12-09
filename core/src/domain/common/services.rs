use crate::domain::{health::port::HealthRepository, message::ports::MessageRepository};

#[derive(Clone)]
pub struct Service<S, H>
where
    S: MessageRepository,
    H: HealthRepository,
{
    pub(crate) message_repository: S,
    pub(crate) health_repository: H,
}

impl<S, H> Service<S, H>
where
    S: MessageRepository,
    H: HealthRepository,
{
    pub fn new(message_repository: S, health_repository: H) -> Self {
        Self {
            message_repository,
            health_repository,
        }
    }
}
