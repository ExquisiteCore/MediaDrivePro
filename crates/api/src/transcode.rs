use axum::{
    Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use mdp_auth::middleware::AuthUser;
use mdp_common::{error::AppError, response::ApiResponse};
use mdp_core::transcode::{TranscodeService, TranscodeTaskInfo};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/transcode", post(create).get(list))
        .route("/transcode/{id}", get(get_task))
}

#[derive(Deserialize)]
struct CreateRequest {
    file_id: Uuid,
    profile: Option<String>,
}

async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::Json(req): axum::Json<CreateRequest>,
) -> Result<ApiResponse<TranscodeTaskInfo>, AppError> {
    let profile = req.profile.as_deref().unwrap_or("720p");
    let info = TranscodeService::create(&state.db, auth.user_id, req.file_id, profile).await?;
    Ok(ApiResponse::ok(info))
}

async fn get_task(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<TranscodeTaskInfo>, AppError> {
    let info = TranscodeService::get(&state.db, id).await?;
    Ok(ApiResponse::ok(info))
}

#[derive(Deserialize)]
struct ListQuery {
    file_id: Uuid,
}

async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<Vec<TranscodeTaskInfo>>, AppError> {
    let items = TranscodeService::list_by_file(&state.db, query.file_id).await?;
    Ok(ApiResponse::ok(items))
}
