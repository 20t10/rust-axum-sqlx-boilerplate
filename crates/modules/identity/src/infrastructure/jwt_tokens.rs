use crate::application::dto::AuthTokens;

pub fn issue_tokens(role: &str) -> AuthTokens {
    AuthTokens {
        access_token: format!("access::{role}"),
        refresh_token: format!("refresh::{role}"),
    }
}
