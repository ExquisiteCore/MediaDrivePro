use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use mdp_auth::middleware::AuthUser;
use mdp_common::{error::AppError, response::ApiResponse};
use mdp_core::folder::{FolderChildren, FolderInfo, FolderService};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/folders", post(create).get(list_root))
        .route("/folders/{id}", get(get_folder).put(update_folder).delete(delete_folder))
        .route("/folders/{id}/children", get(list_children))
}

#[derive(Deserialize)]
struct CreateFolderRequest {
    name: String,
    parent_id: Option<Uuid>,
}

async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateFolderRequest>,
) -> Result<ApiResponse<FolderInfo>, AppError> {
    let folder =
        FolderService::create(&state.db, auth.user_id, req.parent_id, &req.name).await?;
    Ok(ApiResponse::ok(folder))
}

async fn get_folder(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<FolderInfo>, AppError> {
    let folder = FolderService::get_by_id(&state.db, auth.user_id, id).await?;
    Ok(ApiResponse::ok(folder))
}

async fn list_root(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<ApiResponse<FolderChildren>, AppError> {
    let children = FolderService::list_children(&state.db, auth.user_id, None).await?;
    Ok(ApiResponse::ok(children))
}

async fn list_children(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<FolderChildren>, AppError> {
    let children = FolderService::list_children(&state.db, auth.user_id, Some(id)).await?;
    Ok(ApiResponse::ok(children))
}

async fn delete_folder(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    FolderService::delete(&state.db, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct UpdateFolderRequest {
    name: Option<String>,
    parent_id: Option<Option<Uuid>>,
}

async fn update_folder(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateFolderRequest>,
) -> Result<ApiResponse<FolderInfo>, AppError> {
    let folder = FolderService::rename_move(
        &state.db,
        auth.user_id,
        id,
        req.name.as_deref(),
        req.parent_id,
    )
    .await?;
    Ok(ApiResponse::ok(folder))
}
