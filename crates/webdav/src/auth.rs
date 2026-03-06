use base64::Engine;
use mdp_core::api_token::ApiTokenService;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

/// Extract and verify Basic Auth credentials from an Authorization header value.
/// Returns (user_id, permissions) on success.
pub async fn verify_basic_auth(
    db: &DatabaseConnection,
    auth_header: &str,
) -> Result<(Uuid, String), ()> {
    let encoded = auth_header.strip_prefix("Basic ").ok_or(())?;

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| ())?;

    let credentials = String::from_utf8(decoded).map_err(|_| ())?;
    let (username, token) = credentials.split_once(':').ok_or(())?;

    ApiTokenService::verify(db, username, token)
        .await
        .map_err(|_| ())
}
