use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignedUrl {
    pub url: String,
}

impl PresignedUrl {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Deref for PresignedUrl {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.url
    }
}