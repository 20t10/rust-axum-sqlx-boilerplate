use axum::{body::to_bytes, body::Body, http::Request};
use tower::ServiceExt;

#[tokio::test]
async fn register_endpoint_returns_tokens() {
    let app = api::router::build_router();

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"dev@example.com","password":"secret123","role":"admin"}"#,
        ))
        .expect("request should build");

    let response = app
        .oneshot(request)
        .await
        .expect("response should complete");

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should collect");
    let body = String::from_utf8(body.to_vec()).expect("body should be utf8");

    assert!(body.contains("access-admin"));
    assert!(body.contains("refresh-token"));
}
