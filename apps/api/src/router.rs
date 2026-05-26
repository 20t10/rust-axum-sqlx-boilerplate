use axum::{response::Html, routing::get, Router};
use identity::presentation::http::routes as identity_routes;

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/docs", get(|| async { Html("<h1>API Docs</h1>") }))
        .merge(identity_routes::router())
}
