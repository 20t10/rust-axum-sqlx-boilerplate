use crate::jwt::JwtSettings;
use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
};

const REQUIRED_KEYS: [&str; 12] = [
    "APP_HOST",
    "APP_PORT",
    "POSTGRES_USER",
    "POSTGRES_PASSWORD",
    "POSTGRES_DB",
    "POSTGRES_PORT",
    "DATABASE_URL",
    "JWT_ACCESS_SECRET",
    "JWT_REFRESH_SECRET",
    "JWT_ACCESS_TTL_MINUTES",
    "JWT_REFRESH_TTL_DAYS",
    "RUST_LOG",
];

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("could not read required dotenv file {path}")]
    DotenvFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid dotenv syntax in {path} at index {index}")]
    DotenvSyntax { path: PathBuf, index: usize },
    #[error("dotenv interpolation is not supported in {path} at index {index}")]
    DotenvInterpolation { path: PathBuf, index: usize },
    #[error("duplicate keys in dotenv file: {0:?}")]
    DuplicateDotenvKeys(Vec<String>),
    #[error("dotenv keys already exist in process environment: {0:?}")]
    EnvironmentCollision(Vec<String>),
    #[error("missing environment variable {0}")]
    MissingEnv(&'static str),
    #[error("environment variable {0} must not be blank")]
    BlankEnv(&'static str),
    #[error("invalid environment variable {name}")]
    InvalidNumber {
        name: &'static str,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error("environment variable {0} must be positive")]
    NonPositive(&'static str),
    #[error("invalid IP address in APP_HOST")]
    InvalidHost(#[source] std::net::AddrParseError),
    #[error("could not resolve current directory")]
    CurrentDir(#[source] std::io::Error),
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
        let values = REQUIRED_KEYS
            .iter()
            .filter_map(|&key| std::env::var(key).ok().map(|value| (key.to_owned(), value)))
            .collect();
        config_from_values(&values)
    }

    pub fn socket_addr(&self) -> Result<SocketAddr, ConfigError> {
        let host = self
            .app_host
            .parse::<IpAddr>()
            .map_err(ConfigError::InvalidHost)?;
        Ok(SocketAddr::new(host, self.app_port))
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

pub fn load_required_dotenv() -> Result<AppConfig, ConfigError> {
    let path = std::env::current_dir()
        .map_err(ConfigError::CurrentDir)?
        .join(".env");
    let process_keys = std::env::vars_os().filter_map(|(key, _)| key.into_string().ok());
    let (config, values) = parse_required_dotenv(&path, process_keys)?;

    for (key, value) in values {
        std::env::set_var(key, value);
    }

    Ok(config)
}

pub fn load_required_dotenv_from(
    path: &Path,
    process_keys: impl IntoIterator<Item = String>,
) -> Result<AppConfig, ConfigError> {
    parse_required_dotenv(path, process_keys).map(|(config, _)| config)
}

fn parse_required_dotenv(
    path: &Path,
    process_keys: impl IntoIterator<Item = String>,
) -> Result<(AppConfig, BTreeMap<String, String>), ConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::DotenvFile {
        path: path.to_owned(),
        source,
    })?;

    // dotenvy enters substitution mode for every unescaped `$` outside single quotes, even
    // a trailing `$` or `$-`. This loader's file-based entry point is intentionally pure, so
    // reject those dollars before dotenvy sees any source text. Escaped and single-quoted
    // dollars remain supported because dotenvy treats them as literals. The error retains
    // only the byte index, never the line/value.
    if let Some(index) = dotenv_interpolation_index(&contents) {
        return Err(ConfigError::DotenvInterpolation {
            path: path.to_owned(),
            index,
        });
    }

    let iterator = dotenvy::from_read_iter(contents.as_bytes());
    let mut values = BTreeMap::new();
    let mut duplicates = BTreeSet::new();

    for entry in iterator {
        let (key, value) = entry.map_err(|error| map_dotenv_error(path, error))?;
        if values.insert(key.clone(), value).is_some() {
            duplicates.insert(key);
        }
    }

    if !duplicates.is_empty() {
        return Err(ConfigError::DuplicateDotenvKeys(
            duplicates.into_iter().collect(),
        ));
    }

    let process_keys: BTreeSet<_> = process_keys.into_iter().collect();
    let collisions = values
        .keys()
        .filter(|key| process_keys.contains(*key))
        .cloned()
        .collect::<Vec<_>>();
    if !collisions.is_empty() {
        return Err(ConfigError::EnvironmentCollision(collisions));
    }

    for &key in &REQUIRED_KEYS {
        required_value(&values, key)?;
    }

    let config = config_from_values(&values)?;
    Ok((config, values))
}

fn dotenv_interpolation_index(contents: &str) -> Option<usize> {
    let mut single_quoted = false;
    let mut double_quoted = false;
    let mut escaped = false;

    for (index, character) in contents.char_indices() {
        if single_quoted {
            if character == '\'' {
                single_quoted = false;
            }
            continue;
        }

        if escaped {
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            '\'' if !double_quoted => single_quoted = true,
            '"' => double_quoted = !double_quoted,
            '$' => return Some(index),
            _ => {}
        }
    }
    None
}

fn map_dotenv_error(path: &Path, error: dotenvy::Error) -> ConfigError {
    match error {
        dotenvy::Error::LineParse(_, index) => ConfigError::DotenvSyntax {
            path: path.to_owned(),
            index,
        },
        dotenvy::Error::Io(source) => ConfigError::DotenvFile {
            path: path.to_owned(),
            source,
        },
        _ => ConfigError::DotenvFile {
            path: path.to_owned(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, "dotenv parsing failed"),
        },
    }
}

fn config_from_values(values: &BTreeMap<String, String>) -> Result<AppConfig, ConfigError> {
    let config = AppConfig {
        app_host: required_value(values, "APP_HOST")?.to_owned(),
        app_port: parse_required(values, "APP_PORT")?,
        database_url: required_value(values, "DATABASE_URL")?.to_owned(),
        jwt_access_secret: required_value(values, "JWT_ACCESS_SECRET")?.to_owned(),
        jwt_refresh_secret: required_value(values, "JWT_REFRESH_SECRET")?.to_owned(),
        jwt_access_ttl_minutes: parse_positive(values, "JWT_ACCESS_TTL_MINUTES")?,
        jwt_refresh_ttl_days: parse_positive(values, "JWT_REFRESH_TTL_DAYS")?,
    };
    config.socket_addr()?;
    Ok(config)
}

fn required_value<'a>(
    values: &'a BTreeMap<String, String>,
    name: &'static str,
) -> Result<&'a str, ConfigError> {
    let value = values.get(name).ok_or(ConfigError::MissingEnv(name))?;
    if value.trim().is_empty() {
        return Err(ConfigError::BlankEnv(name));
    }
    Ok(value)
}

fn parse_required<T>(
    values: &BTreeMap<String, String>,
    name: &'static str,
) -> Result<T, ConfigError>
where
    T: std::str::FromStr<Err = std::num::ParseIntError>,
{
    required_value(values, name)?
        .parse()
        .map_err(|source| ConfigError::InvalidNumber { name, source })
}

fn parse_positive(
    values: &BTreeMap<String, String>,
    name: &'static str,
) -> Result<i64, ConfigError> {
    let value = parse_required(values, name)?;
    if value <= 0 {
        return Err(ConfigError::NonPositive(name));
    }
    Ok(value)
}
