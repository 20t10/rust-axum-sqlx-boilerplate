use axum::{routing::get, Router};

use crate::presentation::http::handlers;

pub fn router() -> Router {
    Router::new().route("/api/v1/billing", get(handlers::index))
}
