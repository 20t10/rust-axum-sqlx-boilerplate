#[derive(thiserror::Error, Debug)]
pub enum IdentityDomainError {
    #[error("email already exists")]
    EmailAlreadyExists,
    #[error("persistence failure")]
    Persistence,
}
