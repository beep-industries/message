use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{AppState, http::attachments::handlers::{__path_create_attachment, create_attachment}};

pub fn attachment_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(create_attachment))
}
