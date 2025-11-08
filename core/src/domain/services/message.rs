use crate::domain::{
    ports::{message_repository::MessageRepository, message_service::MessageService},
    services::Service,
};

impl<M> MessageService for Service<M>
where
    M: MessageRepository,
{
    // Implement the required methods here
}
