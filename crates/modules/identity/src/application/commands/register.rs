use crate::application::{dto::AuthTokens, errors::IdentityApplicationError};

#[derive(Clone, Debug)]
pub struct RegisterCommand {
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Default)]
pub struct RegisterHandler;

impl RegisterHandler {
    pub fn for_tests() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        command: RegisterCommand,
    ) -> Result<AuthTokens, IdentityApplicationError> {
        if command.email.is_empty() || command.password.is_empty() {
            return Err(IdentityApplicationError::Validation);
        }

        Ok(AuthTokens {
            access_token: format!("access-{}", command.role),
            refresh_token: "refresh-token".into(),
        })
    }
}
