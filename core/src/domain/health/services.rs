use crate::domain::{
    common::{CoreError, services::Service},
    health::{
        entities::IsHealthy,
        port::{HealthRepository, HealthService},
    },
    message::ports::MessageRepository,
};

impl<S, H> HealthService for Service<S, H>
where
    S: MessageRepository,
    H: HealthRepository,
{
    async fn check_health(&self) -> Result<IsHealthy, CoreError> {
        self.health_repository.ping().await.to_result()
    }
}
