use chrono::Utc;
use image::ImageReader;
use mdp_common::{config::ImageConfig, error::AppError};
use mdp_storage::operator::storage_key;
use opendal::Operator;
use sea_orm::*;
use sha2::{Digest, Sha256};
use std::io::Cursor;
use uuid::Uuid;

use crate::entity::images;

#[derive(Debug, serde::Serialize)]
pub struct ImageInfo {
    pub id: Uuid,
    pub hash: String,
    pub original_name: String,
    pub url: String,
    pub thumb_url: String,
    pub markdown: String,
    pub size: i64,
    pub original_size: i64,
    pub width: i32,
    pub height: i32,
    pub created_at: chrono::DateTime<Utc>,
}

impl ImageInfo {
    fn from_model(m: images::Model, cdn_base_url: &str) -> Self {
        let (url, thumb_url) = build_urls(&m.hash_sha256, cdn_base_url);
        let markdown = format!("![{}]({})", m.original_name, url);
        Self {
            id: m.id,
            hash: m.hash_sha256,
            original_name: m.original_name,
            url,
            thumb_url,
            markdown,
            size: m.size,
            original_size: m.original_size,
            width: m.width,
            height: m.height,
            created_at: m.created_at,
        }
    }
}

fn build_urls(hash: &str, cdn_base_url: &str) -> (String, String) {
    if cdn_base_url.is_empty() {
        (
            format!("/img/{hash}.webp"),
            format!("/img/thumb/{hash}.webp"),
        )
    } else {
        let base = cdn_base_url.trim_end_matches('/');
        (
            format!("{base}/{hash}.webp"),
            format!("{base}/thumb/{hash}.webp"),
        )
    }
}

const ALLOWED_TYPES: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp"];

pub struct ImageService;

impl ImageService {
    pub async fn upload(
        db: &DatabaseConnection,
        storage: &Operator,
        user_id: Uuid,
        filename: &str,
        content_type: &str,
        data: Vec<u8>,
        config: &ImageConfig,
        base_url: &str,
    ) -> Result<ImageInfo, AppError> {
        // 1. Validate format
        if !ALLOWED_TYPES.contains(&content_type) {
            return Err(AppError::Validation(
                "不支持的图片格式，仅支持 jpg/png/gif/webp".to_string(),
            ));
        }

        // 2. Validate size
        if data.len() > config.max_upload_size {
            return Err(AppError::PayloadTooLarge);
        }

        let original_size = data.len() as i64;

        // 3. SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hex::encode(hasher.finalize());

        // 4. Dedup check
        let existing = images::Entity::find()
            .filter(images::Column::HashSha256.eq(&hash))
            .one(db)
            .await?;

        if let Some(existing) = existing {
            return Ok(ImageInfo::from_model(existing, base_url));
        }

        // 5. Decode image
        let reader = ImageReader::new(Cursor::new(&data))
            .with_guessed_format()
            .map_err(|e| AppError::Validation(format!("无法识别图片格式: {e}")))?;

        let img = reader
            .decode()
            .map_err(|e| AppError::Validation(format!("图片解码失败: {e}")))?;

        let width = img.width() as i32;
        let height = img.height() as i32;

        // 6. Compress to WebP
        let webp_data = {
            let mut buf = Cursor::new(Vec::new());
            img.write_to(&mut buf, image::ImageFormat::WebP)
                .map_err(|e| AppError::Internal(format!("WebP 编码失败: {e}")))?;
            buf.into_inner()
        };

        // 7. Generate 300x300 thumbnail
        let thumb_data = {
            let thumb = img.thumbnail(300, 300);
            let mut buf = Cursor::new(Vec::new());
            thumb
                .write_to(&mut buf, image::ImageFormat::WebP)
                .map_err(|e| AppError::Internal(format!("缩略图生成失败: {e}")))?;
            buf.into_inner()
        };

        let size = webp_data.len() as i64;

        // 8. Store to object storage
        let storage_key = storage_key::image(&hash);
        let thumb_key = storage_key::image_thumb(&hash);

        storage
            .write(&storage_key, webp_data)
            .await
            .map_err(|e| AppError::Internal(format!("存储图片失败: {e}")))?;

        storage
            .write(&thumb_key, thumb_data)
            .await
            .map_err(|e| AppError::Internal(format!("存储缩略图失败: {e}")))?;

        // 9. Insert DB record
        let image_id = Uuid::new_v4();
        let now = Utc::now();

        let record = images::ActiveModel {
            id: Set(image_id),
            user_id: Set(user_id),
            hash_sha256: Set(hash),
            original_name: Set(filename.to_string()),
            storage_key: Set(storage_key),
            thumb_key: Set(thumb_key),
            size: Set(size),
            original_size: Set(original_size),
            width: Set(width),
            height: Set(height),
            content_type: Set("image/webp".to_string()),
            created_at: Set(now),
        };

        let model = record.insert(db).await?;
        Ok(ImageInfo::from_model(model, base_url))
    }

    pub async fn list(
        db: &DatabaseConnection,
        user_id: Uuid,
        cdn_base_url: &str,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<ImageInfo>, u64), AppError> {
        let paginator = images::Entity::find()
            .filter(images::Column::UserId.eq(user_id))
            .order_by_desc(images::Column::CreatedAt)
            .paginate(db, per_page);

        let total = paginator.num_items().await?;
        let items = paginator
            .fetch_page(page.saturating_sub(1))
            .await?
            .into_iter()
            .map(|m| ImageInfo::from_model(m, cdn_base_url))
            .collect();

        Ok((items, total))
    }

    pub async fn delete(
        db: &DatabaseConnection,
        storage: &Operator,
        user_id: Uuid,
        image_id: Uuid,
    ) -> Result<(), AppError> {
        let img = images::Entity::find_by_id(image_id)
            .filter(images::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("图片不存在".to_string()))?;

        // Delete from storage
        let _ = storage.delete(&img.storage_key).await;
        let _ = storage.delete(&img.thumb_key).await;

        // Delete from DB
        img.delete(db).await?;

        Ok(())
    }

    pub async fn get_by_hash(
        db: &DatabaseConnection,
        hash: &str,
    ) -> Result<images::Model, AppError> {
        images::Entity::find()
            .filter(images::Column::HashSha256.eq(hash))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("图片不存在".to_string()))
    }
}
