use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: AttachmentId,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyEntry {
    #[serde(rename = "type")]
    pub r#type: String, // "role" | "member"
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub author_id: UserId,
    pub content: String,
    pub reply_to: Option<MessageId>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub notify: Vec<NotifyEntry>,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}
