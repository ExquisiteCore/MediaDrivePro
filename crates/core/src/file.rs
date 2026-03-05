use chrono::Utc;
use mdp_common::error::AppError;
use opendal::Operator;
use sea_orm::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::entity::files;

#[derive(Debug, serde::Serialize)]
pub struct FileInfo {
    pub id: Uuid,
    pub name: String,
    pub size: i64,
    pub content_type: String,
    pub folder_id: Option<Uuid>,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl From<files::Model> for FileInfo {
    fn from(f: files::Model) -> Self {
        Self {
            id: f.id,
            name: f.name,
            size: f.size,
            content_type: f.content_type,
            folder_id: f.folder_id,
            status: f.status,
            created_at: f.created_at,
            updated_at: f.updated_at,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct FileListQuery {
    pub folder_id: Option<Uuid>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

pub struct FileService;

impl FileService {
    /// Upload a file: write to storage, then insert DB record.
    pub async fn upload(
        db: &DatabaseConnection,
        storage: &Operator,
        user_id: Uuid,
        folder_id: Option<Uuid>,
        file_name: &str,
        content_type: &str,
        data: Vec<u8>,
        backend: &str,
    ) -> Result<FileInfo, AppError> {
        let file_id = Uuid::new_v4();
        let size = data.len() as i64;

        // Compute hash
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hex::encode(hasher.finalize());

        let storage_key = mdp_storage::storage_key::file(user_id, file_id);

        // Write to object storage
        storage
            .write(&storage_key, data)
            .await
            .map_err(AppError::Storage)?;

        let now = Utc::now();
        let file = files::ActiveModel {
            id: Set(file_id),
            user_id: Set(user_id),
            folder_id: Set(folder_id),
            name: Set(file_name.to_string()),
            storage_key: Set(storage_key),
            size: Set(size),
            content_type: Set(content_type.to_string()),
            hash_sha256: Set(hash),
            storage_backend: Set(backend.to_string()),
            status: Set("active".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let file = file.insert(db).await?;
        Ok(file.into())
    }

    /// Get file metadata by ID, scoped to the user.
    pub async fn get_by_id(
        db: &DatabaseConnection,
        user_id: Uuid,
        file_id: Uuid,
    ) -> Result<FileInfo, AppError> {
        let file = files::Entity::find_by_id(file_id)
            .filter(files::Column::UserId.eq(user_id))
            .filter(files::Column::Status.ne("deleted"))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("文件不存在".to_string()))?;

        Ok(file.into())
    }

    /// Read file bytes from storage for download.
    pub async fn download(
        db: &DatabaseConnection,
        storage: &Operator,
        user_id: Uuid,
        file_id: Uuid,
    ) -> Result<(files::Model, Vec<u8>), AppError> {
        let file = files::Entity::find_by_id(file_id)
            .filter(files::Column::UserId.eq(user_id))
            .filter(files::Column::Status.ne("deleted"))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("文件不存在".to_string()))?;

        let data = storage
            .read(&file.storage_key)
            .await
            .map_err(AppError::Storage)?
            .to_vec();

        Ok((file, data))
    }

    /// List files for a user, optionally filtered by folder, with pagination.
    pub async fn list(
        db: &DatabaseConnection,
        user_id: Uuid,
        query: &FileListQuery,
    ) -> Result<(Vec<FileInfo>, u64), AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100);

        let mut condition = Condition::all()
            .add(files::Column::UserId.eq(user_id))
            .add(files::Column::Status.ne("deleted"));

        if let Some(folder_id) = query.folder_id {
            condition = condition.add(files::Column::FolderId.eq(folder_id));
        } else {
            condition = condition.add(files::Column::FolderId.is_null());
        }

        let paginator = files::Entity::find()
            .filter(condition)
            .order_by_desc(files::Column::CreatedAt)
            .paginate(db, per_page);

        let total = paginator.num_items().await?;
        let items = paginator
            .fetch_page(page - 1)
            .await?
            .into_iter()
            .map(FileInfo::from)
            .collect();

        Ok((items, total))
    }

    /// Soft-delete a file.
    pub async fn delete(
        db: &DatabaseConnection,
        storage: &Operator,
        user_id: Uuid,
        file_id: Uuid,
    ) -> Result<(), AppError> {
        let file = files::Entity::find_by_id(file_id)
            .filter(files::Column::UserId.eq(user_id))
            .filter(files::Column::Status.ne("deleted"))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("文件不存在".to_string()))?;

        // Delete from storage
        storage
            .delete(&file.storage_key)
            .await
            .map_err(AppError::Storage)?;

        // Mark as deleted in DB
        let mut active: files::ActiveModel = file.into();
        active.status = Set("deleted".to_string());
        active.updated_at = Set(Utc::now());
        active.update(db).await?;

        Ok(())
    }
}
