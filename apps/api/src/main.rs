mod bootstrap;
mod router;
mod state;

#[tokio::main]
async fn main() {
    bootstrap::run().await.expect("api should start");
}
