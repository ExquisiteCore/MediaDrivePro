use mdp_common::config::StorageConfig;
use opendal::{services, Operator};
use uuid::Uuid;

/// Create an OpenDAL Operator based on the storage configuration.
pub fn create_operator(config: &StorageConfig) -> Result<Operator, opendal::Error> {
    match config.backend.as_str() {
        "fs" => {
            let fs_config = config
                .fs
                .as_ref()
                .expect("storage.fs config is required when backend = 'fs'");

            // Ensure the directory exists
            std::fs::create_dir_all(&fs_config.root).ok();

            let builder = services::Fs::default().root(&fs_config.root);
            Operator::new(builder)?.finish().into_ok()
        }
        "s3" | "minio" => {
            let s3_config = config
                .s3
                .as_ref()
                .expect("storage.s3 config is required when backend = 's3' or 'minio'");

            let mut builder = services::S3::default()
                .bucket(&s3_config.bucket)
                .region(&s3_config.region)
                .access_key_id(&s3_config.access_key_id)
                .secret_access_key(&s3_config.secret_access_key);

            if !s3_config.endpoint.is_empty() {
                builder = builder.endpoint(&s3_config.endpoint);
            }

            Operator::new(builder)?.finish().into_ok()
        }
        other => panic!("Unsupported storage backend: {other}"),
    }
}

/// Helper to generate storage keys for different resource types.
pub mod storage_key {
    use uuid::Uuid;

    /// Key for user-uploaded files: data/{user_id}/{file_id}
    pub fn file(user_id: Uuid, file_id: Uuid) -> String {
        format!("data/{user_id}/{file_id}")
    }

    /// Key for thumbnails: thumb/{file_id}.webp
    pub fn thumbnail(file_id: Uuid) -> String {
        format!("thumb/{file_id}.webp")
    }

    /// Key for image bed images: image/{hash}.webp
    pub fn image(hash: &str) -> String {
        format!("image/{hash}.webp")
    }
}

trait IntoOk {
    fn into_ok(self) -> Result<Operator, opendal::Error>;
}

impl IntoOk for Operator {
    fn into_ok(self) -> Result<Operator, opendal::Error> {
        Ok(self)
    }
}
