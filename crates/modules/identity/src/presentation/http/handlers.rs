use axum::{http::StatusCode, Json};

use crate::{
    application::commands::register::{RegisterCommand, RegisterHandler},
    presentation::http::contracts::{AuthResponse, RegisterRequest},
};

pub async fn register(Json(request): Json<RegisterRequest>) -> (StatusCode, Json<AuthResponse>) {
    let handler = RegisterHandler;
    let tokens = handler
        .handle(RegisterCommand {
            email: request.email,
            password: request.password,
            role: request.role,
        })
        .await
        .expect("register should succeed");

    (
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        }),
    )
}
