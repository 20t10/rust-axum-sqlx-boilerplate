use identity::domain::{
    entities::user::User,
    value_objects::{email::Email, password_hash::PasswordHash},
};

#[test]
fn user_registration_creates_active_user() {
    let user = User::register(
        Email::parse("dev@example.com").unwrap(),
        PasswordHash("hashed-password".into()),
        "admin".into(),
    );

    assert_eq!(user.email().as_str(), "dev@example.com");
    assert_eq!(user.role(), "admin");
}

#[test]
fn invalid_email_is_rejected() {
    assert!(Email::parse("not-an-email").is_err());
}
