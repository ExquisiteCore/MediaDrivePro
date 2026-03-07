use chrono::Utc;
use mdp_auth::{jwt, password};
use mdp_common::error::AppError;
use sea_orm::*;
use uuid::Uuid;

use crate::entity::users;

/// Response DTO for user info (excludes password).
#[derive(Debug, serde::Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub avatar: Option<String>,
    pub storage_quota: i64,
    pub storage_used: i64,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<users::Model> for UserInfo {
    fn from(u: users::Model) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            role: u.role,
            avatar: u.avatar,
            storage_quota: u.storage_quota,
            storage_used: u.storage_used,
            created_at: u.created_at,
        }
    }
}

/// Token response after login/register.
#[derive(Debug, serde::Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub user: UserInfo,
}

pub struct UserService;

impl UserService {
    pub async fn register(
        db: &DatabaseConnection,
        username: &str,
        email: &str,
        raw_password: &str,
        jwt_secret: &str,
        token_ttl: u64,
    ) -> Result<TokenResponse, AppError> {
        // Validate input
        if username.len() < 2 || username.len() > 64 {
            return Err(AppError::Validation("用户名长度需在 2-64 之间".to_string()));
        }
        if !email.contains('@') {
            return Err(AppError::Validation("邮箱格式不正确".to_string()));
        }
        if raw_password.len() < 6 {
            return Err(AppError::Validation("密码长度至少 6 位".to_string()));
        }

        // Check duplicates
        let exists = users::Entity::find()
            .filter(
                Condition::any()
                    .add(users::Column::Username.eq(username))
                    .add(users::Column::Email.eq(email)),
            )
            .one(db)
            .await?;

        if exists.is_some() {
            return Err(AppError::Conflict("用户名或邮箱已存在".to_string()));
        }

        let hashed = password::hash_password(raw_password)?;
        let now = Utc::now();
        let user_id = Uuid::new_v4();

        // First registered user becomes admin
        let user_count = users::Entity::find().count(db).await?;
        let role = if user_count == 0 { "admin" } else { "user" };

        let user = users::ActiveModel {
            id: Set(user_id),
            username: Set(username.to_string()),
            email: Set(email.to_string()),
            password: Set(hashed),
            role: Set(role.to_string()),
            avatar: Set(None),
            storage_quota: Set(10_737_418_240), // 10GB
            storage_used: Set(0),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let user = user.insert(db).await?;
        let token = jwt::encode_token(user.id, &user.role, jwt_secret, token_ttl)?;

        Ok(TokenResponse {
            access_token: token,
            user: user.into(),
        })
    }

    pub async fn login(
        db: &DatabaseConnection,
        username: &str,
        raw_password: &str,
        jwt_secret: &str,
        token_ttl: u64,
    ) -> Result<TokenResponse, AppError> {
        let user = users::Entity::find()
            .filter(users::Column::Username.eq(username))
            .one(db)
            .await?
            .ok_or(AppError::Validation("用户名或密码错误".to_string()))?;

        let valid = password::verify_password(raw_password, &user.password)?;
        if !valid {
            return Err(AppError::Validation("用户名或密码错误".to_string()));
        }

        let token = jwt::encode_token(user.id, &user.role, jwt_secret, token_ttl)?;

        Ok(TokenResponse {
            access_token: token,
            user: user.into(),
        })
    }

    pub async fn update_avatar(
        db: &DatabaseConnection,
        user_id: Uuid,
        storage_key: &str,
    ) -> Result<(), AppError> {
        let result = users::Entity::update_many()
            .col_expr(
                users::Column::Avatar,
                sea_orm::sea_query::Expr::value(storage_key.to_string()),
            )
            .col_expr(
                users::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(Utc::now()),
            )
            .filter(users::Column::Id.eq(user_id))
            .exec(db)
            .await?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound("用户不存在".to_string()));
        }

        Ok(())
    }

    pub async fn get_by_id(db: &DatabaseConnection, id: Uuid) -> Result<UserInfo, AppError> {
        let user = users::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(AppError::NotFound("用户不存在".to_string()))?;

        Ok(user.into())
    }
}
