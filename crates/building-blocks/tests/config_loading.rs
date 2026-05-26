use building_blocks::config::AppConfig;

#[test]
fn config_reads_port_and_database_url() {
    std::env::set_var("APP_HOST", "127.0.0.1");
    std::env::set_var("APP_PORT", "3000");
    std::env::set_var("DATABASE_URL", "postgres://postgres:postgres@127.0.0.1:5432/axum_boilerplate");
    std::env::set_var("JWT_ACCESS_SECRET", "a");
    std::env::set_var("JWT_REFRESH_SECRET", "b");
    std::env::set_var("JWT_ACCESS_TTL_MINUTES", "15");
    std::env::set_var("JWT_REFRESH_TTL_DAYS", "30");

    let config = AppConfig::from_env().expect("config should load");

    assert_eq!(config.app_host, "127.0.0.1");
    assert_eq!(config.app_port, 3000);
    assert!(config.database_url.contains("axum_boilerplate"));
}
