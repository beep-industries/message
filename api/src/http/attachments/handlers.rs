use axum::{extract::State};
use messages_core::domain::{attachment::port::AttachmentService, message::entities::Attachment};

use crate::{ApiError, AppState, http::server::Response};

#[utoipa::path(
    post,
    path = "/messages/attachments",
    tag = "attachments",
    responses(
        (status = 201, description = "Attachment created successfully", body = Attachment),
        (status = 400, description = "Bad request - Invalid attachment data"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal attachment error")
    )
)]
#[tracing::instrument(skip(state))]
pub async fn create_attachment(
    State(state): State<AppState>,
) -> Result<Response<Attachment>, ApiError> {
    // Authorization is not needed for creating attachments links, it will be checked when sending the message

    let attachment = state.service.create_attachment().await?;
    Ok(Response::created(attachment))
}