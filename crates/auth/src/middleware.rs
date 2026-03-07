use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use chrono::Utc;
use mdp_common::{config::AppConfig, error::AppError};
use sea_orm::*;
use sha2::{Digest, Sha256};
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
    DatabaseConnection: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = AppConfig::from_ref(state);

        // Try Authorization header first, then ?token= query param (for browser embeds)
        let token_result = if let Some(auth_header) = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
        {
            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or(AppError::Unauthorized)?
                .to_string();
            Some(token)
        } else if let Some(query) = parts.uri.query() {
            query
                .split('&')
                .find_map(|pair| pair.strip_prefix("token="))
                .map(|t| urlencoding::decode(t).unwrap_or_default().into_owned())
        } else {
            None
        };

        // If we got a JWT token, try to decode it
        if let Some(token) = token_result {
            let claims = jwt::decode_token(&token, &config.auth.jwt_secret)?;
            return Ok(AuthUser {
                user_id: claims.sub,
                role: claims.role,
            });
        }

        // Try X-API-Token header (format: "username:token")
        if let Some(api_token_header) = parts
            .headers
            .get("X-API-Token")
            .and_then(|v| v.to_str().ok())
        {
            let db = DatabaseConnection::from_ref(state);
            return verify_api_token(&db, api_token_header).await;
        }

        Err(AppError::Unauthorized)
    }
}

/// Verify API token directly (avoids circular dependency with mdp-core).
/// Token format: "username:plaintext_token"
async fn verify_api_token(db: &DatabaseConnection, header_value: &str) -> Result<AuthUser, AppError> {
    let (username, plaintext) = header_value
        .split_once(':')
        .ok_or(AppError::Unauthorized)?;

    if username.is_empty() || plaintext.is_empty() {
        return Err(AppError::Unauthorized);
    }

    // Find user by username
    let user_row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT id, role FROM users WHERE username = $1",
            [username.into()],
        ))
        .await
        .map_err(|_| AppError::Unauthorized)?
        .ok_or(AppError::Unauthorized)?;

    let user_id: Uuid = user_row
        .try_get_by_index(0)
        .map_err(|_| AppError::Unauthorized)?;
    let role: String = user_row
        .try_get_by_index(1)
        .map_err(|_| AppError::Unauthorized)?;

    // Hash the token and find matching api_tokens record
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    let token_row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT id, expires_at FROM api_tokens WHERE user_id = $1 AND token_hash = $2",
            [user_id.into(), token_hash.into()],
        ))
        .await
        .map_err(|_| AppError::Unauthorized)?
        .ok_or(AppError::Unauthorized)?;

    let token_id: Uuid = token_row
        .try_get_by_index(0)
        .map_err(|_| AppError::Unauthorized)?;

    // Check expiration
    let expires_at: Option<chrono::DateTime<Utc>> = token_row
        .try_get_by_index(1)
        .ok();

    if let Some(Some(exp)) = expires_at.map(Some) {
        if Utc::now() > exp {
            return Err(AppError::Unauthorized);
        }
    }

    // Update last_used_at
    let _ = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "UPDATE api_tokens SET last_used_at = $1 WHERE id = $2",
            [Utc::now().into(), token_id.into()],
        ))
        .await;

    Ok(AuthUser { user_id, role })
}
