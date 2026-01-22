use events_protobuf::messages_events::create_message_event::Attachment;
use events_protobuf::messages_events::{CreateMessageEvent, NotifyEntry};
use uuid::Uuid;

/// Convert domain entities to protobuf CreateMessageEvent
pub fn create_message_event_from_domain(
    message_id: Uuid,
    channel_id: Uuid,
    author_id: Uuid,
    content: String,
    reply_to_message_id: Option<Uuid>,
    attachments: Vec<crate::domain::message::entities::Attachment>,
) -> CreateMessageEvent {
    CreateMessageEvent {
        message_id: message_id.to_string(),
        channel_id: channel_id.to_string(),
        author_id: author_id.to_string(),
        content,
        reply_to_message_id: reply_to_message_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
        attachments: attachments
            .into_iter()
            .map(|a| Attachment {
                id: a.id.0.to_string(),
                name: a.name,
                url: a.url,
            })
            .collect(),
        notify_entries: vec![],
    }
}

/// Serialize CreateMessageEvent to protobuf bytes for RabbitMQ publishing
pub fn create_message_event_to_bytes(
    event: &CreateMessageEvent,
) -> Result<Vec<u8>, prost::EncodeError> {
    use prost::Message;
    let mut buf = Vec::new();
    event.encode(&mut buf)?;
    Ok(buf)
}
