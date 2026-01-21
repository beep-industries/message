use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a notify entry indicating who should be notified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyEntry {
    pub user_id: String,
    pub notify_type: String,
}

/// Attachment information for a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentProto {
    pub id: String,
    pub name: String,
    pub url: String,
}

/// CreateMessageEvent - Matches the protobuf schema from the real-time service
/// This event is published to RabbitMQ when a new message is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageEvent {
    pub message_id: String,
    pub channel_id: String,
    pub author_id: String,
    pub content: String,
    pub reply_to_message_id: Option<String>,
    pub attachments: Vec<AttachmentProto>,
    pub notify_entries: Vec<NotifyEntry>,
}

impl CreateMessageEvent {
    /// Convert domain entities to protobuf event
    pub fn from_domain(
        message_id: Uuid,
        channel_id: Uuid,
        author_id: Uuid,
        content: String,
        reply_to_message_id: Option<Uuid>,
        attachments: Vec<crate::domain::message::entities::Attachment>,
    ) -> Self {
        Self {
            message_id: message_id.to_string(),
            channel_id: channel_id.to_string(),
            author_id: author_id.to_string(),
            content,
            reply_to_message_id: reply_to_message_id.map(|id| id.to_string()),
            attachments: attachments
                .into_iter()
                .map(|a| AttachmentProto {
                    id: a.id.0.to_string(),
                    name: a.name,
                    url: a.url,
                })
                .collect(),
            notify_entries: vec![],
        }
    }

    /// Serialize to JSON bytes for RabbitMQ publishing
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}
