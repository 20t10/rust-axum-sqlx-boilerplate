#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppConfig {
    pub app_host: String,
    pub app_port: u16,
    pub database_url: String,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    pub jwt_access_ttl_minutes: i64,
    pub jwt_refresh_ttl_days: i64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            app_host: std::env::var("APP_HOST")?,
            app_port: std::env::var("APP_PORT")?.parse().unwrap_or(3000),
            database_url: std::env::var("DATABASE_URL")?,
            jwt_access_secret: std::env::var("JWT_ACCESS_SECRET")?,
            jwt_refresh_secret: std::env::var("JWT_REFRESH_SECRET")?,
            jwt_access_ttl_minutes: std::env::var("JWT_ACCESS_TTL_MINUTES")?.parse().unwrap_or(15),
            jwt_refresh_ttl_days: std::env::var("JWT_REFRESH_TTL_DAYS")?.parse().unwrap_or(30),
        })
    }
}
