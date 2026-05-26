use shared_kernel::{auth::AuthClaims, ids::UserId, pagination::PageRequest};

#[test]
fn user_id_is_constructible() {
    let id = UserId::new();
    assert_ne!(id.to_string(), "");
}

#[test]
fn page_request_defaults_are_stable() {
    let page = PageRequest::default();
    assert_eq!(page.page, 1);
    assert_eq!(page.per_page, 20);
}

#[test]
fn auth_claims_hold_subject_and_role() {
    let claims = AuthClaims {
        sub: UserId::new(),
        role: "admin".into(),
        exp: 1,
    };

    assert_eq!(claims.role, "admin");
}
