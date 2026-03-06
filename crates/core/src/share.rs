use chrono::Utc;
use mdp_common::error::AppError;
use opendal::Operator;
use sea_orm::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::entity::{files, shares};

#[derive(Debug, serde::Serialize)]
pub struct ShareInfo {
    pub id: Uuid,
    pub file_id: Option<Uuid>,
    pub folder_id: Option<Uuid>,
    pub token: String,
    pub has_password: bool,
    pub permission: String,
    pub max_downloads: Option<i32>,
    pub download_count: i32,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<shares::Model> for ShareInfo {
    fn from(s: shares::Model) -> Self {
        Self {
            id: s.id,
            file_id: s.file_id,
            folder_id: s.folder_id,
            token: s.token,
            has_password: s.password.is_some(),
            permission: s.permission,
            max_downloads: s.max_downloads,
            download_count: s.download_count,
            expires_at: s.expires_at,
            created_at: s.created_at,
        }
    }
}

/// Public share info returned to unauthenticated users.
#[derive(Debug, serde::Serialize)]
pub struct PublicShareInfo {
    pub token: String,
    pub has_password: bool,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

pub struct ShareService;

impl ShareService {
    pub async fn create(
        db: &DatabaseConnection,
        user_id: Uuid,
        file_id: Option<Uuid>,
        folder_id: Option<Uuid>,
        password: Option<&str>,
        max_downloads: Option<i32>,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<ShareInfo, AppError> {
        if file_id.is_none() && folder_id.is_none() {
            return Err(AppError::Validation(
                "必须指定 file_id 或 folder_id".to_string(),
            ));
        }

        // Verify ownership
        if let Some(fid) = file_id {
            files::Entity::find_by_id(fid)
                .filter(files::Column::UserId.eq(user_id))
                .filter(files::Column::Status.ne("deleted"))
                .one(db)
                .await?
                .ok_or(AppError::NotFound("文件不存在".to_string()))?;
        }
        if let Some(fid) = folder_id {
            use crate::entity::folders;
            folders::Entity::find_by_id(fid)
                .filter(folders::Column::UserId.eq(user_id))
                .one(db)
                .await?
                .ok_or(AppError::NotFound("目录不存在".to_string()))?;
        }

        // Generate a short token
        let token = generate_token();

        // Hash password if provided
        let password_hash = password.map(|p| {
            let mut hasher = Sha256::new();
            hasher.update(p.as_bytes());
            hex::encode(hasher.finalize())
        });

        let now = Utc::now();
        let share = shares::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            file_id: Set(file_id),
            folder_id: Set(folder_id),
            token: Set(token),
            password: Set(password_hash),
            permission: Set("read".to_string()),
            max_downloads: Set(max_downloads),
            download_count: Set(0),
            expires_at: Set(expires_at),
            created_at: Set(now),
        };

        let share = share.insert(db).await?;
        Ok(share.into())
    }

    pub async fn list_by_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<ShareInfo>, AppError> {
        let shares = shares::Entity::find()
            .filter(shares::Column::UserId.eq(user_id))
            .order_by_desc(shares::Column::CreatedAt)
            .all(db)
            .await?
            .into_iter()
            .map(ShareInfo::from)
            .collect();

        Ok(shares)
    }

    pub async fn delete(
        db: &DatabaseConnection,
        user_id: Uuid,
        share_id: Uuid,
    ) -> Result<(), AppError> {
        let share = shares::Entity::find_by_id(share_id)
            .filter(shares::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("分享不存在".to_string()))?;

        share.delete(db).await?;
        Ok(())
    }

    /// Get public share info by token (no auth required).
    pub async fn get_by_token(
        db: &DatabaseConnection,
        token: &str,
    ) -> Result<(shares::Model, PublicShareInfo), AppError> {
        let share = shares::Entity::find()
            .filter(shares::Column::Token.eq(token))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("分享不存在或已过期".to_string()))?;

        // Check expiration
        if let Some(expires_at) = share.expires_at {
            if Utc::now() > expires_at {
                return Err(AppError::NotFound("分享已过期".to_string()));
            }
        }

        // Check download limit
        if let Some(max) = share.max_downloads {
            if share.download_count >= max {
                return Err(AppError::NotFound("分享下载次数已达上限".to_string()));
            }
        }

        // Get file info if it's a file share
        let (file_name, file_size) = if let Some(fid) = share.file_id {
            let file = files::Entity::find_by_id(fid)
                .filter(files::Column::Status.ne("deleted"))
                .one(db)
                .await?;
            match file {
                Some(f) => (Some(f.name), Some(f.size)),
                None => return Err(AppError::NotFound("分享的文件已被删除".to_string())),
            }
        } else {
            (None, None)
        };

        let info = PublicShareInfo {
            token: share.token.clone(),
            has_password: share.password.is_some(),
            file_name,
            file_size,
            expires_at: share.expires_at,
        };

        Ok((share, info))
    }

    /// Verify share password. Returns true if no password or password matches.
    pub fn verify_password(share: &shares::Model, password: &str) -> bool {
        match &share.password {
            None => true,
            Some(hash) => {
                let mut hasher = Sha256::new();
                hasher.update(password.as_bytes());
                let input_hash = hex::encode(hasher.finalize());
                input_hash == *hash
            }
        }
    }

    /// Download file through share link. Increments download count.
    pub async fn download(
        db: &DatabaseConnection,
        storage: &Operator,
        share: shares::Model,
    ) -> Result<(files::Model, Vec<u8>), AppError> {
        let file_id = share
            .file_id
            .ok_or(AppError::Validation("该分享不是文件分享".to_string()))?;

        let file = files::Entity::find_by_id(file_id)
            .filter(files::Column::Status.ne("deleted"))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("文件不存在".to_string()))?;

        let data = storage
            .read(&file.storage_key)
            .await
            .map_err(AppError::Storage)?
            .to_vec();

        // Increment download count
        let mut active: shares::ActiveModel = share.into();
        let count = active.download_count.clone().unwrap();
        active.download_count = Set(count + 1);
        active.update(db).await?;

        Ok((file, data))
    }
}

fn generate_token() -> String {
    let id = Uuid::new_v4();
    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    let hash = hex::encode(hasher.finalize());
    hash[..16].to_string()
}
