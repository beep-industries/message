use events_protobuf::messages_events::create_message_event::Attachment;
use events_protobuf::messages_events::{
    CreateMessageEvent, DeleteMessageEvent, NotifyEntry, UpdateMessageEvent
};
use uuid::Uuid;

use crate::domain::message::entities::{ChannelId, MessageId};

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

pub fn delete_message_event_from_domain(
    message_id: MessageId,
    channel_id: ChannelId,
) -> DeleteMessageEvent {
    DeleteMessageEvent {
        message_id: message_id.to_string(),
        channel_id: channel_id.to_string(),
    }
}

pub fn update_message_event_from_domain(
    message_id: MessageId,
    channel_id: ChannelId,
    new_content: String,
    is_pinned: bool,
    notify_entries: Vec<NotifyEntry>,
) -> UpdateMessageEvent {
    UpdateMessageEvent {
        message_id: message_id.to_string(),
        channel_id: channel_id.to_string(),
        content: Some(new_content),
        is_pinned: Some(is_pinned),
        notify_entries,
    }
}

/// Serialize any prost::Message to protobuf bytes for RabbitMQ publishing
pub fn event_to_bytes<M: prost::Message>(event: &M) -> Result<Vec<u8>, prost::EncodeError> {
    let mut buf = Vec::new();
    event.encode(&mut buf)?;
    Ok(buf)
}
