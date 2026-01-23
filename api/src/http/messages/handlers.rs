use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use communities_core::domain::{
    common::GetPaginated,
    message::{
        entities::{AuthorId, ChannelId, CreateMessageRequest, Message, MessageId, UpdateMessageRequest},
        ports::MessageService,
    },
};
use uuid::Uuid;

use crate::http::server::{
    ApiError, AppState, Response, middleware::auth::entities::UserIdentity,
    response::PaginatedResponse,
};
use crate::http::server::authorization::{Permission, Resource};

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
    // Authorization: check user can send messages to this channel
    let channel = request.channel_id;
    let allowed = state
        .authz
        .check(user_identity.user_id, Permission::SendMessages, Resource::Channel(channel.0))
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    if !allowed {
        return Err(ApiError::Forbidden);
    }

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
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<Message>, ApiError> {
    let message_id = MessageId::from(id);
    let message = state.service.get_message(&message_id).await?;

    // Authorization: check user can view the channel where this message belongs
    let allowed = state
        .authz
        .check(user_identity.user_id, Permission::ViewChannels, Resource::Channel(message.channel_id.0))
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    if !allowed {
        return Err(ApiError::Forbidden);
    }

    Ok(Response::ok(message))
}

#[utoipa::path(
    get,
    path = "/channels/{channel_id}/messages",
    tag = "messages",
    params(
        ("channel_id" = String, Path, description = "Channel ID"),
        GetPaginated
    ),
    responses(
        (status = 200, description = "List of messages retrieved successfully", body = PaginatedResponse<Message>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal message error")
    )
)]
#[tracing::instrument(skip(state, user_identity, pagination))]
pub async fn list_messages(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Path(channel_id): Path<Uuid>,
    Query(pagination): Query<GetPaginated>,
) -> Result<Response<PaginatedResponse<Message>>, ApiError> {
    let channel = ChannelId::from(channel_id);

    // Authorization: ensure user can view the channel before listing
    let allowed = state
        .authz
        .check(user_identity.user_id, Permission::ViewChannels, Resource::Channel(channel.0))
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    if !allowed {
        return Err(ApiError::Forbidden);
    }

    let (messages, total) = state.service.list_messages(&channel, &pagination).await?;

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

    state.service.delete_message(&message_id).await?;
    Ok(Response::deleted(()))
}
