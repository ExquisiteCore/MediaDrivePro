use axum::{
    Router,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use mdp_auth::middleware::AuthUser;
use mdp_common::{error::AppError, response::ApiResponse};
use mdp_core::image::{ImageInfo, ImageService};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

/// Routes under /api/v1 (require auth)
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/images",
            post(upload).layer(DefaultBodyLimit::max(20 * 1024 * 1024)),
        )
        .route("/images", get(list))
        .route("/images/{id}", axum::routing::delete(delete))
}

/// Public routes (outside /api/v1)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/img/{hash}", get(serve_image))
        .route("/img/thumb/{hash}", get(serve_thumb))
}

async fn upload(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<ApiResponse<ImageInfo>, AppError> {
    let mut file_name = None;
    let mut content_type = None;
    let mut data = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" || name == "image" {
            file_name = field.file_name().map(|s| s.to_string());
            content_type = field.content_type().map(|s| s.to_string());
            data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("读取文件失败: {e}")))?
                    .to_vec(),
            );
        }
    }

    let data = data.ok_or(AppError::Validation("缺少图片文件".to_string()))?;
    let file_name = file_name.unwrap_or_else(|| "image.jpg".to_string());
    let content_type = content_type.unwrap_or_else(|| "image/jpeg".to_string());

    let info = ImageService::upload(
        &state.db,
        &state.storage,
        auth.user_id,
        &file_name,
        &content_type,
        data,
        &state.config.image,
    )
    .await?;

    Ok(ApiResponse::ok(info))
}

#[derive(Deserialize)]
struct ListQuery {
    page: Option<u64>,
    per_page: Option<u64>,
}

async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<Vec<ImageInfo>>, AppError> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let (items, total) =
        ImageService::list(&state.db, auth.user_id, &state.config.image.cdn_base_url, page, per_page)
            .await?;
    Ok(ApiResponse::paginated(items, page, per_page, total))
}

async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    ImageService::delete(&state.db, &state.storage, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn serve_image(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(hash): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    check_referer(&headers, &state.config.image.allowed_referers)?;

    let hash = hash.trim_end_matches(".webp");
    let img = ImageService::get_by_hash(&state.db, hash).await?;

    let data = state
        .storage
        .read(&img.storage_key)
        .await
        .map_err(|e| AppError::NotFound(format!("图片文件不存在: {e}")))?
        .to_vec();

    let headers = [
        (header::CONTENT_TYPE, "image/webp".to_string()),
        (
            header::CACHE_CONTROL,
            "public, max-age=31536000, immutable".to_string(),
        ),
    ];

    Ok((StatusCode::OK, headers, data))
}

async fn serve_thumb(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(hash): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    check_referer(&headers, &state.config.image.allowed_referers)?;

    let hash = hash.trim_end_matches(".webp");
    let img = ImageService::get_by_hash(&state.db, hash).await?;

    let data = state
        .storage
        .read(&img.thumb_key)
        .await
        .map_err(|e| AppError::NotFound(format!("缩略图不存在: {e}")))?
        .to_vec();

    let headers = [
        (header::CONTENT_TYPE, "image/webp".to_string()),
        (
            header::CACHE_CONTROL,
            "public, max-age=31536000, immutable".to_string(),
        ),
    ];

    Ok((StatusCode::OK, headers, data))
}

fn check_referer(headers: &HeaderMap, allowed: &[String]) -> Result<(), AppError> {
    if allowed.is_empty() {
        return Ok(());
    }

    let referer = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if referer.is_empty() {
        // Allow direct access (no referer)
        return Ok(());
    }

    for allowed_domain in allowed {
        if referer.contains(allowed_domain) {
            return Ok(());
        }
    }

    Err(AppError::Forbidden)
}
