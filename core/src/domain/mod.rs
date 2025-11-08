use thiserror::Error;

use crate::domain::entities::MessageId;

pub mod entities;
pub mod ports;
pub mod services;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Unknown error: {message}")]
    Unknown { message: String },

    #[error("Message with ID {message_id} not found")]
    MessageNotFound { message_id: MessageId },
}
