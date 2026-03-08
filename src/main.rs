use mdp_common::config::AppConfig;
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::MigratorTrait;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration first (needed for log_level)
    let config = AppConfig::load("config.toml")?;

    // Initialize tracing with configured log level
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.server.log_level)))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(
        "Starting MediaDrivePro on {}:{}",
        config.server.host,
        config.server.port
    );

    // Connect to database
    // For SQLite, ensure the parent directory exists
    if config.database.url.starts_with("sqlite:") {
        if let Some(path) = config.database.url.strip_prefix("sqlite:") {
            let path = path.split('?').next().unwrap_or(path);
            if let Some(parent) = std::path::Path::new(path).parent() {
                std::fs::create_dir_all(parent).ok();
            }
        }
    }
    let mut db_opts = ConnectOptions::new(&config.database.url);
    db_opts.max_connections(config.database.max_connections);
    let db = Database::connect(db_opts).await?;
    tracing::info!("Database connected");

    // Run migrations
    if config.database.auto_migrate {
        migration::Migrator::up(&db, None).await?;
        tracing::info!("Migrations applied");
    }

    // Initialize storage
    let storage = mdp_storage::create_operator(&config.storage)
        .map_err(|e| format!("Storage init error: {e}"))?;
    tracing::info!("Storage backend '{}' initialized", config.storage.backend);

    // Initialize upload sessions
    let upload_sessions = mdp_core::multipart_upload::new_sessions();

    // Start background cleanup task for expired upload sessions
    {
        let sessions = upload_sessions.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                mdp_core::multipart_upload::MultipartUploadService::cleanup_expired(&sessions)
                    .await;
            }
        });
    }

    // Start background transcode worker
    {
        let db2 = db.clone();
        let storage2 = storage.clone();
        let video_config = config.video.clone();
        let max_concurrent = video_config.max_concurrent.max(1);
        let poll_interval = video_config.poll_interval_secs;

        tokio::spawn(async move {
            let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));
            tracing::info!(
                "Transcode worker started (max_concurrent={max_concurrent}, poll_interval={poll_interval}s)"
            );

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(poll_interval)).await;

                let permit = match semaphore.clone().try_acquire_owned() {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                match mdp_core::transcode::TranscodeService::poll_pending(&db2).await {
                    Ok(Some(task)) => {
                        let db3 = db2.clone();
                        let storage3 = storage2.clone();
                        let cfg = video_config.clone();
                        tokio::spawn(async move {
                            mdp_core::transcode::run_transcode(&db3, &storage3, &cfg, task)
                                .await;
                            drop(permit);
                        });
                    }
                    Ok(None) => {
                        drop(permit);
                    }
                    Err(e) => {
                        tracing::error!("Transcode poll error: {e}");
                        drop(permit);
                    }
                }
            }
        });
    }

    // Build application
    let state = mdp_api::state::AppState {
        db: db.clone(),
        storage: storage.clone(),
        config: config.clone(),
        upload_sessions,
        room_channels: std::sync::Arc::new(dashmap::DashMap::new()),
    };
    let mut app = mdp_api::build_router(state);

    // Mount WebDAV if enabled
    if config.webdav.enabled {
        let webdav_state = mdp_webdav::WebDavState {
            db,
            storage,
            storage_backend: config.storage.backend.clone(),
            prefix: config.webdav.prefix.clone(),
        };
        // Use wildcard routes so the full request URI (including /dav prefix) is
        // preserved. dav-server's strip_prefix then handles both request path and
        // Destination header consistently.
        let prefix = &config.webdav.prefix;
        app = app
            .route(
                &format!("{prefix}/{{*rest}}"),
                axum::routing::any(mdp_webdav::webdav_handler).with_state(webdav_state.clone()),
            )
            .route(
                prefix,
                axum::routing::any(mdp_webdav::webdav_handler).with_state(webdav_state.clone()),
            )
            .route(
                &format!("{prefix}/"),
                axum::routing::any(mdp_webdav::webdav_handler).with_state(webdav_state),
            );
        tracing::info!("WebDAV enabled at {}", config.webdav.prefix);
    }

    // Serve SPA static files from web/dist/ with fallback to index.html
    let spa = ServeDir::new("web/dist")
        .fallback(ServeFile::new("web/dist/index.html"));
    app = app.fallback_service(spa);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
