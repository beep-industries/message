use axum::{extract::FromRequestParts, http::request::Parts};
use beep_auth::{AuthRepository, KeycloakAuthRepository};
use uuid::Uuid;

use crate::http::server::ApiError;
pub mod entities;

pub struct AuthMiddleware;

impl FromRequestParts<KeycloakAuthRepository> for AuthMiddleware {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &KeycloakAuthRepository,
    ) -> Result<Self, Self::Rejection> {
        tracing::debug!(
            "Authentication middleware: checking request to {}",
            parts.uri
        );

        // Extract the Authorization header
        let auth_header = parts.headers.get(axum::http::header::AUTHORIZATION);

        if auth_header.is_none() {
            tracing::warn!("Authentication failed: Authorization header missing");
            return Err(ApiError::Unauthorized);
        }

        // Ensure the header exists and starts with "Bearer "
        let auth_value = auth_header.unwrap().to_str().map_err(|e| {
            tracing::warn!(
                "Authentication failed: Authorization header is not valid UTF-8: {}",
                e
            );
            ApiError::Unauthorized
        })?;

        tracing::debug!("Authorization header present, checking Bearer prefix");

        let token = auth_value.strip_prefix("Bearer ").ok_or_else(|| {
            tracing::warn!("Authentication failed: Authorization header doesn't start with 'Bearer '. Header value starts with: {:?}", &auth_value.chars().take(10).collect::<String>());
            ApiError::Unauthorized
        })?;

        tracing::debug!(
            "Token extracted, length: {} chars, validating with Keycloak",
            token.len()
        );

        // Validate the token
        let keycloak_identity = state.identify(token).await.map_err(|e| {
            tracing::warn!(
                "Authentication failed: Keycloak token validation failed: {:?}",
                e
            );
            ApiError::Unauthorized
        })?;

        let user_id_str = keycloak_identity.id();
        tracing::debug!("Keycloak validation successful, user ID: {}", user_id_str);

        let user_id = Uuid::try_parse(user_id_str).map_err(|e| {
            tracing::error!(
                "Authentication failed: Invalid UUID format from Keycloak: '{}', error: {}",
                user_id_str,
                e
            );
            ApiError::Unauthorized
        })?;

        let user_identity = entities::UserIdentity { user_id };

        tracing::debug!("Authentication successful for user: {}", user_id);

        // Add auth state to request
        parts.extensions.insert(user_identity);
        Ok(Self)
    }
}
