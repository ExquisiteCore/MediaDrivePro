use axum::extract::FromRef;
use mdp_common::config::AppConfig;
use mdp_core::multipart_upload::UploadSessions;
use opendal::Operator;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub storage: Operator,
    pub config: AppConfig,
    pub upload_sessions: UploadSessions,
}

// Allow extracting AppConfig directly from AppState in Axum extractors.
impl FromRef<AppState> for AppConfig {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}
