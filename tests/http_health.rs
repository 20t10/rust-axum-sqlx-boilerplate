#[tokio::test]
async fn health_route_returns_ok() {
    let response = reqwest::get("http://127.0.0.1:3000/health").await;
    assert!(response.is_ok());
}
