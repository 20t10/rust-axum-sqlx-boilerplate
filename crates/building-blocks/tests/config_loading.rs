use building_blocks::config::AppConfig;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn set_common_test_env() {
    std::env::set_var("APP_HOST", env_or("TEST_APP_HOST", "127.0.0.1"));
    std::env::set_var("APP_PORT", env_or("TEST_APP_PORT", "3000"));
    std::env::set_var(
        "DATABASE_URL",
        env_or("TEST_DATABASE_URL", &default_database_url()),
    );
    std::env::set_var(
        "JWT_ACCESS_SECRET",
        env_or("TEST_JWT_ACCESS_SECRET", "test-access"),
    );
    std::env::set_var(
        "JWT_REFRESH_SECRET",
        env_or("TEST_JWT_REFRESH_SECRET", "test-refresh"),
    );
    std::env::set_var(
        "JWT_ACCESS_TTL_MINUTES",
        env_or("TEST_JWT_ACCESS_TTL_MINUTES", "15"),
    );
    std::env::set_var(
        "JWT_REFRESH_TTL_DAYS",
        env_or("TEST_JWT_REFRESH_TTL_DAYS", "30"),
    );
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn default_database_url() -> String {
    let user = env_or("TEST_DB_USER", "postgres");
    let password = env_or("TEST_DB_PASSWORD", "postgres");
    let host = env_or("TEST_DB_HOST", "127.0.0.1");
    let port = env_or("TEST_DB_PORT", "5432");
    let database = env_or("TEST_DB_NAME", "axum_boilerplate");

    format!("postgres://{user}:{password}@{host}:{port}/{database}")
}

#[test]
fn config_reads_port_and_database_url() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    set_common_test_env();

    let config = AppConfig::from_env().expect("config should load");

    assert_eq!(config.app_host, "127.0.0.1");
    assert_eq!(config.app_port, 3000);
    assert!(config.database_url.contains("axum_boilerplate"));
    assert_eq!(config.jwt_settings().access_secret, "test-access");
    assert_eq!(config.jwt_settings().refresh_secret, "test-refresh");
}

#[test]
fn config_rejects_invalid_numeric_values() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    set_common_test_env();
    std::env::set_var("APP_PORT", env_or("TEST_INVALID_APP_PORT", "not-a-port"));

    let error = AppConfig::from_env().expect_err("invalid numeric config should fail");

    assert!(error.to_string().contains("APP_PORT"));
}
