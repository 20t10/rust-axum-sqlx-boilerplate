use shared_kernel::ids::UserId;

use crate::domain::value_objects::{email::Email, password_hash::PasswordHash};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct User {
    id: UserId,
    email: Email,
    password_hash: PasswordHash,
    role: String,
}

impl User {
    pub fn register(email: Email, password_hash: PasswordHash, role: String) -> Self {
        Self {
            id: UserId::new(),
            email,
            password_hash,
            role,
        }
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn role(&self) -> &str {
        &self.role
    }
}
