use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use mdp_auth::middleware::AuthUser;
use mdp_common::{error::AppError, response::ApiResponse};
use mdp_core::share::{PublicShareInfo, ShareInfo, ShareService};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

/// Authenticated share management routes: /api/v1/shares
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/shares", post(create_share).get(list_shares))
        .route("/shares/{id}", axum::routing::delete(delete_share))
}

/// Public share access routes (no auth required)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/shares/public/{token}", get(get_share))
        .route("/shares/public/{token}/verify", post(verify_share))
        .route("/shares/public/{token}/download", get(download_share))
}

#[derive(Deserialize)]
struct CreateShareRequest {
    file_id: Option<Uuid>,
    folder_id: Option<Uuid>,
    password: Option<String>,
    max_downloads: Option<i32>,
    /// ISO 8601 datetime
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn create_share(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateShareRequest>,
) -> Result<ApiResponse<ShareInfo>, AppError> {
    let share = ShareService::create(
        &state.db,
        auth.user_id,
        req.file_id,
        req.folder_id,
        req.password.as_deref(),
        req.max_downloads,
        req.expires_at,
    )
    .await?;
    Ok(ApiResponse::ok(share))
}

async fn list_shares(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<ApiResponse<Vec<ShareInfo>>, AppError> {
    let shares = ShareService::list_by_user(&state.db, auth.user_id).await?;
    Ok(ApiResponse::ok(shares))
}

async fn delete_share(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    ShareService::delete(&state.db, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_share(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<ApiResponse<PublicShareInfo>, AppError> {
    let (_, info) = ShareService::get_by_token(&state.db, &token).await?;
    Ok(ApiResponse::ok(info))
}

#[derive(Deserialize)]
struct VerifyRequest {
    password: String,
}

async fn verify_share(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(req): Json<VerifyRequest>,
) -> Result<ApiResponse<PublicShareInfo>, AppError> {
    let (share, info) = ShareService::get_by_token(&state.db, &token).await?;
    if !ShareService::verify_password(&share, &req.password) {
        return Err(AppError::Forbidden);
    }
    Ok(ApiResponse::ok(info))
}

async fn download_share(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let (share, _) = ShareService::get_by_token(&state.db, &token).await?;

    // If share has a password, require it via query param or deny
    if share.password.is_some() {
        return Err(AppError::Forbidden);
    }

    let (file, data) = ShareService::download(&state.db, &state.storage, share).await?;

    let headers = [
        (header::CONTENT_TYPE, file.content_type),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file.name),
        ),
    ];

    Ok((StatusCode::OK, headers, data))
}
