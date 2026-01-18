use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use communities_core::domain::{
    common::GetPaginated,
    message::{
        entities::{AuthorId, CreateMessageRequest, Message, MessageId, UpdateMessageRequest},
        ports::MessageService,
    },
};
use uuid::Uuid;

use crate::http::server::{
    ApiError, AppState, Response, middleware::auth::entities::UserIdentity,
    response::PaginatedResponse,
};

#[utoipa::path(
    post,
    path = "/messages",
    tag = "messages",
    request_body = CreateMessageRequest,
    responses(
        (status = 201, description = "Message created successfully", body = Message),
        (status = 400, description = "Bad request - Invalid message name"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal message error")
    )
)]
#[tracing::instrument(skip(state, user_identity, request))]
pub async fn create_message(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Json(request): Json<CreateMessageRequest>,
) -> Result<Response<Message>, ApiError> {
    let owner_id = AuthorId::from(user_identity.user_id);
    let input = request.into_input(owner_id);
    let message = state.service.create_message(input).await?;
    Ok(Response::created(message))
}

#[utoipa::path(
    get,
    path = "/messages/{id}",
    tag = "messages",
    params(
        ("id" = String, Path, description = "Message ID")
    ),
    responses(
        (status = 200, description = "Message retrieved successfully", body = Message),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Message is private"),
        (status = 404, description = "Message not found"),
        (status = 500, description = "Internal message error")
    )
)]
#[tracing::instrument(skip(state))]
pub async fn get_message(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Response<Message>, ApiError> {
    let message_id = MessageId::from(id);
    let message = state.service.get_message(&message_id).await?;

    //FIXME: Check that user is allowed to see the message. (is in the channel's server etc)

    Ok(Response::ok(message))
}

#[utoipa::path(
    get,
    path = "/messages",
    tag = "messages",
    params(
        GetPaginated
    ),
    responses(
        (status = 200, description = "List of messages retrieved successfully", body = PaginatedResponse<Message>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal message error")
    )
)]
#[tracing::instrument(skip(state, _user_identity, pagination))]
pub async fn list_messages(
    State(state): State<AppState>,
    Extension(_user_identity): Extension<UserIdentity>,
    Query(pagination): Query<GetPaginated>,
) -> Result<Response<PaginatedResponse<Message>>, ApiError> {
    let (messages, total) = state.service.list_messages(&pagination).await?;

    let response = PaginatedResponse {
        data: messages,
        total,
        page: pagination.page,
    };

    Ok(Response::ok(response))
}

#[utoipa::path(
    put,
    path = "/messages/{id}",
    tag = "messages",
    params(
        ("id" = String, Path, description = "Message ID")
    ),
    request_body = UpdateMessageRequest,
    responses(
        (status = 200, description = "Message updated successfully", body = Message),
        (status = 400, description = "Bad request - Invalid message name"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not the message owner"),
        (status = 404, description = "Message not found"),
        (status = 500, description = "Internal message error")
    )
)]
#[tracing::instrument(skip(state, user_identity, request))]
pub async fn update_message(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Json(request): Json<UpdateMessageRequest>,
) -> Result<Response<Message>, ApiError> {
    let message_id = MessageId::from(id);

    // Check if message exists and user is the owner
    let existing_message = state.service.get_message(&message_id).await?;
    if existing_message.author_id.0 != user_identity.user_id {
        return Err(ApiError::Forbidden);
    }
    //FIXME: Check that user is authorized to update the message, not just is the author

    let input = request.into_input(message_id);
    let message = state.service.update_message(input).await?;
    Ok(Response::ok(message))
}

#[utoipa::path(
    delete,
    path = "/messages/{id}",
    tag = "messages",
    params(
        ("id" = String, Path, description = "Message ID")
    ),
    responses(
        (status = 200, description = "Message deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not the message owner"),
        (status = 404, description = "Message not found"),
        (status = 500, description = "Internal message error")
    )
)]
#[tracing::instrument(skip(state, user_identity))]
pub async fn delete_message(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<()>, ApiError> {
    let message_id = MessageId::from(id);

    // Check if message exists and user is the owner
    let existing_message = state.service.get_message(&message_id).await?;
    if existing_message.author_id.0 != user_identity.user_id {
        return Err(ApiError::Forbidden);
    }
    //FIXME: Check that user is authorized to update the message, not just is the author

    state.service.delete_message(&message_id).await?;
    Ok(Response::deleted(()))
}
