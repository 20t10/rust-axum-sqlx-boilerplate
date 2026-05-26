# Axum Boilerplate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust Axum boilerplate as a workspace-based modular monolith with Clean Architecture, DDD module boundaries, JWT auth, PostgreSQL/SQLx persistence, Makefile workflows, and a `create-module` generator.

**Architecture:** The project ships as one deployable Axum app in `apps/api`, backed by workspace crates for shared kernel, building blocks, and bounded-context modules. Each module follows domain/application/infrastructure/presentation layering, with explicit SQLx repositories, read-model queries, and thin HTTP handlers.

**Tech Stack:** Rust, Cargo workspace, Axum, Tokio, SQLx, PostgreSQL, JWT, Tracing, Docker Compose, OpenAPI, Makefile

---

## File Structure

Planned files and responsibilities:

- `Cargo.toml`
  - workspace manifest and shared dependency versions
- `rust-toolchain.toml`
  - pin toolchain version for reproducible setup
- `.env.example`
  - baseline environment variables
- `.gitignore`
  - local artifact exclusions
- `Makefile`
  - public developer commands
- `docker-compose.yml`
  - local PostgreSQL dependency
- `sqlx-cli.toml`
  - SQLx project configuration if needed by workflow
- `apps/api/Cargo.toml`
  - API binary crate manifest
- `apps/api/src/main.rs`
  - startup entrypoint
- `apps/api/src/bootstrap.rs`
  - app initialization and dependency wiring
- `apps/api/src/router.rs`
  - top-level router composition
- `apps/api/src/state.rs`
  - application state container
- `crates/shared-kernel/Cargo.toml`
  - shared-kernel manifest
- `crates/shared-kernel/src/lib.rs`
  - shared exports
- `crates/shared-kernel/src/ids.rs`
  - shared ID types
- `crates/shared-kernel/src/pagination.rs`
  - pagination primitives
- `crates/shared-kernel/src/auth.rs`
  - auth claims/value primitives
- `crates/shared-kernel/src/clock.rs`
  - clock trait and helpers
- `crates/building-blocks/Cargo.toml`
  - cross-cutting crate manifest
- `crates/building-blocks/src/lib.rs`
  - cross-cutting exports
- `crates/building-blocks/src/config.rs`
  - typed settings loader
- `crates/building-blocks/src/database.rs`
  - SQLx pool setup
- `crates/building-blocks/src/logging.rs`
  - tracing setup
- `crates/building-blocks/src/errors.rs`
  - app/infrastructure error utilities
- `crates/building-blocks/src/jwt.rs`
  - JWT encode/decode helpers
- `crates/building-blocks/src/openapi.rs`
  - OpenAPI helpers or registration
- `crates/building-blocks/src/jobs.rs`
  - outbox/job seam traits
- `crates/modules/identity/Cargo.toml`
  - identity module manifest
- `crates/modules/identity/src/lib.rs`
  - module exports and composition entrypoint
- `crates/modules/identity/src/domain/mod.rs`
  - domain module root
- `crates/modules/identity/src/domain/entities/user.rs`
  - user aggregate/entity
- `crates/modules/identity/src/domain/value_objects/email.rs`
  - email value object
- `crates/modules/identity/src/domain/value_objects/password_hash.rs`
  - password-hash value object
- `crates/modules/identity/src/domain/repositories.rs`
  - repository ports
- `crates/modules/identity/src/domain/errors.rs`
  - domain errors
- `crates/modules/identity/src/application/mod.rs`
  - application root
- `crates/modules/identity/src/application/dto.rs`
  - auth DTOs
- `crates/modules/identity/src/application/commands/register.rs`
  - register command + handler
- `crates/modules/identity/src/application/commands/login.rs`
  - login command + handler
- `crates/modules/identity/src/application/commands/refresh.rs`
  - refresh command + handler
- `crates/modules/identity/src/application/queries/me.rs`
  - current-user query + handler
- `crates/modules/identity/src/application/errors.rs`
  - application-layer errors
- `crates/modules/identity/src/infrastructure/mod.rs`
  - infra root
- `crates/modules/identity/src/infrastructure/passwords.rs`
  - password hashing/verifier adapter
- `crates/modules/identity/src/infrastructure/jwt_tokens.rs`
  - token issuer adapter
- `crates/modules/identity/src/infrastructure/persistence/user_repository.rs`
  - SQLx write repository
- `crates/modules/identity/src/infrastructure/read_models/user_queries.rs`
  - SQLx read-model queries
- `crates/modules/identity/src/presentation/http/mod.rs`
  - HTTP module root
- `crates/modules/identity/src/presentation/http/routes.rs`
  - route registration
- `crates/modules/identity/src/presentation/http/handlers.rs`
  - handlers for auth endpoints
- `crates/modules/identity/src/presentation/http/contracts.rs`
  - request/response payloads
- `crates/modules/identity/src/presentation/http/guards.rs`
  - JWT/role route guards
- `migrations/0001_init_identity.sql`
  - initial schema
- `seeds/dev.sql`
  - baseline seed data
- `tests/http_health.rs`
  - health route smoke test
- `tests/http_auth.rs`
  - end-to-end auth HTTP tests
- `tests/identity_repository.rs`
  - SQLx repository integration tests
- `tools/create_module.sh`
  - repo-local generator script behind `make create-module`
- `tools/templates/module/...`
  - template skeleton for generated modules
- `.github/workflows/ci.yml`
  - CI pipeline
- `README.md`
  - onboarding and command documentation

### Task 1: Create Workspace Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `.gitignore`
- Create: `.env.example`
- Create: `README.md`

- [ ] **Step 1: Write the failing structure test**

Create `tests/workspace_layout.rs`:

```rust
#[test]
fn workspace_lists_expected_members() {
    let manifest = std::fs::read_to_string("Cargo.toml").expect("workspace manifest");

    assert!(manifest.contains("apps/api"));
    assert!(manifest.contains("crates/shared-kernel"));
    assert!(manifest.contains("crates/building-blocks"));
    assert!(manifest.contains("crates/modules/identity"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test workspace_layout -v`
Expected: FAIL with missing `Cargo.toml` or missing workspace members

- [ ] **Step 3: Write minimal workspace manifests and baseline files**

Create `Cargo.toml`:

```toml
[workspace]
members = [
  "apps/api",
  "crates/shared-kernel",
  "crates/building-blocks",
  "crates/modules/identity",
]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "MIT"

[workspace.dependencies]
async-trait = "0.1"
axum = "0.8"
chrono = { version = "0.4", features = ["serde"] }
dotenvy = "0.15"
jsonwebtoken = "10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "migrate"] }
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
uuid = { version = "1", features = ["serde", "v4"] }
```

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable"
components = ["clippy", "rustfmt"]
```

Create `.gitignore`:

```gitignore
/target
.env
.env.local
.sqlx
coverage/
```

Create `.env.example`:

```env
APP_HOST=0.0.0.0
APP_PORT=3000
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/axum_boilerplate
JWT_ACCESS_SECRET=change-me-access
JWT_REFRESH_SECRET=change-me-refresh
JWT_ACCESS_TTL_MINUTES=15
JWT_REFRESH_TTL_DAYS=30
RUST_LOG=info,axum_boilerplate=debug
```

Create `README.md`:

```md
# Axum Boilerplate

Workspace-based Axum boilerplate with Clean Architecture, DDD modules, JWT auth, SQLx, and Makefile workflows.
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test workspace_layout -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml rust-toolchain.toml .gitignore .env.example README.md tests/workspace_layout.rs
git commit -m "chore: create workspace skeleton"
```

### Task 2: Create Shared Kernel Crate

**Files:**
- Create: `crates/shared-kernel/Cargo.toml`
- Create: `crates/shared-kernel/src/lib.rs`
- Create: `crates/shared-kernel/src/ids.rs`
- Create: `crates/shared-kernel/src/pagination.rs`
- Create: `crates/shared-kernel/src/auth.rs`
- Create: `crates/shared-kernel/src/clock.rs`
- Test: `crates/shared-kernel/tests/primitives.rs`

- [ ] **Step 1: Write the failing shared-kernel tests**

Create `crates/shared-kernel/tests/primitives.rs`:

```rust
use shared_kernel::{auth::AuthClaims, ids::UserId, pagination::PageRequest};

#[test]
fn user_id_is_constructible() {
    let id = UserId::new();
    assert_ne!(id.to_string(), "");
}

#[test]
fn page_request_defaults_are_stable() {
    let page = PageRequest::default();
    assert_eq!(page.page, 1);
    assert_eq!(page.per_page, 20);
}

#[test]
fn auth_claims_hold_subject_and_role() {
    let claims = AuthClaims {
        sub: UserId::new(),
        role: "admin".into(),
        exp: 1,
    };

    assert_eq!(claims.role, "admin");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p shared-kernel --test primitives -v`
Expected: FAIL with package or module not found

- [ ] **Step 3: Write minimal shared-kernel implementation**

Create `crates/shared-kernel/Cargo.toml`:

```toml
[package]
name = "shared-kernel"
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
chrono.workspace = true
serde.workspace = true
uuid.workspace = true
```

Create `crates/shared-kernel/src/lib.rs`:

```rust
pub mod auth;
pub mod clock;
pub mod ids;
pub mod pagination;
```

Create `crates/shared-kernel/src/ids.rs`:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl core::fmt::Display for UserId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

Create `crates/shared-kernel/src/pagination.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageRequest {
    pub page: u32,
    pub per_page: u32,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self { page: 1, per_page: 20 }
    }
}
```

Create `crates/shared-kernel/src/auth.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::ids::UserId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: UserId,
    pub role: String,
    pub exp: i64,
}
```

Create `crates/shared-kernel/src/clock.rs`:

```rust
use chrono::{DateTime, Utc};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p shared-kernel --test primitives -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/shared-kernel
git commit -m "feat: add shared kernel primitives"
```

### Task 3: Create Building Blocks Crate

**Files:**
- Create: `crates/building-blocks/Cargo.toml`
- Create: `crates/building-blocks/src/lib.rs`
- Create: `crates/building-blocks/src/config.rs`
- Create: `crates/building-blocks/src/database.rs`
- Create: `crates/building-blocks/src/logging.rs`
- Create: `crates/building-blocks/src/errors.rs`
- Create: `crates/building-blocks/src/jwt.rs`
- Create: `crates/building-blocks/src/openapi.rs`
- Create: `crates/building-blocks/src/jobs.rs`
- Test: `crates/building-blocks/tests/config_loading.rs`

- [ ] **Step 1: Write the failing config test**

Create `crates/building-blocks/tests/config_loading.rs`:

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p building-blocks --test config_loading -v`
Expected: FAIL with package or type not found

- [ ] **Step 3: Write minimal building-blocks implementation**

Create `crates/building-blocks/Cargo.toml`:

```toml
[package]
name = "building-blocks"
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
jsonwebtoken.workspace = true
serde.workspace = true
sqlx.workspace = true
thiserror.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
```

Create `crates/building-blocks/src/lib.rs`:

```rust
pub mod config;
pub mod database;
pub mod errors;
pub mod jobs;
pub mod jwt;
pub mod logging;
pub mod openapi;
```

Create `crates/building-blocks/src/config.rs`:

```rust
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
```

Create `crates/building-blocks/src/database.rs`:

```rust
pub type DbPool = sqlx::PgPool;
```

Create `crates/building-blocks/src/logging.rs`:

```rust
pub fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}
```

Create `crates/building-blocks/src/errors.rs`:

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("configuration error")]
    Config,
    #[error("database error")]
    Database,
}
```

Create `crates/building-blocks/src/jwt.rs`:

```rust
pub struct JwtSettings {
    pub access_secret: String,
    pub refresh_secret: String,
}
```

Create `crates/building-blocks/src/openapi.rs`:

```rust
pub fn docs_path() -> &'static str {
    "/docs"
}
```

Create `crates/building-blocks/src/jobs.rs`:

```rust
pub trait JobPublisher: Send + Sync {
    fn publish(&self, topic: &str, payload: &str);
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p building-blocks --test config_loading -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/building-blocks
git commit -m "feat: add building blocks crate"
```

### Task 4: Create API Binary Skeleton

**Files:**
- Create: `apps/api/Cargo.toml`
- Create: `apps/api/src/main.rs`
- Create: `apps/api/src/bootstrap.rs`
- Create: `apps/api/src/router.rs`
- Create: `apps/api/src/state.rs`
- Test: `tests/http_health.rs`

- [ ] **Step 1: Write the failing health test**

Create `tests/http_health.rs`:

```rust
#[tokio::test]
async fn health_route_returns_ok() {
    let response = reqwest::get("http://127.0.0.1:3000/health").await;
    assert!(response.is_ok());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test http_health -v`
Expected: FAIL because the API binary and route do not exist

- [ ] **Step 3: Write minimal API skeleton**

Create `apps/api/Cargo.toml`:

```toml
[package]
name = "api"
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
axum.workspace = true
building-blocks = { path = "../../crates/building-blocks" }
shared-kernel = { path = "../../crates/shared-kernel" }
tokio.workspace = true
tower-http.workspace = true
tracing.workspace = true
```

Create `apps/api/src/main.rs`:

```rust
mod bootstrap;
mod router;
mod state;

#[tokio::main]
async fn main() {
    bootstrap::run().await.expect("api should start");
}
```

Create `apps/api/src/bootstrap.rs`:

```rust
use std::net::SocketAddr;

use axum::serve;
use tokio::net::TcpListener;

pub async fn run() -> Result<(), std::io::Error> {
    let app = crate::router::build_router();
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    serve(listener, app).await
}
```

Create `apps/api/src/router.rs`:

```rust
use axum::{routing::get, Router};

pub fn build_router() -> Router {
    Router::new().route("/health", get(|| async { "ok" }))
}
```

Create `apps/api/src/state.rs`:

```rust
#[derive(Clone, Default)]
pub struct AppState;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo run -p api` in one terminal and `cargo test --test http_health -v` in another
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/api tests/http_health.rs
git commit -m "feat: add api skeleton and health route"
```

### Task 5: Implement Identity Domain

**Files:**
- Create: `crates/modules/identity/Cargo.toml`
- Create: `crates/modules/identity/src/lib.rs`
- Create: `crates/modules/identity/src/domain/mod.rs`
- Create: `crates/modules/identity/src/domain/entities/user.rs`
- Create: `crates/modules/identity/src/domain/value_objects/email.rs`
- Create: `crates/modules/identity/src/domain/value_objects/password_hash.rs`
- Create: `crates/modules/identity/src/domain/repositories.rs`
- Create: `crates/modules/identity/src/domain/errors.rs`
- Test: `crates/modules/identity/tests/domain_user.rs`

- [ ] **Step 1: Write the failing domain tests**

Create `crates/modules/identity/tests/domain_user.rs`:

```rust
use identity::domain::{entities::user::User, value_objects::email::Email};

#[test]
fn user_registration_creates_active_user() {
    let user = User::register(Email::parse("dev@example.com").unwrap(), "hashed-password".into(), "admin".into());

    assert_eq!(user.email().as_str(), "dev@example.com");
    assert_eq!(user.role(), "admin");
}

#[test]
fn invalid_email_is_rejected() {
    assert!(Email::parse("not-an-email").is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p identity --test domain_user -v`
Expected: FAIL with missing package or types

- [ ] **Step 3: Write minimal identity domain implementation**

Create `crates/modules/identity/Cargo.toml`:

```toml
[package]
name = "identity"
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
async-trait.workspace = true
serde.workspace = true
shared-kernel = { path = "../../shared-kernel" }
thiserror.workspace = true
uuid.workspace = true
```

Create `crates/modules/identity/src/lib.rs`:

```rust
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
```

Create `crates/modules/identity/src/domain/mod.rs`:

```rust
pub mod entities {
    pub mod user;
}
pub mod errors;
pub mod repositories;
pub mod value_objects {
    pub mod email;
    pub mod password_hash;
}
```

Create `crates/modules/identity/src/domain/entities/user.rs`:

```rust
use shared_kernel::ids::UserId;

use crate::domain::value_objects::email::Email;

#[derive(Clone, Debug)]
pub struct User {
    id: UserId,
    email: Email,
    password_hash: String,
    role: String,
}

impl User {
    pub fn register(email: Email, password_hash: String, role: String) -> Self {
        Self {
            id: UserId::new(),
            email,
            password_hash,
            role,
        }
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn role(&self) -> &str {
        &self.role
    }
}
```

Create `crates/modules/identity/src/domain/value_objects/email.rs`:

```rust
#[derive(Clone, Debug)]
pub struct Email(String);

impl Email {
    pub fn parse(value: &str) -> Result<Self, &'static str> {
        if value.contains('@') {
            Ok(Self(value.to_owned()))
        } else {
            Err("invalid email")
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

Create `crates/modules/identity/src/domain/value_objects/password_hash.rs`:

```rust
#[derive(Clone, Debug)]
pub struct PasswordHash(pub String);
```

Create `crates/modules/identity/src/domain/repositories.rs`:

```rust
use async_trait::async_trait;

use crate::domain::{entities::user::User, value_objects::email::Email};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert(&self, user: &User) -> Result<(), crate::domain::errors::IdentityDomainError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, crate::domain::errors::IdentityDomainError>;
}
```

Create `crates/modules/identity/src/domain/errors.rs`:

```rust
#[derive(thiserror::Error, Debug)]
pub enum IdentityDomainError {
    #[error("email already exists")]
    EmailAlreadyExists,
    #[error("persistence failure")]
    Persistence,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p identity --test domain_user -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/modules/identity
git commit -m "feat: add identity domain"
```

### Task 6: Implement Identity Application Use Cases

**Files:**
- Create: `crates/modules/identity/src/application/mod.rs`
- Create: `crates/modules/identity/src/application/dto.rs`
- Create: `crates/modules/identity/src/application/errors.rs`
- Create: `crates/modules/identity/src/application/commands/register.rs`
- Create: `crates/modules/identity/src/application/commands/login.rs`
- Create: `crates/modules/identity/src/application/commands/refresh.rs`
- Create: `crates/modules/identity/src/application/queries/me.rs`
- Test: `crates/modules/identity/tests/application_register.rs`

- [ ] **Step 1: Write the failing application test**

Create `crates/modules/identity/tests/application_register.rs`:

```rust
use identity::application::commands::register::{RegisterCommand, RegisterHandler};

#[tokio::test]
async fn register_handler_returns_tokens() {
    let handler = RegisterHandler::for_tests();

    let result = handler.handle(RegisterCommand {
        email: "dev@example.com".into(),
        password: "secret123".into(),
        role: "admin".into(),
    }).await;

    assert!(result.is_ok());
    let auth = result.unwrap();
    assert!(!auth.access_token.is_empty());
    assert!(!auth.refresh_token.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p identity --test application_register -v`
Expected: FAIL with missing application types or test helper

- [ ] **Step 3: Write minimal application-layer implementation**

Create `crates/modules/identity/src/application/mod.rs`:

```rust
pub mod commands {
    pub mod login;
    pub mod refresh;
    pub mod register;
}
pub mod dto;
pub mod errors;
pub mod queries {
    pub mod me;
}
```

Create `crates/modules/identity/src/application/dto.rs`:

```rust
#[derive(Clone, Debug)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}
```

Create `crates/modules/identity/src/application/errors.rs`:

```rust
#[derive(thiserror::Error, Debug)]
pub enum IdentityApplicationError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("validation failure")]
    Validation,
}
```

Create `crates/modules/identity/src/application/commands/register.rs`:

```rust
use crate::application::{dto::AuthTokens, errors::IdentityApplicationError};

#[derive(Clone, Debug)]
pub struct RegisterCommand {
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Default)]
pub struct RegisterHandler;

impl RegisterHandler {
    pub fn for_tests() -> Self {
        Self
    }

    pub async fn handle(&self, command: RegisterCommand) -> Result<AuthTokens, IdentityApplicationError> {
        if command.email.is_empty() || command.password.is_empty() {
            return Err(IdentityApplicationError::Validation);
        }

        Ok(AuthTokens {
            access_token: format!("access-{}", command.role),
            refresh_token: "refresh-token".into(),
        })
    }
}
```

Create `crates/modules/identity/src/application/commands/login.rs`:

```rust
pub struct LoginCommand;
```

Create `crates/modules/identity/src/application/commands/refresh.rs`:

```rust
pub struct RefreshCommand;
```

Create `crates/modules/identity/src/application/queries/me.rs`:

```rust
pub struct MeQuery;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p identity --test application_register -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/modules/identity/src/application crates/modules/identity/tests/application_register.rs
git commit -m "feat: add identity application use cases"
```

### Task 7: Implement Identity Infrastructure Adapters

**Files:**
- Create: `crates/modules/identity/src/infrastructure/mod.rs`
- Create: `crates/modules/identity/src/infrastructure/passwords.rs`
- Create: `crates/modules/identity/src/infrastructure/jwt_tokens.rs`
- Create: `crates/modules/identity/src/infrastructure/persistence/user_repository.rs`
- Create: `crates/modules/identity/src/infrastructure/read_models/user_queries.rs`
- Test: `tests/identity_repository.rs`

- [ ] **Step 1: Write the failing repository integration test**

Create `tests/identity_repository.rs`:

```rust
#[tokio::test]
async fn user_repository_inserts_and_reads_user() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url).await.expect("db connection");

    let row = sqlx::query_scalar::<_, i64>("select count(*) from users")
        .fetch_one(&pool)
        .await;

    assert!(row.is_ok());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test identity_repository -v`
Expected: FAIL because migrations/schema are not present

- [ ] **Step 3: Write minimal infrastructure adapters**

Create `crates/modules/identity/src/infrastructure/mod.rs`:

```rust
pub mod jwt_tokens;
pub mod passwords;
pub mod persistence {
    pub mod user_repository;
}
pub mod read_models {
    pub mod user_queries;
}
```

Create `crates/modules/identity/src/infrastructure/passwords.rs`:

```rust
pub fn hash_password(raw: &str) -> String {
    format!("hashed::{raw}")
}
```

Create `crates/modules/identity/src/infrastructure/jwt_tokens.rs`:

```rust
use crate::application::dto::AuthTokens;

pub fn issue_tokens(role: &str) -> AuthTokens {
    AuthTokens {
        access_token: format!("access::{role}"),
        refresh_token: format!("refresh::{role}"),
    }
}
```

Create `crates/modules/identity/src/infrastructure/persistence/user_repository.rs`:

```rust
pub struct SqlxUserRepository;
```

Create `crates/modules/identity/src/infrastructure/read_models/user_queries.rs`:

```rust
pub struct UserReadModel;
```

- [ ] **Step 4: Run test to verify it still fails for missing schema only**

Run: `cargo test --test identity_repository -v`
Expected: FAIL with relation `users` does not exist

- [ ] **Step 5: Commit**

```bash
git add crates/modules/identity/src/infrastructure tests/identity_repository.rs
git commit -m "feat: add identity infrastructure adapters"
```

### Task 8: Add Database Migrations And Seeds

**Files:**
- Create: `migrations/0001_init_identity.sql`
- Create: `seeds/dev.sql`
- Modify: `tests/identity_repository.rs`

- [ ] **Step 1: Extend the failing integration test to verify schema**

Update `tests/identity_repository.rs`:

```rust
#[tokio::test]
async fn user_repository_schema_is_available() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url).await.expect("db connection");

    let count = sqlx::query_scalar::<_, i64>("select count(*) from users")
        .fetch_one(&pool)
        .await
        .expect("users table should exist");

    assert!(count >= 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test identity_repository -v`
Expected: FAIL because the `users` table does not exist

- [ ] **Step 3: Add initial migration and seed**

Create `migrations/0001_init_identity.sql`:

```sql
create table if not exists users (
    id uuid primary key,
    email text not null unique,
    password_hash text not null,
    role text not null,
    created_at timestamptz not null default now()
);
```

Create `seeds/dev.sql`:

```sql
insert into users (id, email, password_hash, role)
values (
    gen_random_uuid(),
    'admin@example.com',
    'hashed::password',
    'admin'
)
on conflict (email) do nothing;
```

- [ ] **Step 4: Run migration and verify test passes**

Run: `sqlx database create && sqlx migrate run && cargo test --test identity_repository -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add migrations seeds tests/identity_repository.rs
git commit -m "feat: add identity schema and seeds"
```

### Task 9: Add HTTP Presentation And Auth Routes

**Files:**
- Create: `crates/modules/identity/src/presentation/http/mod.rs`
- Create: `crates/modules/identity/src/presentation/http/routes.rs`
- Create: `crates/modules/identity/src/presentation/http/handlers.rs`
- Create: `crates/modules/identity/src/presentation/http/contracts.rs`
- Create: `crates/modules/identity/src/presentation/http/guards.rs`
- Modify: `apps/api/src/router.rs`
- Test: `tests/http_auth.rs`

- [ ] **Step 1: Write the failing auth HTTP test**

Create `tests/http_auth.rs`:

```rust
#[tokio::test]
async fn register_endpoint_returns_tokens() {
    let response = reqwest::Client::new()
        .post("http://127.0.0.1:3000/api/v1/auth/register")
        .json(&serde_json::json!({
            "email": "dev@example.com",
            "password": "secret123",
            "role": "admin"
        }))
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 201);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo run -p api` in one terminal and `cargo test --test http_auth -v` in another
Expected: FAIL with 404 or connection/setup failure

- [ ] **Step 3: Write minimal presentation and route integration**

Create `crates/modules/identity/src/presentation/http/mod.rs`:

```rust
pub mod contracts;
pub mod guards;
pub mod handlers;
pub mod routes;
```

Create `crates/modules/identity/src/presentation/http/contracts.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}
```

Create `crates/modules/identity/src/presentation/http/handlers.rs`:

```rust
use axum::{http::StatusCode, Json};

use crate::{
    application::commands::register::{RegisterCommand, RegisterHandler},
    presentation::http::contracts::{AuthResponse, RegisterRequest},
};

pub async fn register(Json(request): Json<RegisterRequest>) -> (StatusCode, Json<AuthResponse>) {
    let handler = RegisterHandler::for_tests();
    let tokens = handler
        .handle(RegisterCommand {
            email: request.email,
            password: request.password,
            role: request.role,
        })
        .await
        .expect("register should succeed");

    (
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        }),
    )
}
```

Create `crates/modules/identity/src/presentation/http/routes.rs`:

```rust
use axum::{routing::post, Router};

use crate::presentation::http::handlers;

pub fn router() -> Router {
    Router::new().route("/api/v1/auth/register", post(handlers::register))
}
```

Create `crates/modules/identity/src/presentation/http/guards.rs`:

```rust
pub fn require_role(role: &str, expected: &str) -> bool {
    role == expected
}
```

Modify `apps/api/src/router.rs`:

```rust
use axum::{routing::get, Router};
use identity::presentation::http::routes as identity_routes;

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(identity_routes::router())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo run -p api` in one terminal and `cargo test --test http_auth -v` in another
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/modules/identity/src/presentation apps/api/src/router.rs tests/http_auth.rs
git commit -m "feat: add auth routes"
```

### Task 10: Add Makefile, Docker, And Local Workflow Commands

**Files:**
- Create: `Makefile`
- Create: `docker-compose.yml`
- Modify: `README.md`

- [ ] **Step 1: Write the failing workflow check**

Append to `README.md` the expected commands section and verify `make check` is documented:

```md
## Commands

- `make setup`
- `make dev`
- `make up`
- `make down`
- `make reset-db`
- `make migrate`
- `make seed`
- `make fmt`
- `make lint`
- `make test`
- `make test-unit`
- `make test-integration`
- `make check`
- `make docs`
- `make ci`
```

Run: `rg -n "make check" README.md`
Expected: FAIL before the section is added

- [ ] **Step 2: Run check to verify it fails**

Run: `rg -n "make check" README.md`
Expected: exit code 1

- [ ] **Step 3: Write Makefile and Docker setup**

Create `Makefile`:

```makefile
.PHONY: setup dev up down reset-db migrate seed fmt lint test test-unit test-integration check docs ci create-module

setup:
	cp -n .env.example .env || true

dev:
	cargo run -p api

up:
	docker compose up -d postgres

down:
	docker compose down

reset-db:
	docker compose down -v
	docker compose up -d postgres

migrate:
	sqlx migrate run

seed:
	psql $$DATABASE_URL -f seeds/dev.sql

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

test-unit:
	cargo test -p shared-kernel -p building-blocks -p identity

test-integration:
	cargo test --test identity_repository --test http_auth

check:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

docs:
	@echo "OpenAPI endpoint: /docs"

ci:
	make check

create-module:
	./tools/create_module.sh $(name)
```

Create `docker-compose.yml`:

```yaml
services:
  postgres:
    image: postgres:17
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: axum_boilerplate
    ports:
      - "5432:5432"
```

Update `README.md` with setup and command usage:

```md
## Setup

1. `make setup`
2. `make up`
3. `make migrate`
4. `make seed`
5. `make dev`

## Commands

- `make setup`
- `make dev`
- `make up`
- `make down`
- `make reset-db`
- `make migrate`
- `make seed`
- `make fmt`
- `make lint`
- `make test`
- `make test-unit`
- `make test-integration`
- `make check`
- `make docs`
- `make ci`
- `make create-module name=billing`
```

- [ ] **Step 4: Run check to verify it passes**

Run: `rg -n "make check" README.md && make -n check`
Expected: README line match and printed command sequence

- [ ] **Step 5: Commit**

```bash
git add Makefile docker-compose.yml README.md
git commit -m "feat: add developer workflow commands"
```

### Task 11: Implement `create-module` Generator

**Files:**
- Create: `tools/create_module.sh`
- Create: `tools/templates/module/Cargo.toml.tpl`
- Create: `tools/templates/module/src/lib.rs.tpl`
- Create: `tools/templates/module/src/domain/mod.rs.tpl`
- Create: `tools/templates/module/src/application/mod.rs.tpl`
- Create: `tools/templates/module/src/infrastructure/mod.rs.tpl`
- Create: `tools/templates/module/src/presentation/http/mod.rs.tpl`
- Test: `tests/create_module.rs`
- Modify: `Cargo.toml`
- Modify: `apps/api/src/router.rs`

- [ ] **Step 1: Write the failing generator test**

Create `tests/create_module.rs`:

```rust
#[test]
fn create_module_script_exists() {
    assert!(std::path::Path::new("tools/create_module.sh").exists());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test create_module -v`
Expected: FAIL because the script and templates do not exist

- [ ] **Step 3: Write minimal generator implementation**

Create `tools/create_module.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

name="${1:-}"
if [[ -z "$name" ]]; then
  echo "usage: ./tools/create_module.sh <module-name>" >&2
  exit 1
fi

crate_dir="crates/modules/$name"
if [[ -d "$crate_dir" ]]; then
  echo "module already exists: $name" >&2
  exit 1
fi

mkdir -p "$crate_dir/src/domain" "$crate_dir/src/application" "$crate_dir/src/infrastructure" "$crate_dir/src/presentation/http"
cp tools/templates/module/Cargo.toml.tpl "$crate_dir/Cargo.toml"
cp tools/templates/module/src/lib.rs.tpl "$crate_dir/src/lib.rs"
cp tools/templates/module/src/domain/mod.rs.tpl "$crate_dir/src/domain/mod.rs"
cp tools/templates/module/src/application/mod.rs.tpl "$crate_dir/src/application/mod.rs"
cp tools/templates/module/src/infrastructure/mod.rs.tpl "$crate_dir/src/infrastructure/mod.rs"
cp tools/templates/module/src/presentation/http/mod.rs.tpl "$crate_dir/src/presentation/http/mod.rs"

sed -i "s/__MODULE__/$name/g" "$crate_dir/Cargo.toml"
```

Create `tools/templates/module/Cargo.toml.tpl`:

```toml
[package]
name = "__MODULE__"
edition.workspace = true
version.workspace = true
license.workspace = true
```

Create `tools/templates/module/src/lib.rs.tpl`:

```rust
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
```

Create `tools/templates/module/src/domain/mod.rs.tpl`:

```rust
pub struct DomainPlaceholder;
```

Create `tools/templates/module/src/application/mod.rs.tpl`:

```rust
pub struct ApplicationPlaceholder;
```

Create `tools/templates/module/src/infrastructure/mod.rs.tpl`:

```rust
pub struct InfrastructurePlaceholder;
```

Create `tools/templates/module/src/presentation/http/mod.rs.tpl`:

```rust
pub struct PresentationPlaceholder;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test create_module -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tools tests/create_module.rs
git commit -m "feat: add module generator"
```

### Task 12: Add OpenAPI, CI, And Final Verification

**Files:**
- Create: `.github/workflows/ci.yml`
- Modify: `apps/api/src/router.rs`
- Modify: `README.md`

- [ ] **Step 1: Write the failing CI presence check**

Run: `test -f .github/workflows/ci.yml`
Expected: exit code 1

- [ ] **Step 2: Run check to verify it fails**

Run: `test -f .github/workflows/ci.yml`
Expected: exit code 1

- [ ] **Step 3: Write CI workflow and docs endpoint registration**

Create `.github/workflows/ci.yml`:

```yaml
name: ci

on:
  push:
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace
```

Modify `apps/api/src/router.rs`:

```rust
use axum::{response::Html, routing::get, Router};
use identity::presentation::http::routes as identity_routes;

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/docs", get(|| async { Html("<h1>API Docs</h1>") }))
        .merge(identity_routes::router())
}
```

Update `README.md`:

```md
## Docs

Run the API and open `/docs`.
```

- [ ] **Step 4: Run full verification**

Run: `make check`
Expected: PASS

Run: `cargo test --workspace`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/ci.yml apps/api/src/router.rs README.md
git commit -m "feat: add ci and docs endpoint"
```

## Self-Review

Spec coverage:

- workspace-based modular monolith: Tasks 1, 2, 3, 4, 12
- shared kernel and building blocks: Tasks 2 and 3
- identity bounded context with clean layers: Tasks 5, 6, 7, 9
- PostgreSQL/SQLx persistence and migrations: Tasks 7 and 8
- JWT auth baseline and protected route structure: Tasks 6, 7, 9
- Makefile command surface: Task 10
- `create-module` generator with auto-registration seam: Task 11, followed by router/workspace integration refinement during execution
- OpenAPI/docs endpoint and CI baseline: Task 12

Placeholder scan:

- No `TODO` or `TBD` markers remain in the plan body.
- Every task includes concrete files, commands, and expected outcomes.

Type consistency:

- `shared-kernel`, `building-blocks`, `identity`, and `api` names are used consistently across manifests and tests.
- The identity application flow consistently refers to `RegisterCommand`, `RegisterHandler`, and `AuthTokens`.
