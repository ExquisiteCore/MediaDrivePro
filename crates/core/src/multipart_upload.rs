use chrono::{DateTime, Utc};
use dashmap::DashMap;
use mdp_common::error::AppError;
use opendal::Operator;
use sea_orm::DatabaseConnection;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

use crate::file::FileService;

/// In-memory upload session state.
pub struct UploadSession {
    pub user_id: Uuid,
    pub file_name: String,
    pub folder_id: Option<Uuid>,
    pub content_type: String,
    pub parts: BTreeMap<u32, PathBuf>,
    pub created_at: DateTime<Utc>,
}

/// Shared upload session manager.
pub type UploadSessions = Arc<DashMap<String, UploadSession>>;

/// Create a new shared session manager.
pub fn new_sessions() -> UploadSessions {
    Arc::new(DashMap::new())
}

#[derive(Debug, serde::Serialize)]
pub struct InitResponse {
    pub upload_id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct PartResponse {
    pub part_number: u32,
    pub size: u64,
}

pub struct MultipartUploadService;

impl MultipartUploadService {
    /// Initialize a new multipart upload.
    pub fn init(
        sessions: &UploadSessions,
        user_id: Uuid,
        file_name: &str,
        folder_id: Option<Uuid>,
        content_type: &str,
    ) -> Result<InitResponse, AppError> {
        let upload_id = Uuid::new_v4().to_string();

        sessions.insert(
            upload_id.clone(),
            UploadSession {
                user_id,
                file_name: file_name.to_string(),
                folder_id,
                content_type: content_type.to_string(),
                parts: BTreeMap::new(),
                created_at: Utc::now(),
            },
        );

        Ok(InitResponse { upload_id })
    }

    /// Upload a single part. Writes to temp file.
    pub async fn upload_part(
        sessions: &UploadSessions,
        upload_id: &str,
        part_number: u32,
        user_id: Uuid,
        data: Vec<u8>,
    ) -> Result<PartResponse, AppError> {
        // Verify session exists and belongs to user
        {
            let session = sessions
                .get(upload_id)
                .ok_or(AppError::NotFound("上传会话不存在".to_string()))?;
            if session.user_id != user_id {
                return Err(AppError::Forbidden);
            }
        }

        // Write part to temp file
        let tmp_dir = Path::new("./data/tmp");
        tokio::fs::create_dir_all(tmp_dir)
            .await
            .map_err(|e| AppError::Internal(format!("创建临时目录失败: {e}")))?;

        let part_path = tmp_dir.join(format!("{upload_id}_{part_number:06}"));
        let size = data.len() as u64;

        tokio::fs::write(&part_path, &data)
            .await
            .map_err(|e| AppError::Internal(format!("写入分片失败: {e}")))?;

        // Record part path in session
        sessions
            .get_mut(upload_id)
            .ok_or(AppError::NotFound("上传会话不存在".to_string()))?
            .parts
            .insert(part_number, part_path);

        Ok(PartResponse { part_number, size })
    }

    /// Complete the multipart upload: concatenate parts → write to storage → insert DB record.
    pub async fn complete(
        sessions: &UploadSessions,
        db: &DatabaseConnection,
        storage: &Operator,
        upload_id: &str,
        user_id: Uuid,
        storage_backend: &str,
    ) -> Result<crate::file::FileInfo, AppError> {
        // Remove session (take ownership)
        let (_, session) = sessions
            .remove(upload_id)
            .ok_or(AppError::NotFound("上传会话不存在".to_string()))?;

        if session.user_id != user_id {
            return Err(AppError::Forbidden);
        }

        if session.parts.is_empty() {
            return Err(AppError::Validation("没有上传任何分片".to_string()));
        }

        // Concatenate all parts in order
        let mut combined = Vec::new();
        for (_part_num, part_path) in &session.parts {
            let data = tokio::fs::read(part_path)
                .await
                .map_err(|e| AppError::Internal(format!("读取分片失败: {e}")))?;
            combined.extend_from_slice(&data);
        }

        // Clean up temp files
        for (_, part_path) in &session.parts {
            tokio::fs::remove_file(part_path).await.ok();
        }

        // Use FileService.upload to handle quota, hash, storage, DB
        let info = FileService::upload(
            db,
            storage,
            user_id,
            session.folder_id,
            &session.file_name,
            &session.content_type,
            combined,
            storage_backend,
        )
        .await?;

        Ok(info)
    }

    /// Cancel (abort) a multipart upload. Cleans up temp files.
    pub async fn cancel(
        sessions: &UploadSessions,
        upload_id: &str,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let (_, session) = sessions
            .remove(upload_id)
            .ok_or(AppError::NotFound("上传会话不存在".to_string()))?;

        if session.user_id != user_id {
            return Err(AppError::Forbidden);
        }

        // Clean up temp files
        for (_, part_path) in &session.parts {
            tokio::fs::remove_file(part_path).await.ok();
        }

        Ok(())
    }

    /// Clean up expired sessions (older than 1 hour). Run periodically.
    pub async fn cleanup_expired(sessions: &UploadSessions) {
        let cutoff = Utc::now() - chrono::Duration::hours(1);
        let expired_ids: Vec<String> = sessions
            .iter()
            .filter(|entry| entry.value().created_at < cutoff)
            .map(|entry| entry.key().clone())
            .collect();

        for id in expired_ids {
            if let Some((_, session)) = sessions.remove(&id) {
                for (_, part_path) in &session.parts {
                    tokio::fs::remove_file(part_path).await.ok();
                }
                tracing::info!("Cleaned up expired upload session: {id}");
            }
        }
    }
}
