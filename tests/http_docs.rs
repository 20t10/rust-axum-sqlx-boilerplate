use api::router;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn docs_route_returns_html() {
    let response = router::build_router()
        .oneshot(
            Request::builder()
                .uri("/docs")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should collect");
    let body = String::from_utf8(body.to_vec()).expect("body should be utf8");

    assert!(body.contains("API Docs"));
}
