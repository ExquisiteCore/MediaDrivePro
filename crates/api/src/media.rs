use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use mdp_auth::middleware::AuthUser;
use mdp_common::{error::AppError, response::ApiResponse};
use mdp_core::media::{MediaInfoDto, MediaService};
use uuid::Uuid;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/media/{file_id}", get(get_media))
        .route("/media/{file_id}/scan", post(scan_media))
}

async fn get_media(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(file_id): Path<Uuid>,
) -> Result<ApiResponse<Option<MediaInfoDto>>, AppError> {
    let info = MediaService::get_by_file(&state.db, file_id).await?;
    Ok(ApiResponse::ok(info))
}

async fn scan_media(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(file_id): Path<Uuid>,
) -> Result<(StatusCode, ApiResponse<MediaInfoDto>), AppError> {
    // Get the file to use its name for parsing
    use mdp_core::entity::files;
    use sea_orm::*;

    let file = files::Entity::find_by_id(file_id)
        .filter(files::Column::Status.ne("deleted"))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("文件不存在".to_string()))?;

    let info =
        MediaService::scan(&state.db, &state.config.tmdb, &file.name, file_id).await?;
    Ok((StatusCode::OK, ApiResponse::ok(info)))
}
