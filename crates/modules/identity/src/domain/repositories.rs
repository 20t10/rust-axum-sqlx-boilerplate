use async_trait::async_trait;

use crate::domain::{entities::user::User, value_objects::email::Email};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert(&self, user: &User) -> Result<(), crate::domain::errors::IdentityDomainError>;
    async fn find_by_email(
        &self,
        email: &Email,
    ) -> Result<Option<User>, crate::domain::errors::IdentityDomainError>;
}
