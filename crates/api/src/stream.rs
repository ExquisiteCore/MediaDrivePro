use axum::{
    Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use mdp_auth::middleware::AuthUser;
use mdp_common::error::AppError;
use mdp_storage::operator::storage_key;
use uuid::Uuid;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/files/{file_id}/stream/{*path}",
        get(stream),
    )
}

async fn stream(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path((file_id, path)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    let storage_path = format!("{}/{}", storage_key::transcode_dir(file_id), path);

    let data = state
        .storage
        .read(&storage_path)
        .await
        .map_err(|e| AppError::NotFound(format!("流文件不存在: {e}")))?
        .to_vec();

    let content_type = if path.ends_with(".m3u8") {
        "application/vnd.apple.mpegurl"
    } else if path.ends_with(".ts") {
        "video/mp2t"
    } else if path.ends_with(".vtt") {
        "text/vtt"
    } else {
        "application/octet-stream"
    };

    let headers = [
        (header::CONTENT_TYPE, content_type.to_string()),
        (
            header::CACHE_CONTROL,
            "public, max-age=3600".to_string(),
        ),
    ];

    Ok((StatusCode::OK, headers, data))
}
