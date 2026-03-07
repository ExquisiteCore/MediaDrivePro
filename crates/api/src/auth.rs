use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use mdp_auth::middleware::AuthUser;
use mdp_common::{error::AppError, response::ApiResponse};
use mdp_core::user::{TokenResponse, UserInfo, UserService};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
        .route(
            "/auth/avatar",
            post(upload_avatar).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route("/users/{id}/avatar", get(get_avatar))
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<ApiResponse<TokenResponse>, AppError> {
    let result = UserService::register(
        &state.db,
        &req.username,
        &req.email,
        &req.password,
        &state.config.auth.jwt_secret,
        state.config.auth.access_token_ttl_secs,
    )
    .await?;

    Ok(ApiResponse::ok(result))
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<ApiResponse<TokenResponse>, AppError> {
    let result = UserService::login(
        &state.db,
        &req.username,
        &req.password,
        &state.config.auth.jwt_secret,
        state.config.auth.access_token_ttl_secs,
    )
    .await?;

    Ok(ApiResponse::ok(result))
}

async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<ApiResponse<UserInfo>, AppError> {
    let user = UserService::get_by_id(&state.db, auth.user_id).await?;
    Ok(ApiResponse::ok(user))
}

async fn upload_avatar(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<ApiResponse<UserInfo>, AppError> {
    let mut data = None;
    let mut content_type = None;
    let mut ext = "jpg".to_string();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "avatar" {
            content_type = field.content_type().map(|s| s.to_string());
            if let Some(fname) = field.file_name().map(|s| s.to_string()) {
                if let Some(e) = fname.rsplit('.').next() {
                    ext = e.to_lowercase();
                }
            }
            data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("读取文件失败: {e}")))?,
            );
        }
    }

    let data = data.ok_or(AppError::Validation("缺少头像文件".to_string()))?;
    let ct = content_type.unwrap_or_else(|| "image/jpeg".to_string());

    if !ct.starts_with("image/") {
        return Err(AppError::Validation("只能上传图片文件".to_string()));
    }

    let storage_key = format!("avatars/{}.{}", auth.user_id, ext);
    state
        .storage
        .write(&storage_key, data.to_vec())
        .await
        .map_err(|e| AppError::Internal(format!("存储头像失败: {e}")))?;

    UserService::update_avatar(&state.db, auth.user_id, &storage_key).await?;

    let user = UserService::get_by_id(&state.db, auth.user_id).await?;
    Ok(ApiResponse::ok(user))
}

async fn get_avatar(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user = UserService::get_by_id(&state.db, id).await?;
    let key = user
        .avatar
        .ok_or(AppError::NotFound("用户未设置头像".to_string()))?;

    let data = state
        .storage
        .read(&key)
        .await
        .map_err(|e| AppError::NotFound(format!("头像文件不存在: {e}")))?
        .to_vec();

    // Guess content type from extension
    let ct = if key.ends_with(".png") {
        "image/png"
    } else if key.ends_with(".gif") {
        "image/gif"
    } else if key.ends_with(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    };

    let headers = [
        (header::CONTENT_TYPE, ct.to_string()),
        (
            header::CACHE_CONTROL,
            "public, max-age=3600".to_string(),
        ),
    ];

    Ok((StatusCode::OK, headers, data))
}
