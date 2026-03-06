use chrono::Utc;
use mdp_common::error::AppError;
use sea_orm::*;
use uuid::Uuid;

use crate::entity::{files, folders};

#[derive(Debug, serde::Serialize)]
pub struct FolderInfo {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl From<folders::Model> for FolderInfo {
    fn from(f: folders::Model) -> Self {
        Self {
            id: f.id,
            name: f.name,
            parent_id: f.parent_id,
            created_at: f.created_at,
            updated_at: f.updated_at,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct FolderChildren {
    pub folders: Vec<FolderInfo>,
    pub files: Vec<crate::file::FileInfo>,
}

pub struct FolderService;

impl FolderService {
    pub async fn create(
        db: &DatabaseConnection,
        user_id: Uuid,
        parent_id: Option<Uuid>,
        name: &str,
    ) -> Result<FolderInfo, AppError> {
        if name.is_empty() || name.len() > 255 {
            return Err(AppError::Validation(
                "目录名长度需在 1-255 之间".to_string(),
            ));
        }

        // Verify parent exists and belongs to user
        if let Some(pid) = parent_id {
            folders::Entity::find_by_id(pid)
                .filter(folders::Column::UserId.eq(user_id))
                .one(db)
                .await?
                .ok_or(AppError::NotFound("父目录不存在".to_string()))?;
        }

        let now = Utc::now();
        let folder = folders::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            parent_id: Set(parent_id),
            name: Set(name.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let folder = folder.insert(db).await.map_err(|e| {
            if matches!(e, DbErr::RecordNotInserted) {
                AppError::Conflict("同名目录已存在".to_string())
            } else {
                AppError::Database(e)
            }
        })?;

        Ok(folder.into())
    }

    pub async fn get_by_id(
        db: &DatabaseConnection,
        user_id: Uuid,
        folder_id: Uuid,
    ) -> Result<FolderInfo, AppError> {
        let folder = folders::Entity::find_by_id(folder_id)
            .filter(folders::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("目录不存在".to_string()))?;

        Ok(folder.into())
    }

    /// List children (sub-folders and files) of a folder.
    pub async fn list_children(
        db: &DatabaseConnection,
        user_id: Uuid,
        folder_id: Option<Uuid>,
    ) -> Result<FolderChildren, AppError> {
        // If a specific folder, verify ownership
        if let Some(fid) = folder_id {
            folders::Entity::find_by_id(fid)
                .filter(folders::Column::UserId.eq(user_id))
                .one(db)
                .await?
                .ok_or(AppError::NotFound("目录不存在".to_string()))?;
        }

        // Sub-folders
        let mut folder_query = folders::Entity::find().filter(folders::Column::UserId.eq(user_id));

        if let Some(fid) = folder_id {
            folder_query = folder_query.filter(folders::Column::ParentId.eq(fid));
        } else {
            folder_query = folder_query.filter(folders::Column::ParentId.is_null());
        }

        let sub_folders: Vec<FolderInfo> = folder_query
            .order_by_asc(folders::Column::Name)
            .all(db)
            .await?
            .into_iter()
            .map(FolderInfo::from)
            .collect();

        // Files in this folder
        let mut file_query = files::Entity::find()
            .filter(files::Column::UserId.eq(user_id))
            .filter(files::Column::Status.ne("deleted"));

        if let Some(fid) = folder_id {
            file_query = file_query.filter(files::Column::FolderId.eq(fid));
        } else {
            file_query = file_query.filter(files::Column::FolderId.is_null());
        }

        let file_list: Vec<crate::file::FileInfo> = file_query
            .order_by_desc(files::Column::CreatedAt)
            .all(db)
            .await?
            .into_iter()
            .map(crate::file::FileInfo::from)
            .collect();

        Ok(FolderChildren {
            folders: sub_folders,
            files: file_list,
        })
    }

    /// Delete a folder and all its contents recursively.
    pub async fn delete(
        db: &DatabaseConnection,
        user_id: Uuid,
        folder_id: Uuid,
    ) -> Result<(), AppError> {
        let folder = folders::Entity::find_by_id(folder_id)
            .filter(folders::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("目录不存在".to_string()))?;

        // CASCADE delete handles sub-folders; files get folder_id set to NULL
        folder.delete(db).await?;

        Ok(())
    }

    /// Rename and/or move a folder to another parent.
    pub async fn rename_move(
        db: &DatabaseConnection,
        user_id: Uuid,
        folder_id: Uuid,
        new_name: Option<&str>,
        new_parent_id: Option<Option<Uuid>>,
    ) -> Result<FolderInfo, AppError> {
        let folder = folders::Entity::find_by_id(folder_id)
            .filter(folders::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("目录不存在".to_string()))?;

        let mut active: folders::ActiveModel = folder.into();

        if let Some(name) = new_name {
            if name.is_empty() || name.len() > 255 {
                return Err(AppError::Validation(
                    "目录名长度需在 1-255 之间".to_string(),
                ));
            }
            active.name = Set(name.to_string());
        }

        if let Some(parent_id) = new_parent_id {
            // Cannot move a folder into itself
            if parent_id == Some(folder_id) {
                return Err(AppError::Validation("不能将目录移动到自身".to_string()));
            }
            // Verify target parent exists and belongs to user
            if let Some(pid) = parent_id {
                folders::Entity::find_by_id(pid)
                    .filter(folders::Column::UserId.eq(user_id))
                    .one(db)
                    .await?
                    .ok_or(AppError::NotFound("目标父目录不存在".to_string()))?;
            }
            active.parent_id = Set(parent_id);
        }

        active.updated_at = Set(Utc::now());
        let updated = active.update(db).await?;
        Ok(updated.into())
    }
}
