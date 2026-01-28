use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub struct RequestSignUrl {
    action: ContentVerb,
    expires_in_ms: u32,
}

impl From<ContentVerb> for RequestSignUrl {
    fn from(value: ContentVerb) -> Self {
        Self::new(value)
    }
}

pub struct PresignedUrl {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ContentVerb {
    Put,
    Get,
}