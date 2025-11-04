use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
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
    pub id: Uuid,
    pub channel_id: String,
    pub author_id: String,
    pub content: String,
    pub reply_to: Option<Uuid>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub notify: Vec<NotifyEntry>,
    #[serde(default)]
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageUpdate {
    pub content: Option<String>,
    pub notify: Option<Vec<NotifyEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(rename = "type")]
    pub r#type: String, // "message" | "document"
    pub id: String,
    pub channel_id: String,
    pub snippet: String,
    pub score: f64,
    pub message_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: Option<String>,
    pub code: Option<String>,
}
