#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("configuration error")]
    Config,
    #[error("database error")]
    Database,
}
