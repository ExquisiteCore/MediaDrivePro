use chrono::Utc;
use mdp_common::error::AppError;
use opendal::Operator;
use sea_orm::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::entity::files;
use crate::entity::users;

#[derive(Debug, serde::Serialize)]
pub struct FileInfo {
    pub id: Uuid,
    pub name: String,
    pub size: i64,
    pub content_type: String,
    pub folder_id: Option<Uuid>,
    pub status: String,
    pub transcode_status: Option<String>,
    pub has_media_info: bool,
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
            transcode_status: None,
            has_media_info: false,
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
    pub search: Option<String>,
    /// Sort field: name | size | created_at (default: created_at)
    pub sort: Option<String>,
    /// Sort order: asc | desc (default: desc)
    pub order: Option<String>,
}

pub struct FileService;

impl FileService {
    /// Upload a file: check quota, write to storage, then insert DB record.
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

        // Check storage quota
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or(AppError::NotFound("用户不存在".to_string()))?;

        if user.storage_used + size > user.storage_quota {
            return Err(AppError::QuotaExceeded);
        }

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

        // Update storage_used
        let mut user_active: users::ActiveModel = user.into();
        user_active.storage_used = Set(user_active.storage_used.unwrap() + size);
        user_active.update(db).await?;

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

    /// List files for a user, optionally filtered by folder, with pagination, search and sort.
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
        } else if query.search.is_none() {
            // Only filter to root when not searching (search should be global)
            condition = condition.add(files::Column::FolderId.is_null());
        }

        if let Some(ref search) = query.search {
            if !search.is_empty() {
                condition = condition.add(files::Column::Name.contains(search));
            }
        }

        let order = match query.order.as_deref() {
            Some("asc") => Order::Asc,
            _ => Order::Desc,
        };

        let sort_col: files::Column = match query.sort.as_deref() {
            Some("name") => files::Column::Name,
            Some("size") => files::Column::Size,
            _ => files::Column::CreatedAt,
        };

        let paginator = files::Entity::find()
            .filter(condition)
            .order_by(sort_col, order)
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

        let file_size = file.size;

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

        // Decrease storage_used
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or(AppError::NotFound("用户不存在".to_string()))?;
        let mut user_active: users::ActiveModel = user.into();
        let current = user_active.storage_used.clone().unwrap();
        user_active.storage_used = Set((current - file_size).max(0));
        user_active.update(db).await?;

        Ok(())
    }

    /// Rename and/or move a file to another folder.
    pub async fn rename_move(
        db: &DatabaseConnection,
        user_id: Uuid,
        file_id: Uuid,
        new_name: Option<&str>,
        new_folder_id: Option<Option<Uuid>>,
    ) -> Result<FileInfo, AppError> {
        let file = files::Entity::find_by_id(file_id)
            .filter(files::Column::UserId.eq(user_id))
            .filter(files::Column::Status.ne("deleted"))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("文件不存在".to_string()))?;

        let mut active: files::ActiveModel = file.into();

        if let Some(name) = new_name {
            if name.is_empty() || name.len() > 255 {
                return Err(AppError::Validation(
                    "文件名长度需在 1-255 之间".to_string(),
                ));
            }
            active.name = Set(name.to_string());
        }

        if let Some(folder_id) = new_folder_id {
            // Verify target folder exists and belongs to user
            if let Some(fid) = folder_id {
                use crate::entity::folders;
                folders::Entity::find_by_id(fid)
                    .filter(folders::Column::UserId.eq(user_id))
                    .one(db)
                    .await?
                    .ok_or(AppError::NotFound("目标目录不存在".to_string()))?;
            }
            active.folder_id = Set(folder_id);
        }

        active.updated_at = Set(Utc::now());
        let updated = active.update(db).await?;
        Ok(updated.into())
    }
}
