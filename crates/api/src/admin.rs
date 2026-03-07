use axum::{Router, extract::State, routing::get};
use mdp_auth::middleware::AuthUser;
use mdp_common::{error::AppError, response::ApiResponse};
use mdp_core::user::UserInfo;
use sea_orm::EntityTrait;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/admin/users", get(list_users))
}

async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<ApiResponse<Vec<UserInfo>>, AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let users: Vec<UserInfo> = mdp_core::entity::users::Entity::find()
        .all(&state.db)
        .await?
        .into_iter()
        .map(UserInfo::from)
        .collect();

    Ok(ApiResponse::ok(users))
}
