#[tokio::main]
async fn main() {
    api::bootstrap::run().await.expect("api should start");
}
