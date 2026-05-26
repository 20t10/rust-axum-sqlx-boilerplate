use axum::{routing::get, Router};

pub fn build_router() -> Router {
    Router::new().route("/health", get(|| async { "ok" }))
}
