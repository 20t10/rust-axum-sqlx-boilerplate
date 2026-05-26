use identity::application::commands::register::{RegisterCommand, RegisterHandler};

#[tokio::test]
async fn register_handler_returns_tokens() {
    let handler = RegisterHandler::for_tests();

    let result = handler
        .handle(RegisterCommand {
            email: "dev@example.com".into(),
            password: "secret123".into(),
            role: "admin".into(),
        })
        .await;

    assert!(result.is_ok());
    let auth = result.unwrap();
    assert!(!auth.access_token.is_empty());
    assert!(!auth.refresh_token.is_empty());
}
