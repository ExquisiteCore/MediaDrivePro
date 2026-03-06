mod auth;
mod filesystem;

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
};
use dav_server::DavHandler;
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
pub async fn webdav_handler(
    State(state): State<WebDavState>,
    req: Request<Body>,
) -> impl IntoResponse {
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

    // Create a per-user DavFileSystem
    let fs = MdpDavFs::new(
        state.db.clone(),
        state.storage.clone(),
        user_id,
        state.storage_backend.clone(),
    );

    let dav_handler = DavHandler::builder()
        .filesystem(Box::new(fs))
        .strip_prefix("/dav")
        .build_handler();

    // Convert axum Request to dav-server compatible request
    match dav_handler.handle(req).await {
        Ok(resp) => resp,
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("WebDAV error"))
            .unwrap(),
    }
}
