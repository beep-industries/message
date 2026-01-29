
use crate::domain::{
    attachment::port::AttachmentRepository, common::{CoreError, services::Service}, health::{
        entities::IsHealthy,
        port::{HealthRepository, HealthService},
    }, message::ports::MessageRepository, outbox::ports::OutboxEventRepository
};

impl<S, H, A, O> HealthService for Service<S, H, A, O>
where
    S: MessageRepository,
    H: HealthRepository,
    A: AttachmentRepository,
    O: OutboxEventRepository,
{
    async fn check_health(&self) -> Result<IsHealthy, CoreError> {
        self.health_repository.ping().await.to_result()
    }
}
