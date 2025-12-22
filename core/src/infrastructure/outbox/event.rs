use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Outbox event record (domain-level abstraction)
#[derive(Debug, Clone)]
pub struct OutboxEventRecord<TPayload, TRouter>
where
    TPayload: Serialize + Send + Sync,
    TRouter: MessageRouter + Send + Sync,
{
    pub id: Uuid,
    pub router: TRouter,
    pub payload: TPayload,
}

impl<TPayload, TRouter> OutboxEventRecord<TPayload, TRouter>
where
    TPayload: Serialize + Send + Sync,
    TRouter: MessageRouter + Send + Sync,
{
    pub fn new(router: TRouter, payload: TPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            router,
            payload,
        }
    }
}

/// Routing info (infrastructure-friendly, domain-safe)
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct MessageRoutingInfo {
    pub exchange: String,
    pub routing_key: String,
}

impl MessageRoutingInfo {
    pub fn new(exchange: impl Into<String>, routing_key: impl Into<String>) -> Self {
        Self {
            exchange: exchange.into(),
            routing_key: routing_key.into(),
        }
    }
}

/// Router abstraction
pub trait MessageRouter {
    fn exchange_name(&self) -> &str;
    fn routing_key(&self) -> &str;
}

impl MessageRouter for MessageRoutingInfo {
    fn exchange_name(&self) -> &str {
        &self.exchange
    }

    fn routing_key(&self) -> &str {
        &self.routing_key
    }
}
