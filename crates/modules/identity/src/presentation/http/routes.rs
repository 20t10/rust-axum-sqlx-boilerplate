use axum::{routing::post, Router};

use crate::presentation::http::handlers;

pub fn router() -> Router {
    Router::new().route("/api/v1/auth/register", post(handlers::register))
}
