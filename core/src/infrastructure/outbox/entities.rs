#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MessageOutboxEventRouting {
    Create,
    Update,
    Delete,
}

impl MessageOutboxEventRouting {
    pub fn to_event_type(&self) -> &str {
        match self {
            MessageOutboxEventRouting::Create => "message.create",
            MessageOutboxEventRouting::Update => "message.update",
            MessageOutboxEventRouting::Delete => "message.delete",
        }
    }

    pub fn to_routing_key(&self) -> &str {
        match self {
            MessageOutboxEventRouting::Create => "message.created",
            MessageOutboxEventRouting::Update => "message.updated",
            MessageOutboxEventRouting::Delete => "message.deleted",
        }
    }

    pub fn get_exchange(&self) -> &str {
        "notifications"
    }
}

