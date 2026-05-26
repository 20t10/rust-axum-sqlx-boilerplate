use axum::serve;
use tokio::net::TcpListener;

pub async fn run() -> Result<(), std::io::Error> {
    let app = crate::router::build_router();
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    serve(listener, app).await
}
