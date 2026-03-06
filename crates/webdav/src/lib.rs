mod auth;
mod filesystem;

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
};
use dav_server::DavHandler;
use http_body_util::BodyExt;
use opendal::Operator;
use sea_orm::DatabaseConnection;

use filesystem::MdpDavFs;

#[derive(Clone)]
pub struct WebDavState {
    pub db: DatabaseConnection,
    pub storage: Operator,
    pub storage_backend: String,
}

/// Axum handler that processes all WebDAV requests under /dav/.
/// Mounted via `nest("/dav", ...)` so the prefix is already stripped.
pub async fn webdav_handler(
    State(state): State<WebDavState>,
    req: Request<Body>,
) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().clone();
    tracing::debug!("WebDAV request: {method} {uri}");

    // Extract Basic Auth
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let (user_id, _permissions) = match auth::verify_basic_auth(&state.db, auth_header).await {
        Ok(result) => result,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("WWW-Authenticate", "Basic realm=\"MediaDrivePro WebDAV\"")
                .body(Body::from("Unauthorized"))
                .unwrap();
        }
    };

    tracing::debug!("WebDAV auth OK for user {user_id}");

    // Create a per-user DavFileSystem
    let fs = MdpDavFs::new(
        state.db.clone(),
        state.storage.clone(),
        user_id,
        state.storage_backend.clone(),
    );

    let dav_handler = DavHandler::builder()
        .filesystem(Box::new(fs))
        .build_handler();

    // No strip_prefix needed — nest() already stripped "/dav"
    let dav_resp = dav_handler.handle(req).await;

    tracing::debug!("WebDAV dav-server responded: {}", dav_resp.status());

    // Convert dav_server::body::Body → axum::body::Body
    // Collect the full body to avoid streaming conversion issues
    let (parts, dav_body) = dav_resp.into_parts();
    match BodyExt::collect(dav_body).await {
        Ok(collected) => {
            let body = Body::from(collected.to_bytes());
            Response::from_parts(parts, body)
        }
        Err(e) => {
            tracing::error!("WebDAV body collect error: {e}");
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal Server Error"))
                .unwrap()
        }
    }
}
