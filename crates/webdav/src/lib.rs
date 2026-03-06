mod auth;
mod filesystem;

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
};
use dav_server::{DavConfig, DavHandler};
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
        .build_handler();

    let config = DavConfig::new().strip_prefix("/dav");

    let dav_resp = dav_handler.handle_with(config, req).await;

    // Convert dav_server::body::Body → axum::body::Body
    // dav body implements HttpBody<Data=Bytes, Error=io::Error>
    // We need to collect it and convert
    let (parts, dav_body) = dav_resp.into_parts();
    let mapped = dav_body.map_err(|e| {
        axum::Error::new(e)
    });
    let axum_body = Body::new(mapped);
    Response::from_parts(parts, axum_body)
}
