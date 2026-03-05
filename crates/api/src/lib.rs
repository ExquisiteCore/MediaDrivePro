use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod auth;
mod files;
mod folders;
pub mod state;

pub fn build_router(state: state::AppState) -> Router {
    let api_v1 = Router::new()
        .merge(auth::routes())
        .merge(files::routes())
        .merge(folders::routes());

    Router::new()
        .nest("/api/v1", api_v1)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
