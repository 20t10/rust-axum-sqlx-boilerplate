use crate::application::dto::AuthTokens;

pub fn issue_tokens(_role: &str) -> AuthTokens {
    AuthTokens {
        access_token: "placeholder-access-token".into(),
        refresh_token: "placeholder-refresh-token".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::issue_tokens;

    #[test]
    fn issue_tokens_returns_neutral_placeholder_tokens() {
        let tokens = issue_tokens("admin");

        assert_eq!(tokens.access_token, "placeholder-access-token");
        assert_eq!(tokens.refresh_token, "placeholder-refresh-token");
    }
}
