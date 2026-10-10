# Axum Boilerplate

Initial workspace skeleton for an Axum boilerplate project in progress.

## Workspace

- `apps/api`: API entry point and router composition.
- `crates/shared-kernel`: shared IDs, clock, auth claims, and pagination.
- `crates/building-blocks`: configuration, database pool type, and logging.
- `crates/modules/identity`: the example module.
- `tools/templates/module`: templates for modules added when needed.

Billing is not included by default. Create it with `make create-module name=billing` when needed.
Authentication and persistence implementations are still scaffolding.

## Setup
Install SQLx CLI if it is not already available:

```sh
cargo install sqlx-cli --version 0.8.6 --no-default-features --features rustls,postgres --locked
```

0. `make init-project name=my-project`
1. `make setup`
2. `make up`
3. `make migrate`
4. `make seed`
5. `make dev`

## Configuration

Run `make setup` to create the mandatory `.env` from `.env.example`. The API refuses to start when `.env` is missing, malformed, incomplete, or contains a key that is already exported in the process environment. Keep runtime values in `.env`; do not export the same keys in your shell. Errors list conflicting key names but never their values.

All values in `.env.example` are examples. Replace database credentials and JWT secrets before using the project outside local development.

Dotenv interpolation is not supported. To include a literal dollar sign in a value, use single quotes (`KEY='costs-$5'`) or escaping (`KEY=costs-\$5`).

Because `make migrate` and `make seed` source `.env` through the shell before invoking their tools, keep `.env` entries shell-compatible (`KEY=VALUE`, with shell quoting or escaping where needed).

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
- `make build-check`
- `make check`
- `make docs`
- `make ci`
- `make create-module name=billing`
- `make target-size`
- `make clean-doc`
- `make clean`

## Lean builds

All crates share the workspace's `target/` directory. Development and test profiles
use `debug = 0` and `incremental = false`: builds omit debug information and
incremental caches, at the cost of less debugger information and potentially slower
recompilation. Profiles do not remove artifacts from earlier builds.

SQLx disables default features and explicitly enables only `postgres`,
`runtime-tokio`, `tls-rustls`, `uuid`, and `chrono`. Current queries use runtime
functions such as `query_scalar()`, and migrations run through the separate SQLx CLI.
Add SQLx features when code requires them:

| Code or data | Feature |
| --- | --- |
| `#[derive(sqlx::FromRow)]`, `#[derive(sqlx::Type)]` | `derive` |
| `query!`, `query_as!`, `#[sqlx::test]` | `macros` |
| Rust migration support or automatic test database management | `migrate` |
| `migrate!` | `macros` and `migrate` |
| PostgreSQL JSON / JSONB values | `json` |
| Decimal values | `rust_decimal` |

The SQLx CLI's features are configured separately from the application's dependency.
All consuming crates inherit SQLx from the workspace rather than enabling defaults
in their own manifests. User IDs are generated as UUID v7; the development seed
uses PostgreSQL 18's `uuidv7()`.

Use `cargo check -p api` while editing to check compilation without producing an
executable. `make build-check` checks all workspace targets. Keep S3, PDF, email,
and other integrations out of the base dependency set until their code is added.

### Manual rebuild

To start without old artifacts, run the following from the workspace root:

```sh
make target-size
make clean
make build-check
make target-size
make setup
make up
make migrate
make seed
make dev
```

`make clean` removes generated artifacts and requires rebuilding afterwards.
Compare disk sizes using the same commands and build features each time: checking,
running, testing, and generating coverage produce different artifacts. `make clean-doc`
removes generated documentation only. There is no automatic cleanup during builds.

Build checks, formatting, linting, unit tests, and cleanup commands do not require
`.env`. Database operations and the API still use the configuration described above.
The Makefile and project/module scripts require Bash and Unix utilities; on Windows,
use Git Bash/WSL with Make installed, or invoke Cargo directly. The Bash-specific
project initialization tests run only on Unix.

For a manual run using Cargo directly after setting up the database:

```sh
cargo check --workspace --all-targets
cargo test -p shared-kernel -p building-blocks -p identity -p api
cargo run -p api
```


## Docs

Run the API and open `/docs`.
