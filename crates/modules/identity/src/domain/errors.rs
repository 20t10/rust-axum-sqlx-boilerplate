#[derive(thiserror::Error, Debug)]
pub enum IdentityDomainError {
    #[error("email already exists")]
    EmailAlreadyExists,
}
