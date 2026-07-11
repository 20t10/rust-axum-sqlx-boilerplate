# Strict `.env` Startup Configuration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Require a collision-free `.env` file, validate every startup setting, and remove hardcoded API and Docker host/port defaults.

**Architecture:** `building-blocks::config` will own a strict dotenv loader with a pure validation core and a small process-environment adapter. The API bootstrap will load configuration before logging and bind a validated `SocketAddr`; Docker and Make targets will consume the same mandatory file without interpolation fallbacks.

**Tech Stack:** Rust 2021, `dotenvy`, `thiserror`, Tokio, Axum, Docker Compose, GNU Make

## Global Constraints

- A real `.env` file in the current working directory is mandatory.
- Every key has exactly one source; a key present in both `.env` and the process environment is a startup error.
- Duplicate keys inside `.env` are errors.
- Error output may include paths and key names but must never include configuration values.
- Required strings are non-blank, `APP_PORT` is a `u16`, and both JWT TTL values are positive `i64` values.
- `APP_HOST` must be an IP address so `APP_HOST` plus `APP_PORT` produces a validated `SocketAddr` before bind.
- Docker Compose must not provide defaults for PostgreSQL settings.
- Follow RED → GREEN → REFACTOR and do not write production behavior before its failing test.

## File structure

- Modify `Cargo.toml`: expose `dotenvy` as a shared dependency.
- Modify `crates/building-blocks/Cargo.toml`: consume `dotenvy`.
- Modify `crates/building-blocks/src/config.rs`: own dotenv parsing, collision detection, required-value validation, typed errors, and `SocketAddr` creation.
- Replace `crates/building-blocks/tests/config_loading.rs`: test the pure loader boundary with temporary files and explicit process keys, avoiding parallel mutation of global environment.
- Modify `apps/api/src/bootstrap.rs`: load `.env`, initialize logging, build the router, and bind the configured address with a typed startup error.
- Modify `apps/api/src/main.rs`: return `StartupError` from `main` instead of calling `expect`.
- Modify `docker-compose.yml`: remove PostgreSQL interpolation defaults.
- Modify `Makefile`: require `.env` and export it for commands that consume configuration.
- Modify `README.md`: document mandatory-file and collision behavior.
- Modify `tests/init_project.rs`: assert generated projects retain strict Compose interpolation and required `.env` content.

---

### Task 1: Strict dotenv loader and typed configuration

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/building-blocks/Cargo.toml`
- Modify: `crates/building-blocks/src/config.rs`
- Replace: `crates/building-blocks/tests/config_loading.rs`

**Interfaces:**
- Produces: `pub fn load_required_dotenv() -> Result<AppConfig, ConfigError>`
- Produces: `pub fn load_required_dotenv_from(path: &Path, process_keys: impl IntoIterator<Item = String>) -> Result<AppConfig, ConfigError>` for deterministic loader tests; this function never reads or mutates the real process environment.
- Produces: `pub fn AppConfig::socket_addr(&self) -> Result<SocketAddr, ConfigError>`
- Produces: `ConfigError` variants `DotenvFile`, `DotenvSyntax`, `DuplicateDotenvKeys`, `EnvironmentCollision`, `MissingEnv`, `BlankEnv`, `InvalidNumber`, `NonPositive`, and `InvalidHost`.

- [ ] **Step 1: Add failing loader tests using a temporary-file helper**

Replace `crates/building-blocks/tests/config_loading.rs` with tests whose helper creates a unique directory under `std::env::temp_dir()`, writes `.env`, calls a pure parser/validator entry point, and removes the directory in `Drop`. The test module must include these cases and exact assertions:

```rust
use building_blocks::config::{load_required_dotenv_from, ConfigError};
use std::{fs, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

const COMPLETE_ENV: &str = "APP_HOST=127.0.0.1\nAPP_PORT=4100\nPOSTGRES_USER=postgres\nPOSTGRES_PASSWORD=postgres\nPOSTGRES_DB=axum_boilerplate\nPOSTGRES_PORT=5432\nDATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/axum_boilerplate\nJWT_ACCESS_SECRET=test-access\nJWT_REFRESH_SECRET=test-refresh\nJWT_ACCESS_TTL_MINUTES=15\nJWT_REFRESH_TTL_DAYS=30\nRUST_LOG=info\n";

struct TempEnv { dir: PathBuf }

impl TempEnv {
    fn new(contents: &str) -> Self {
        let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("strict-dotenv-{}-{id}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join(".env"), contents).unwrap();
        Self { dir }
    }

    fn path(&self) -> &Path { &self.dir }
}

impl Drop for TempEnv {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.dir); }
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
    let error = load_required_dotenv_from(&fixture.path().join("missing.env"), Vec::new()).unwrap_err();
    assert!(matches!(error, ConfigError::DotenvFile { .. }));
}

#[test]
fn rejects_duplicate_dotenv_keys_without_leaking_values() {
    let fixture = TempEnv::new(&format!("{COMPLETE_ENV}JWT_ACCESS_SECRET=do-not-leak\nAPP_PORT=9999\n"));
    let error = load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap_err();
    assert!(matches!(&error, ConfigError::DuplicateDotenvKeys(keys) if keys == &vec!["APP_PORT".to_owned(), "JWT_ACCESS_SECRET".to_owned()]));
    assert!(!error.to_string().contains("do-not-leak"));
}

#[test]
fn rejects_sorted_environment_collisions_without_leaking_values() {
    let fixture = TempEnv::new(COMPLETE_ENV);
    let error = load_required_dotenv_from(
        &fixture.path().join(".env"),
        vec!["JWT_ACCESS_SECRET".to_owned(), "APP_PORT".to_owned()],
    ).unwrap_err();
    assert!(matches!(&error, ConfigError::EnvironmentCollision(keys) if keys == &vec!["APP_PORT".to_owned(), "JWT_ACCESS_SECRET".to_owned()]));
    assert!(!error.to_string().contains("test-access"));
}

#[test]
fn rejects_missing_blank_invalid_and_non_positive_values() {
    let cases = [
        (COMPLETE_ENV.replace("RUST_LOG=info\n", ""), "RUST_LOG"),
        (COMPLETE_ENV.replace("DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/axum_boilerplate", "DATABASE_URL=   "), "DATABASE_URL"),
        (COMPLETE_ENV.replace("APP_PORT=4100", "APP_PORT=nope"), "APP_PORT"),
        (COMPLETE_ENV.replace("JWT_ACCESS_TTL_MINUTES=15", "JWT_ACCESS_TTL_MINUTES=0"), "JWT_ACCESS_TTL_MINUTES"),
        (COMPLETE_ENV.replace("APP_HOST=127.0.0.1", "APP_HOST=localhost"), "APP_HOST"),
    ];
    for (contents, key) in cases {
        let fixture = TempEnv::new(&contents);
        let error = load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap_err();
        assert!(error.to_string().contains(key), "unexpected error: {error}");
    }
}

#[test]
fn rejects_malformed_dotenv() {
    let fixture = TempEnv::new("'unterminated=value\n");
    let error = load_required_dotenv_from(&fixture.path().join(".env"), Vec::new()).unwrap_err();
    assert!(matches!(error, ConfigError::DotenvSyntax { .. }));
}
```

- [ ] **Step 2: Run the loader tests and verify RED**

Run: `cargo test -p building-blocks --test config_loading`

Expected: compilation fails because `load_required_dotenv_from`, the new `ConfigError` variants, and `AppConfig::socket_addr` do not exist.

- [ ] **Step 3: Add dotenvy dependencies**

Add `dotenvy = "0.15"` to `[workspace.dependencies]` in `Cargo.toml`, then add `dotenvy.workspace = true` to `[dependencies]` in `crates/building-blocks/Cargo.toml`.

- [ ] **Step 4: Implement the minimal strict loader**

In `crates/building-blocks/src/config.rs`:

1. Define `REQUIRED_KEYS` containing all twelve keys from `.env.example`.
2. Parse with `dotenvy::from_path_iter(path)` and collect entries into a `BTreeMap<String, String>` while separately collecting repeated names into a `BTreeSet`.
3. Map file-open errors to `DotenvFile { path, source }`. For `dotenvy::Error::LineParse`, discard the source line (which may contain a secret) and retain only its error index in `DotenvSyntax { path, index }`. Map iterator I/O failures to `DotenvFile`. Do not store a `dotenvy::Error` because its `Debug` and `Display` representations may expose the source line.
4. Intersect parsed names with the supplied `process_keys` set and return sorted `EnvironmentCollision(Vec<String>)` before installing any values.
5. Validate every `REQUIRED_KEYS` entry exists and is not blank.
6. Construct `AppConfig` directly from the parsed map so the pure test entry point does not mutate global state.
7. Parse the port as `u16`, TTLs as `i64`, reject TTL values `<= 0`, and parse `APP_HOST` as `IpAddr` in `socket_addr`. Call `socket_addr` during config construction so an invalid host fails loading rather than being deferred until bootstrap.
8. Implement `load_required_dotenv()` by resolving `current_dir()?.join(".env")`, capturing `std::env::vars_os()` names before loading, calling the strict parser, and only after successful validation calling `std::env::set_var` for every parsed pair. Keep a private helper returning `(AppConfig, BTreeMap<String, String>)` so values are installed exactly once.

Use errors with value-safe displays equivalent to:

```rust
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("could not read required dotenv file {path}")]
    DotenvFile { path: PathBuf, #[source] source: std::io::Error },
    #[error("invalid dotenv syntax in {path} at index {index}")]
    DotenvSyntax { path: PathBuf, index: usize },
    #[error("duplicate keys in dotenv file: {0:?}")]
    DuplicateDotenvKeys(Vec<String>),
    #[error("dotenv keys already exist in process environment: {0:?}")]
    EnvironmentCollision(Vec<String>),
    #[error("missing environment variable {0}")]
    MissingEnv(&'static str),
    #[error("environment variable {0} must not be blank")]
    BlankEnv(&'static str),
    #[error("invalid environment variable {name}")]
    InvalidNumber { name: &'static str, #[source] source: std::num::ParseIntError },
    #[error("environment variable {0} must be positive")]
    NonPositive(&'static str),
    #[error("invalid IP address in APP_HOST")]
    InvalidHost(#[source] std::net::AddrParseError),
    #[error("could not resolve current directory")]
    CurrentDir(#[source] std::io::Error),
}
```

Keep `AppConfig::from_env()` for compatibility with existing callers, but route its field validation through the same private helpers and add blank/positive/host validation so it cannot bypass the new invariants.

- [ ] **Step 5: Run focused tests and verify GREEN**

Run: `cargo test -p building-blocks --test config_loading`

Expected: all strict loader tests pass and no configuration value appears in failure output.

- [ ] **Step 6: Run building-block tests and lint**

Run: `cargo test -p building-blocks && cargo clippy -p building-blocks --all-targets -- -D warnings`

Expected: both commands exit 0 without warnings.

- [ ] **Step 7: Commit Task 1**

```bash
git add Cargo.toml Cargo.lock crates/building-blocks/Cargo.toml crates/building-blocks/src/config.rs crates/building-blocks/tests/config_loading.rs
git commit -m "feat: add strict dotenv configuration loader"
```

---

### Task 2: Wire validated configuration into API startup

**Files:**
- Modify: `apps/api/Cargo.toml`
- Modify: `apps/api/src/bootstrap.rs`
- Modify: `apps/api/src/main.rs`
- Create: `apps/api/tests/bootstrap_config.rs`

**Interfaces:**
- Consumes: `building_blocks::config::{load_required_dotenv, AppConfig, ConfigError}`
- Produces: `pub enum StartupError { Config(ConfigError), Bind(std::io::Error), Serve(std::io::Error) }`
- Produces: `pub fn listener_address(config: &AppConfig) -> Result<SocketAddr, ConfigError>`
- Keeps: `pub async fn run() -> Result<(), StartupError>`

- [ ] **Step 1: Write a failing bootstrap-address test**

Create `apps/api/tests/bootstrap_config.rs`:

```rust
use api::bootstrap::listener_address;
use building_blocks::config::AppConfig;

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

    assert_eq!(listener_address(&config).unwrap().to_string(), "127.0.0.2:4321");
}
```

- [ ] **Step 2: Run the API test and verify RED**

Run: `cargo test -p api --test bootstrap_config`

Expected: compilation fails because `api::bootstrap::listener_address` does not exist.

- [ ] **Step 3: Implement typed, configured startup**

Replace hardcoded binding in `apps/api/src/bootstrap.rs` with this flow:

```rust
use building_blocks::config::{load_required_dotenv, AppConfig, ConfigError};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(thiserror::Error, Debug)]
pub enum StartupError {
    #[error("startup configuration failed")]
    Config(#[from] ConfigError),
    #[error("failed to bind API listener")]
    Bind(#[source] std::io::Error),
    #[error("API server failed")]
    Serve(#[source] std::io::Error),
}

pub fn listener_address(config: &AppConfig) -> Result<SocketAddr, ConfigError> {
    config.socket_addr()
}

pub async fn run() -> Result<(), StartupError> {
    let config = load_required_dotenv()?;
    building_blocks::logging::init_logging();
    let address = listener_address(&config)?;
    let app = crate::router::build_router();
    let listener = TcpListener::bind(address).await.map_err(StartupError::Bind)?;
    axum::serve(listener, app).await.map_err(StartupError::Serve)
}
```

Add `thiserror.workspace = true` to `apps/api/Cargo.toml`, because the API crate owns `StartupError`.

Change `apps/api/src/main.rs` to propagate the error and exit non-zero without an `expect` panic:

```rust
#[tokio::main]
async fn main() -> Result<(), api::bootstrap::StartupError> {
    api::bootstrap::run().await
}
```

- [ ] **Step 4: Run focused tests and verify GREEN**

Run: `cargo test -p api --test bootstrap_config && cargo test -p api`

Expected: the bootstrap config test and all API tests pass.

- [ ] **Step 5: Manually verify missing `.env` fails before bind**

Run from a temporary directory so the repository's local `.env` cannot be found:

```bash
repo="$(pwd)"
tmp="$(mktemp -d)"
(cd "$tmp" && cargo run --manifest-path "$repo/Cargo.toml" -p api)
status=$?
rm -rf "$tmp"
test "$status" -ne 0
```

Expected: non-zero exit with `could not read required dotenv file`; it must not mention `127.0.0.1:3000` or start listening.

- [ ] **Step 6: Commit Task 2**

```bash
git add apps/api/Cargo.toml apps/api/src/bootstrap.rs apps/api/src/main.rs apps/api/tests/bootstrap_config.rs Cargo.lock
git commit -m "feat: fail fast on invalid API startup config"
```

---

### Task 3: Make Docker and developer commands strict

**Files:**
- Modify: `docker-compose.yml`
- Modify: `Makefile`
- Modify: `README.md`
- Modify: `tests/init_project.rs`

**Interfaces:**
- Consumes: `.env.example` as the only local configuration template.
- Produces: Make targets that fail with a clear message when `.env` is absent; only commands such as `psql` that cannot read dotenv themselves source it in their own subprocess.
- Produces: Docker Compose references `${POSTGRES_USER:?set POSTGRES_USER in .env}` and equivalent expressions for every required PostgreSQL setting.

- [ ] **Step 1: Extend the generated-project test and verify RED**

In `tests/init_project.rs`, extend the existing generated-file assertions to read `docker-compose.yml` and assert:

```rust
assert!(compose.contains("${POSTGRES_USER:?set POSTGRES_USER in .env}"));
assert!(compose.contains("${POSTGRES_PASSWORD:?set POSTGRES_PASSWORD in .env}"));
assert!(compose.contains("${POSTGRES_DB:?set POSTGRES_DB in .env}"));
assert!(compose.contains("${POSTGRES_PORT:?set POSTGRES_PORT in .env}"));
assert!(!compose.contains(":-"));
```

Also assert the generated `.env.example` contains all twelve keys, including `APP_HOST`, `APP_PORT`, `DATABASE_URL`, both JWT secrets, both TTLs, `RUST_LOG`, and the four PostgreSQL keys.

- [ ] **Step 2: Run the generated-project test and verify RED**

Run: `cargo test --test init_project`

Expected: failure because `docker-compose.yml` still contains `${VAR:-default}` expressions.

- [ ] **Step 3: Remove Compose defaults and require `.env` in Make**

Change every Docker Compose PostgreSQL expression to required interpolation:

```yaml
environment:
  POSTGRES_USER: ${POSTGRES_USER:?set POSTGRES_USER in .env}
  POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:?set POSTGRES_PASSWORD in .env}
  POSTGRES_DB: ${POSTGRES_DB:?set POSTGRES_DB in .env}
ports:
  - "${POSTGRES_PORT:?set POSTGRES_PORT in .env}:5432"
```

Use the same required expressions in the healthcheck command.

At the top of `Makefile`, after `.PHONY`, require `.env` for all targets except `setup` and `init-project` by using a reusable guard rather than silently defaulting:

```make
ifneq ($(filter setup init-project,$(MAKECMDGOALS)),)
else
ifeq (,$(wildcard .env))
$(error missing required .env; run 'make setup')
endif
endif
```

Do not `include` or globally export `.env`: doing so would make `make dev` inject the same keys into Cargo's process environment and correctly trigger the API collision error. Keep `setup` as the explicit operation that creates `.env`. Change `seed` so only its shell subprocess sources the file:

```make
seed:
	@set -a; . ./.env; set +a; psql "$$DATABASE_URL" -f seeds/dev.sql
```

`cargo run` loads `.env` through the application, Docker Compose reads `.env` itself, and SQLx CLI reads dotenv itself.

- [ ] **Step 4: Document the strict startup contract**

Add a `Configuration` section to `README.md` stating:

```markdown
## Configuration

Run `make setup` to create the mandatory `.env` from `.env.example`. The API refuses to start when `.env` is missing, malformed, incomplete, or contains a key that is already exported in the process environment. Keep runtime values in `.env`; do not export the same keys in your shell. Errors list conflicting key names but never their values.

All values in `.env.example` are examples. Replace database credentials and JWT secrets before using the project outside local development.
```

- [ ] **Step 5: Run tooling tests and verify GREEN**

Run: `cargo test --test init_project`

Expected: generated-project tests pass.

Run: `docker compose --env-file .env.example config >/dev/null`

Expected: exit 0 with all required Compose values sourced from `.env.example`.

Run: `tmp="$(mktemp -d)" && cp Makefile "$tmp/" && (cd "$tmp" && make dev); status=$?; rm -rf "$tmp"; test "$status" -ne 0`

Expected: Make fails with `missing required .env; run 'make setup'`.

- [ ] **Step 6: Commit Task 3**

```bash
git add docker-compose.yml Makefile README.md tests/init_project.rs
git commit -m "chore: require dotenv configuration in local tooling"
```

---

### Task 4: Full verification and hardcode audit

**Files:**
- Modify only files implicated by verification failures; do not broaden feature scope.

**Interfaces:**
- Consumes: all deliverables from Tasks 1–3.
- Produces: a formatted, lint-clean, test-clean workspace with no runtime API bind literal or Compose fallback.

- [ ] **Step 1: Run formatting, lint, and all tests**

Run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all commands exit 0 with no warnings or test failures.

- [ ] **Step 2: Audit runtime hardcodes and fallbacks**

Run:

```bash
rg -n '(127\.0\.0\.1:3000|\$\{POSTGRES_(USER|PASSWORD|DB|PORT):-)' \
  apps crates docker-compose.yml Makefile
```

Expected: no matches. Literals in `.env.example`, tests, seeds, and the project-renaming script are permitted because they are examples or fixtures, not runtime fallbacks.

- [ ] **Step 3: Verify the working tree contains only intended changes**

Run: `git status --short && git diff --check`

Expected: no uncommitted changes. If formatting or verification required a correction, commit only those scoped corrections:

```bash
git add <scoped-files>
git commit -m "fix: complete strict startup configuration"
```
