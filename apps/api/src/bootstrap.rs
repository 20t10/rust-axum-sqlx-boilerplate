use building_blocks::config::{load_required_dotenv, AppConfig, ConfigError};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(thiserror::Error)]
pub enum StartupError {
    #[error("startup configuration failed")]
    Config(#[from] ConfigError),
    #[error("failed to bind API listener")]
    Bind(#[source] std::io::Error),
    #[error("API server failed")]
    Serve(#[source] std::io::Error),
}

impl std::fmt::Debug for StartupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::error::Error;

        write!(formatter, "{self}")?;
        if let Some(source) = self.source() {
            write!(formatter, ": {source}")?;
        }
        Ok(())
    }
}

pub fn listener_address(config: &AppConfig) -> Result<SocketAddr, ConfigError> {
    config.socket_addr()
}

pub async fn run() -> Result<(), StartupError> {
    let config = load_required_dotenv()?;
    building_blocks::logging::init_logging();
    let address = listener_address(&config)?;
    let app = crate::router::build_router();
    let listener = TcpListener::bind(address)
        .await
        .map_err(StartupError::Bind)?;
    axum::serve(listener, app)
        .await
        .map_err(StartupError::Serve)
}
