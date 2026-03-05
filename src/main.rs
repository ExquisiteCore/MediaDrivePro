use mdp_common::config::AppConfig;
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::MigratorTrait;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::load("config.toml")?;
    tracing::info!(
        "Starting MediaDrivePro on {}:{}",
        config.server.host,
        config.server.port
    );

    // Connect to database
    let mut db_opts = ConnectOptions::new(&config.database.url);
    db_opts.max_connections(config.database.max_connections);
    let db = Database::connect(db_opts).await?;
    tracing::info!("Database connected");

    // Run migrations
    migration::Migrator::up(&db, None).await?;
    tracing::info!("Migrations applied");

    // Initialize storage
    let storage = mdp_storage::create_operator(&config.storage)
        .map_err(|e| format!("Storage init error: {e}"))?;
    tracing::info!("Storage backend '{}' initialized", config.storage.backend);

    // Build application
    let state = mdp_api::state::AppState {
        db,
        storage,
        config: config.clone(),
    };
    let app = mdp_api::build_router(state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
