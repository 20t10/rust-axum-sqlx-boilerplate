#[tokio::main]
async fn main() -> Result<(), api::bootstrap::StartupError> {
    api::bootstrap::run().await
}
