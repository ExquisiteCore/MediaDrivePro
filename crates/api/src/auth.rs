use axum::{extract::State, routing::{get, post}, Json, Router};
use mdp_auth::middleware::AuthUser;
use mdp_common::{error::AppError, response::ApiResponse};
use mdp_core::user::{TokenResponse, UserService};
use serde::Deserialize;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
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
) -> Result<ApiResponse<mdp_core::user::UserInfo>, AppError> {
    let user = UserService::get_by_id(&state.db, auth.user_id).await?;
    Ok(ApiResponse::ok(user))
}
