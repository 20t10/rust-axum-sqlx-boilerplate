use axum::{routing::get, Router};

use crate::presentation::http::handlers;

pub fn router() -> Router {
    Router::new().route("/api/v1/__MODULE__", get(handlers::index))
}
