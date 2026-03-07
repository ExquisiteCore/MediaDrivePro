use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub auth: AuthConfig,
    #[serde(default)]
    pub webdav: WebDavConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    #[serde(default = "default_true")]
    pub auto_migrate: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub backend: String,
    #[serde(default)]
    pub fs: Option<FsStorageConfig>,
    #[serde(default)]
    pub s3: Option<S3StorageConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FsStorageConfig {
    pub root: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3StorageConfig {
    pub bucket: String,
    pub region: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub access_key_id: String,
    #[serde(default)]
    pub secret_access_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_ttl_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebDavConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_webdav_prefix")]
    pub prefix: String,
}

impl Default for WebDavConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: "/dav".to_string(),
        }
    }
}

fn default_webdav_prefix() -> String {
    "/dav".to_string()
}

impl AppConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;

        // Environment variable overrides (MDP_ prefix)
        if let Ok(val) = std::env::var("MDP_SERVER__PORT") {
            if let Ok(port) = val.parse() {
                config.server.port = port;
            }
        }
        if let Ok(val) = std::env::var("MDP_DATABASE__URL") {
            config.database.url = val;
        }
        if let Ok(val) = std::env::var("MDP_AUTH__JWT_SECRET") {
            config.auth.jwt_secret = val;
        }
        if let Ok(val) = std::env::var("MDP_STORAGE__BACKEND") {
            config.storage.backend = val;
        }
        if let Ok(val) = std::env::var("MDP_STORAGE__FS__ROOT") {
            config
                .storage
                .fs
                .get_or_insert(FsStorageConfig {
                    root: String::new(),
                })
                .root = val;
        }
        if let Ok(val) = std::env::var("MDP_STORAGE__S3__BUCKET") {
            config
                .storage
                .s3
                .get_or_insert(S3StorageConfig {
                    bucket: String::new(),
                    region: String::new(),
                    endpoint: String::new(),
                    access_key_id: String::new(),
                    secret_access_key: String::new(),
                })
                .bucket = val;
        }
        if let Ok(val) = std::env::var("MDP_STORAGE__S3__REGION") {
            config
                .storage
                .s3
                .get_or_insert(S3StorageConfig {
                    bucket: String::new(),
                    region: String::new(),
                    endpoint: String::new(),
                    access_key_id: String::new(),
                    secret_access_key: String::new(),
                })
                .region = val;
        }
        if let Ok(val) = std::env::var("MDP_STORAGE__S3__ENDPOINT") {
            config
                .storage
                .s3
                .get_or_insert(S3StorageConfig {
                    bucket: String::new(),
                    region: String::new(),
                    endpoint: String::new(),
                    access_key_id: String::new(),
                    secret_access_key: String::new(),
                })
                .endpoint = val;
        }
        if let Ok(val) = std::env::var("MDP_STORAGE__S3__ACCESS_KEY_ID") {
            config
                .storage
                .s3
                .get_or_insert(S3StorageConfig {
                    bucket: String::new(),
                    region: String::new(),
                    endpoint: String::new(),
                    access_key_id: String::new(),
                    secret_access_key: String::new(),
                })
                .access_key_id = val;
        }
        if let Ok(val) = std::env::var("MDP_STORAGE__S3__SECRET_ACCESS_KEY") {
            config
                .storage
                .s3
                .get_or_insert(S3StorageConfig {
                    bucket: String::new(),
                    region: String::new(),
                    endpoint: String::new(),
                    access_key_id: String::new(),
                    secret_access_key: String::new(),
                })
                .secret_access_key = val;
        }
        if let Ok(val) = std::env::var("MDP_WEBDAV__ENABLED") {
            config.webdav.enabled = val == "true" || val == "1";
        }

        Ok(config)
    }
}
