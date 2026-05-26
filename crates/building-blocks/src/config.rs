use crate::jwt::JwtSettings;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("missing environment variable {0}")]
    MissingEnv(&'static str),
    #[error("invalid environment variable {name}: {source}")]
    InvalidNumber {
        name: &'static str,
        #[source]
        source: std::num::ParseIntError,
    },
}

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
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            app_host: required_env("APP_HOST")?,
            app_port: parse_required("APP_PORT")?,
            database_url: required_env("DATABASE_URL")?,
            jwt_access_secret: required_env("JWT_ACCESS_SECRET")?,
            jwt_refresh_secret: required_env("JWT_REFRESH_SECRET")?,
            jwt_access_ttl_minutes: parse_required("JWT_ACCESS_TTL_MINUTES")?,
            jwt_refresh_ttl_days: parse_required("JWT_REFRESH_TTL_DAYS")?,
        })
    }

    pub fn jwt_settings(&self) -> JwtSettings {
        JwtSettings {
            access_secret: self.jwt_access_secret.clone(),
            refresh_secret: self.jwt_refresh_secret.clone(),
            access_ttl_minutes: self.jwt_access_ttl_minutes,
            refresh_ttl_days: self.jwt_refresh_ttl_days,
        }
    }
}

fn required_env(name: &'static str) -> Result<String, ConfigError> {
    std::env::var(name).map_err(|_| ConfigError::MissingEnv(name))
}

fn parse_required<T>(name: &'static str) -> Result<T, ConfigError>
where
    T: std::str::FromStr<Err = std::num::ParseIntError>,
{
    required_env(name)?
        .parse()
        .map_err(|source| ConfigError::InvalidNumber { name, source })
}
