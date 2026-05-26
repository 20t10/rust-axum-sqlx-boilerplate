use shared_kernel::ids::UserId;

use crate::domain::value_objects::email::Email;

#[derive(Clone, Debug)]
pub struct User {
    id: UserId,
    email: Email,
    password_hash: String,
    role: String,
}

impl User {
    pub fn register(email: Email, password_hash: String, role: String) -> Self {
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
