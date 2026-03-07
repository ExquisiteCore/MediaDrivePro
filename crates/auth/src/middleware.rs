use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use mdp_common::{config::AppConfig, error::AppError};
use uuid::Uuid;

use crate::jwt;

/// Authenticated user extracted from the Authorization header.
/// Use this as an extractor in Axum handlers to require authentication.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    AppConfig: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = AppConfig::from_ref(state);

        // Try Authorization header first, then ?token= query param (for browser embeds)
        let token = if let Some(auth_header) = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
        {
            auth_header
                .strip_prefix("Bearer ")
                .ok_or(AppError::Unauthorized)?
                .to_string()
        } else if let Some(query) = parts.uri.query() {
            query
                .split('&')
                .find_map(|pair| pair.strip_prefix("token="))
                .map(|t| urlencoding::decode(t).unwrap_or_default().into_owned())
                .ok_or(AppError::Unauthorized)?
        } else {
            return Err(AppError::Unauthorized);
        };

        let claims = jwt::decode_token(&token, &config.auth.jwt_secret)?;

        Ok(AuthUser {
            user_id: claims.sub,
            role: claims.role,
        })
    }
}
