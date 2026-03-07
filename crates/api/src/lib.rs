use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod admin;
mod auth;
mod files;
mod folders;
mod shares;
pub mod state;
mod tokens;

pub fn build_router(state: state::AppState) -> Router {
    let api_v1 = Router::new()
        .merge(auth::routes())
        .merge(files::routes())
        .merge(folders::routes())
        .merge(shares::routes())
        .merge(tokens::routes())
        .merge(admin::routes());

    Router::new()
        .nest("/api/v1", api_v1)
        .merge(shares::public_routes())
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
