#[derive(thiserror::Error, Debug)]
pub enum IdentityApplicationError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("validation failure")]
    Validation,
}
