use serde::Deserialize;
use thiserror::Error;
use utoipa::{IntoParams, ToSchema};

use crate::domain::message::entities::MessageId;

pub mod services;

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Service is currently unavailable")]
    ServiceUnavailable(String),

    #[error("Message with id {id} not found")]
    MessageNotFound { id: MessageId },

    #[error("Failed to insert message with name {name}")]
    FailedToInsertMessage { name: String },

    #[error("Message name cannot be empty")]
    InvalidMessageName,

    #[error("Health check failed")]
    Unhealthy,

    #[error("An unknown error occurred: {message}")]
    UnknownError { message: String },

    #[error("Database error: {msg}")]
    DatabaseError { msg: String },

    /// Serialization error occurred when converting event to JSON
    #[error("Serialization error: {msg}")]
    SerializationError { msg: String },
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetPaginated {
    pub page: u32,
    pub limit: u32,
}

impl Default for GetPaginated {
    fn default() -> Self {
        Self { page: 1, limit: 20 }
    }
}

pub type TotalPaginatedElements = u64;
