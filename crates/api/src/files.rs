use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use mdp_auth::middleware::AuthUser;
use mdp_common::{error::AppError, response::ApiResponse};
use mdp_core::file::{FileInfo, FileListQuery, FileService};
use uuid::Uuid;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/files", post(upload).get(list))
        .route("/files/{id}", get(get_file).delete(delete_file))
        .route("/files/{id}/download", get(download))
}

async fn upload(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<ApiResponse<FileInfo>, AppError> {
    let mut file_name = None;
    let mut content_type = None;
    let mut data = None;
    let mut folder_id: Option<Uuid> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());
                data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::Validation(format!("读取文件失败: {e}")))?,
                );
            }
            "folder_id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("读取 folder_id 失败: {e}")))?;
                if !text.is_empty() {
                    folder_id = Some(
                        text.parse::<Uuid>()
                            .map_err(|_| AppError::Validation("folder_id 格式错误".to_string()))?,
                    );
                }
            }
            _ => {}
        }
    }

    let data = data.ok_or(AppError::Validation("缺少文件".to_string()))?;
    let file_name = file_name.unwrap_or_else(|| "unnamed".to_string());
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

    let info = FileService::upload(
        &state.db,
        &state.storage,
        auth.user_id,
        folder_id,
        &file_name,
        &content_type,
        data.to_vec(),
        &state.config.storage.backend,
    )
    .await?;

    Ok(ApiResponse::ok(info))
}

async fn get_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<FileInfo>, AppError> {
    let info = FileService::get_by_id(&state.db, auth.user_id, id).await?;
    Ok(ApiResponse::ok(info))
}

async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<FileListQuery>,
) -> Result<ApiResponse<Vec<FileInfo>>, AppError> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let (items, total) = FileService::list(&state.db, auth.user_id, &query).await?;
    Ok(ApiResponse::paginated(items, page, per_page, total))
}

async fn download(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let (file, data) = FileService::download(&state.db, &state.storage, auth.user_id, id).await?;

    let headers = [
        (header::CONTENT_TYPE, file.content_type),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file.name),
        ),
    ];

    Ok((StatusCode::OK, headers, data))
}

async fn delete_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    FileService::delete(&state.db, &state.storage, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
