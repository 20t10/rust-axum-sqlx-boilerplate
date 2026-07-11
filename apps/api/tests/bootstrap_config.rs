use api::bootstrap::listener_address;
use api::bootstrap::StartupError;
use building_blocks::config::{AppConfig, ConfigError};
use std::path::PathBuf;

#[test]
fn listener_address_comes_from_app_config() {
    let config = AppConfig {
        app_host: "127.0.0.2".to_owned(),
        app_port: 4321,
        database_url: "postgres://example".to_owned(),
        jwt_access_secret: "access".to_owned(),
        jwt_refresh_secret: "refresh".to_owned(),
        jwt_access_ttl_minutes: 15,
        jwt_refresh_ttl_days: 30,
    };

    assert_eq!(
        listener_address(&config).unwrap().to_string(),
        "127.0.0.2:4321"
    );
}

#[test]
fn startup_error_debug_includes_actionable_configuration_details() {
    let error = StartupError::Config(ConfigError::DotenvFile {
        path: PathBuf::from("/tmp/missing/.env"),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
    });

    let diagnostic = format!("{error:?}");
    assert!(diagnostic.contains("could not read required dotenv file /tmp/missing/.env"));
}
