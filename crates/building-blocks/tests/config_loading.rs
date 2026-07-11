use building_blocks::config::{load_required_dotenv_from, ConfigError};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const COMPLETE_ENV: &str = "APP_HOST=127.0.0.1\nAPP_PORT=4100\nPOSTGRES_USER=postgres\nPOSTGRES_PASSWORD=postgres\nPOSTGRES_DB=axum_boilerplate\nPOSTGRES_PORT=5432\nDATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/axum_boilerplate\nJWT_ACCESS_SECRET=test-access\nJWT_REFRESH_SECRET=test-refresh\nJWT_ACCESS_TTL_MINUTES=15\nJWT_REFRESH_TTL_DAYS=30\nRUST_LOG=info\n";

struct TempEnv {
    dir: PathBuf,
}

impl TempEnv {
    fn new(contents: &str) -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("strict-dotenv-{}-{id}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join(".env"), contents).unwrap();
        Self { dir }
    }

    fn path(&self) -> &Path {
        &self.dir
    }
}

impl Drop for TempEnv {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn loads_complete_dotenv_and_builds_socket_address() {
    let fixture = TempEnv::new(COMPLETE_ENV);
    let config = load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap();
    assert_eq!(config.socket_addr().unwrap().to_string(), "127.0.0.1:4100");
}

#[test]
fn rejects_missing_dotenv_file() {
    let fixture = TempEnv::new(COMPLETE_ENV);
    let error =
        load_required_dotenv_from(&fixture.path().join("missing.env"), Vec::new()).unwrap_err();
    assert!(matches!(error, ConfigError::DotenvFile { .. }));
}

#[test]
fn rejects_duplicate_dotenv_keys_without_leaking_values() {
    let fixture = TempEnv::new(&format!(
        "{COMPLETE_ENV}JWT_ACCESS_SECRET=do-not-leak\nAPP_PORT=9999\n"
    ));
    let error = load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap_err();
    assert!(
        matches!(&error, ConfigError::DuplicateDotenvKeys(keys) if keys == &vec!["APP_PORT".to_owned(), "JWT_ACCESS_SECRET".to_owned()])
    );
    assert!(!error.to_string().contains("do-not-leak"));
}

#[test]
fn rejects_sorted_environment_collisions_without_leaking_values() {
    let fixture = TempEnv::new(COMPLETE_ENV);
    let error = load_required_dotenv_from(
        &fixture.path().join(".env"),
        vec!["JWT_ACCESS_SECRET".to_owned(), "APP_PORT".to_owned()],
    )
    .unwrap_err();
    assert!(
        matches!(&error, ConfigError::EnvironmentCollision(keys) if keys == &vec!["APP_PORT".to_owned(), "JWT_ACCESS_SECRET".to_owned()])
    );
    assert!(!error.to_string().contains("test-access"));
}

#[test]
fn rejects_missing_blank_invalid_and_non_positive_values() {
    let cases = [
        (COMPLETE_ENV.replace("RUST_LOG=info\n", ""), "RUST_LOG"),
        (
            COMPLETE_ENV.replace(
                "DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/axum_boilerplate",
                "DATABASE_URL=   ",
            ),
            "DATABASE_URL",
        ),
        (
            COMPLETE_ENV.replace("APP_PORT=4100", "APP_PORT=nope"),
            "APP_PORT",
        ),
        (
            COMPLETE_ENV.replace("JWT_ACCESS_TTL_MINUTES=15", "JWT_ACCESS_TTL_MINUTES=0"),
            "JWT_ACCESS_TTL_MINUTES",
        ),
        (
            COMPLETE_ENV.replace("APP_HOST=127.0.0.1", "APP_HOST=localhost"),
            "APP_HOST",
        ),
    ];
    for (contents, key) in cases {
        let fixture = TempEnv::new(&contents);
        let error =
            load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap_err();
        assert!(error.to_string().contains(key), "unexpected error: {error}");
    }
}

#[test]
fn rejects_malformed_dotenv() {
    let fixture = TempEnv::new("'unterminated=value\n");
    let error = load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap_err();
    assert!(matches!(error, ConfigError::DotenvSyntax { .. }));
}

#[test]
fn rejects_dotenv_interpolation_without_reading_the_process_environment() {
    for interpolation in ["$PATH", "${PATH}"] {
        let secret = format!("ordinary-value-{interpolation}-must-not-leak");
        let contents = COMPLETE_ENV.replace("RUST_LOG=info", &format!("RUST_LOG={secret}"));
        let fixture = TempEnv::new(&contents);

        let error =
            load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap_err();

        assert!(matches!(&error, ConfigError::DotenvInterpolation { .. }));
        assert!(!error.to_string().contains(&secret));
        assert!(!format!("{error:?}").contains(&secret));
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn rejects_every_unescaped_dollar_that_dotenvy_treats_as_substitution() {
    for value in ["costs-$-5", "trailing-$"] {
        let contents = COMPLETE_ENV.replace("RUST_LOG=info", &format!("RUST_LOG={value}"));
        let fixture = TempEnv::new(&contents);

        let error =
            load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap_err();

        assert!(matches!(&error, ConfigError::DotenvInterpolation { .. }));
        assert!(!error.to_string().contains(value));
        assert!(!format!("{error:?}").contains(value));
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn permits_literal_dollars_that_dotenvy_does_not_substitute() {
    for value in [r"costs-\$5", "'costs-$5'"] {
        let contents = COMPLETE_ENV.replace("RUST_LOG=info", &format!("RUST_LOG={value}"));
        let fixture = TempEnv::new(&contents);

        load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap();
    }
}

#[test]
fn permits_dollars_in_comments() {
    let fixture = TempEnv::new(&format!("{COMPLETE_ENV}# costs $5\n"));

    load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap();
}

#[test]
fn permits_dollars_in_consecutive_comment_lines() {
    let fixture = TempEnv::new(&format!("{COMPLETE_ENV}# first\n# costs $5\n"));

    load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap();
}

#[test]
fn rejects_dollars_after_non_comment_hashes() {
    let contents = COMPLETE_ENV.replace("RUST_LOG=info", "RUST_LOG=info#cost$5");
    let fixture = TempEnv::new(&contents);

    let error = load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap_err();

    assert!(matches!(error, ConfigError::DotenvInterpolation { .. }));
}
