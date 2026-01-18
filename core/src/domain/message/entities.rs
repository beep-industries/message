use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct MessageId(pub Uuid);

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for MessageId {
    fn from(uuid: Uuid) -> Self {
        MessageId(uuid)
    }
}

impl From<MessageId> for Uuid {
    fn from(message_id: MessageId) -> Self {
        message_id.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ChannelId(pub Uuid);

impl std::fmt::Display for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ChannelId {
    fn from(uuid: Uuid) -> Self {
        ChannelId(uuid)
    }
}

impl From<ChannelId> for Uuid {
    fn from(message_id: ChannelId) -> Self {
        message_id.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct AuthorId(pub Uuid);

impl std::fmt::Display for AuthorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for AuthorId {
    fn from(uuid: Uuid) -> Self {
        AuthorId(uuid)
    }
}

impl From<AuthorId> for Uuid {
    fn from(message_id: AuthorId) -> Self {
        message_id.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct AttachmentId(pub Uuid);

impl std::fmt::Display for AttachmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for AttachmentId {
    fn from(uuid: Uuid) -> Self {
        AttachmentId(uuid)
    }
}

impl From<AttachmentId> for Uuid {
    fn from(message_id: AttachmentId) -> Self {
        message_id.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Attachment {
    pub id: AttachmentId,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub author_id: AuthorId,
    pub content: String,
    pub reply_to_message_id: Option<MessageId>,
    pub attachments: Vec<Attachment>,
    pub is_pinned: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct InsertMessageInput {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub author_id: AuthorId,
    pub content: String,
    pub reply_to_message_id: Option<MessageId>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateMessageRequest {
    pub channel_id: ChannelId,
    pub content: String,
    pub reply_to_message_id: Option<MessageId>,
    pub attachments: Vec<Attachment>,
}

impl CreateMessageRequest {
    pub fn into_input(self, author_id: AuthorId) -> InsertMessageInput {
        InsertMessageInput {
            id: MessageId::from(Uuid::new_v4()),
            channel_id: self.channel_id,
            author_id,
            content: self.content,
            reply_to_message_id: self.reply_to_message_id,
            attachments: self.attachments,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateMessageInput {
    pub id: MessageId,
    pub content: Option<String>,
    pub is_pinned: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateMessageRequest {
    pub content: Option<String>,
    pub is_pinned: Option<bool>,
}

impl UpdateMessageRequest {
    pub fn into_input(self, id: MessageId) -> UpdateMessageInput {
        UpdateMessageInput {
            id,
            content: self.content,
            is_pinned: self.is_pinned,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateMessageEvent {
    pub id: MessageId,
    pub content: String,
    pub is_pinned: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteMessageEvent {
    pub id: MessageId,
}
