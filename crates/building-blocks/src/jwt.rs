#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JwtSettings {
    pub access_secret: String,
    pub refresh_secret: String,
    pub access_ttl_minutes: i64,
    pub refresh_ttl_days: i64,
}
