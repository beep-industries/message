use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::messages::handlers::{
        __path_create_message, __path_delete_message, __path_get_message, __path_list_messages,
        __path_update_message, create_message, delete_message, get_message, list_messages,
        update_message,
    },
    http::server::AppState,
};

pub fn message_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create_message))
        .routes(routes!(get_message))
        .routes(routes!(list_messages))
        .routes(routes!(update_message))
        .routes(routes!(delete_message))
}
