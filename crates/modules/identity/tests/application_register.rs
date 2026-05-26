use identity::application::{
    commands::register::{RegisterCommand, RegisterHandler},
    errors::IdentityApplicationError,
};

#[tokio::test]
async fn register_handler_returns_tokens() {
    let handler = RegisterHandler::default();

    let result = handler
        .handle(RegisterCommand {
            email: "dev@example.com".into(),
            password: "secret123".into(),
            role: "admin".into(),
        })
        .await;

    let auth = result.expect("register should succeed");
    assert_eq!(auth.access_token, "access-admin");
    assert_eq!(auth.refresh_token, "refresh-token");
}

#[tokio::test]
async fn register_handler_returns_validation_error_for_missing_email() {
    let handler = RegisterHandler::default();

    let result = handler
        .handle(RegisterCommand {
            email: String::new(),
            password: "secret123".into(),
            role: "admin".into(),
        })
        .await;

    let err = result.expect_err("empty email should fail validation");
    assert!(matches!(err, IdentityApplicationError::Validation));
}
