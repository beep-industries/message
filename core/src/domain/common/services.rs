use crate::domain::{health::port::HealthRepository, server::ports::ServerRepository};

#[derive(Clone)]
pub struct Service<S, H>
where
    S: ServerRepository,
    H: HealthRepository,
{
    pub(crate) server_repository: S,
    pub(crate) health_repository: H,
}

impl<S, H> Service<S, H>
where
    S: ServerRepository,
    H: HealthRepository,
{
    pub fn new(server_repository: S, health_repository: H) -> Self {
        Self {
            server_repository,
            health_repository,
        }
    }
}
