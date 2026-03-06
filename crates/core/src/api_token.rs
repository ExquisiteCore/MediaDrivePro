use chrono::Utc;
use mdp_common::error::AppError;
use rand::Rng;
use sea_orm::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::entity::api_tokens;

#[derive(Debug, serde::Serialize)]
pub struct ApiTokenInfo {
    pub id: Uuid,
    pub name: String,
    pub permissions: String,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub last_used_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<api_tokens::Model> for ApiTokenInfo {
    fn from(t: api_tokens::Model) -> Self {
        Self {
            id: t.id,
            name: t.name,
            permissions: t.permissions,
            expires_at: t.expires_at,
            last_used_at: t.last_used_at,
            created_at: t.created_at,
        }
    }
}

/// Returned only on creation, includes the plaintext token.
#[derive(Debug, serde::Serialize)]
pub struct ApiTokenCreated {
    #[serde(flatten)]
    pub info: ApiTokenInfo,
    pub token: String,
}

pub struct ApiTokenService;

impl ApiTokenService {
    /// Create a new API token. Returns the plaintext token (only shown once).
    pub async fn create(
        db: &DatabaseConnection,
        user_id: Uuid,
        name: &str,
        permissions: Option<&str>,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<ApiTokenCreated, AppError> {
        if name.is_empty() || name.len() > 64 {
            return Err(AppError::Validation(
                "Token 名称长度需在 1-64 之间".to_string(),
            ));
        }

        let plaintext = generate_random_token();
        let token_hash = hash_token(&plaintext);

        let now = Utc::now();
        let token = api_tokens::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            name: Set(name.to_string()),
            token_hash: Set(token_hash),
            permissions: Set(permissions.unwrap_or("read,write").to_string()),
            expires_at: Set(expires_at),
            last_used_at: Set(None),
            created_at: Set(now),
        };

        let model = token.insert(db).await?;
        Ok(ApiTokenCreated {
            info: model.into(),
            token: plaintext,
        })
    }

    /// List all tokens for a user.
    pub async fn list_by_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<ApiTokenInfo>, AppError> {
        let tokens = api_tokens::Entity::find()
            .filter(api_tokens::Column::UserId.eq(user_id))
            .order_by_desc(api_tokens::Column::CreatedAt)
            .all(db)
            .await?
            .into_iter()
            .map(ApiTokenInfo::from)
            .collect();
        Ok(tokens)
    }

    /// Delete a token by ID (must belong to user).
    pub async fn delete(
        db: &DatabaseConnection,
        user_id: Uuid,
        token_id: Uuid,
    ) -> Result<(), AppError> {
        let token = api_tokens::Entity::find_by_id(token_id)
            .filter(api_tokens::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("Token 不存在".to_string()))?;
        token.delete(db).await?;
        Ok(())
    }

    /// Verify an API token by plaintext. Returns (user_id, permissions) if valid.
    /// Also updates last_used_at.
    pub async fn verify(
        db: &DatabaseConnection,
        username: &str,
        plaintext_token: &str,
    ) -> Result<(Uuid, String), AppError> {
        use crate::entity::users;

        // Find user by username
        let user = users::Entity::find()
            .filter(users::Column::Username.eq(username))
            .one(db)
            .await?
            .ok_or(AppError::Unauthorized)?;

        // Hash the provided token and find matching record
        let token_hash = hash_token(plaintext_token);
        let token = api_tokens::Entity::find()
            .filter(api_tokens::Column::UserId.eq(user.id))
            .filter(api_tokens::Column::TokenHash.eq(token_hash))
            .one(db)
            .await?
            .ok_or(AppError::Unauthorized)?;

        // Check expiration
        if let Some(expires_at) = token.expires_at {
            if Utc::now() > expires_at {
                return Err(AppError::Unauthorized);
            }
        }

        // Update last_used_at
        let permissions = token.permissions.clone();
        let mut active: api_tokens::ActiveModel = token.into();
        active.last_used_at = Set(Some(Utc::now()));
        active.update(db).await?;

        Ok((user.id, permissions))
    }
}

fn generate_random_token() -> String {
    let mut rng = rand::rng();
    let bytes: [u8; 32] = rng.random();
    hex::encode(bytes)
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
