use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use mdp_auth::middleware::AuthUser;
use mdp_common::{error::AppError, response::ApiResponse};
use mdp_core::api_token::{ApiTokenCreated, ApiTokenInfo, ApiTokenService};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tokens", post(create_token).get(list_tokens))
        .route("/tokens/{id}", axum::routing::delete(delete_token))
}

#[derive(Deserialize)]
struct CreateTokenRequest {
    name: String,
    permissions: Option<String>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn create_token(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateTokenRequest>,
) -> Result<ApiResponse<ApiTokenCreated>, AppError> {
    let token = ApiTokenService::create(
        &state.db,
        auth.user_id,
        &req.name,
        req.permissions.as_deref(),
        req.expires_at,
    )
    .await?;
    Ok(ApiResponse::ok(token))
}

async fn list_tokens(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<ApiResponse<Vec<ApiTokenInfo>>, AppError> {
    let tokens = ApiTokenService::list_by_user(&state.db, auth.user_id).await?;
    Ok(ApiResponse::ok(tokens))
}

async fn delete_token(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    ApiTokenService::delete(&state.db, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
