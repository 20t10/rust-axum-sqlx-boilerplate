use chrono::{DateTime, Utc};
use shared_kernel::{auth::AuthClaims, ids::UserId, pagination::PageRequest, Clock};

struct FixedClock {
    now: DateTime<Utc>,
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.now
    }
}

#[test]
fn user_id_is_constructible() {
    let id = UserId::new();
    let value = id.to_string();

    assert_eq!(value.len(), 36);
    assert_eq!(value.matches('-').count(), 4);
    let parsed = uuid::Uuid::parse_str(&value).unwrap();
    assert_eq!(parsed.get_version_num(), 7);
}

#[test]
fn page_request_defaults_are_stable() {
    let page = PageRequest::default();
    assert_eq!(page.page, 1);
    assert_eq!(page.per_page, 20);
}

#[test]
fn auth_claims_hold_subject_and_role() {
    let sub = UserId::new();
    let claims = AuthClaims {
        sub,
        role: "admin".into(),
        exp: 123,
    };

    assert_eq!(claims.sub, sub);
    assert_eq!(claims.role, "admin");
    assert_eq!(claims.exp, 123);
}

#[test]
fn clock_trait_returns_now_from_test_double() {
    let now = DateTime::parse_from_rfc3339("2026-05-26T15:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let clock = FixedClock { now };

    assert_eq!(clock.now(), now);
}
