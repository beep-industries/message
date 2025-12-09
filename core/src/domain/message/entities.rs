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

impl From<Uuid> for OwnerId {
    fn from(uuid: Uuid) -> Self {
        OwnerId(uuid)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct OwnerId(pub Uuid);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, sqlx::Type, Default, ToSchema)]
#[sqlx(type_name = "message_visibility", rename_all = "lowercase")]
pub enum MessageVisibility {
    #[default]
    Public,
    Private,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Message {
    pub id: MessageId,
    pub name: String,
    pub banner_url: Option<String>,
    pub picture_url: Option<String>,
    pub description: Option<String>,
    pub owner_id: OwnerId,
    pub visibility: MessageVisibility,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct InsertMessageInput {
    pub name: String,
    pub owner_id: OwnerId,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub visibility: MessageVisibility,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateMessageRequest {
    pub name: String,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub visibility: MessageVisibility,
}

impl CreateMessageRequest {
    pub fn into_input(self, owner_id: OwnerId) -> InsertMessageInput {
        InsertMessageInput {
            name: self.name,
            owner_id,
            picture_url: self.picture_url,
            banner_url: self.banner_url,
            description: self.description,
            visibility: self.visibility,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateMessageInput {
    pub id: MessageId,
    pub name: Option<String>,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<MessageVisibility>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateMessageRequest {
    pub name: Option<String>,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<MessageVisibility>,
}

impl UpdateMessageRequest {
    pub fn into_input(self, id: MessageId) -> UpdateMessageInput {
        UpdateMessageInput {
            id,
            name: self.name,
            picture_url: self.picture_url,
            banner_url: self.banner_url,
            description: self.description,
            visibility: self.visibility,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateMessageEvent {
    pub id: MessageId,
    pub name: Option<String>,
    pub picture_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<MessageVisibility>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteMessageEvent {
    pub id: MessageId,
}
