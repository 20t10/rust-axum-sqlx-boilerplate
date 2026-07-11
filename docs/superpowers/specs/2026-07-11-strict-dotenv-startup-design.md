# Strict `.env` Startup Configuration Design

## Goal

Make application startup deterministic and fail fast. The API must have a real `.env` file, every required application setting must be present and valid, and no key declared by `.env` may already exist in the process environment.

## Required configuration

The application requires these keys:

- `APP_HOST`
- `APP_PORT`
- `DATABASE_URL`
- `JWT_ACCESS_SECRET`
- `JWT_REFRESH_SECRET`
- `JWT_ACCESS_TTL_MINUTES`
- `JWT_REFRESH_TTL_DAYS`
- `RUST_LOG`

Required string values must not be empty or whitespace-only. `APP_PORT` must parse as a `u16`. Both JWT TTL values must parse as positive `i64` values. `APP_HOST` and `APP_PORT` together must produce an address that Tokio can bind.

The existing PostgreSQL variables used by Docker Compose (`POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`, and `POSTGRES_PORT`) must also be declared in `.env`; Compose must not silently supply fallback values.

## Loading flow

The configuration loader belongs in `building-blocks`, while the API bootstrap remains responsible for starting the server:

1. Resolve the required `.env` path from the process working directory.
2. Return an error if `.env` does not exist, cannot be read, or has invalid dotenv syntax.
3. Parse the keys without mutating the process environment and reject a key declared more than once inside `.env`.
4. Compare all keys declared in `.env` with the process environment captured before loading.
5. If any keys collide, return one error containing the sorted, deduplicated key names. Never include their values.
6. Load the parsed values into the process environment.
7. Construct and validate `AppConfig` from the environment, and verify `RUST_LOG` is non-blank before logging initialization.
8. Use `APP_HOST` and `APP_PORT` in API bootstrap instead of the hardcoded `127.0.0.1:3000`.

This deliberately rejects the normal dotenv precedence model. A key has exactly one source: the mandatory `.env` file.

## Errors

Configuration errors will be typed and retain their causes. They distinguish:

- missing `.env` file;
- unreadable `.env` file;
- invalid dotenv syntax;
- a key declared more than once inside `.env`;
- keys duplicated between `.env` and the process environment;
- missing or blank required values;
- invalid numeric values;
- non-positive TTL values.

API startup will expose a single startup error type that can wrap configuration and listener I/O failures. `main` may report that error and terminate non-zero, but bootstrap itself must return errors rather than panic. Error messages may contain paths, variable names, and parse failures, but must never contain configuration values.

## Docker and developer workflow

`.env.example` remains the committed template and documents every required key. `make setup` continues to copy it to `.env`. Docker Compose interpolation defaults such as `${POSTGRES_PORT:-5432}` will be replaced by required `${POSTGRES_PORT}` references so missing Compose settings fail instead of being silently defaulted.

The API's strict duplicate check means values intended for the API must not be exported in the shell when running `make dev`; they must come from `.env`. Tools such as SQLx and `psql` may still consume `DATABASE_URL` after the Makefile explicitly loads `.env` for those commands.

## Testing

Tests will be written before production changes and will cover:

- missing `.env`;
- malformed dotenv input;
- duplicate keys within `.env`;
- duplicate keys, including deterministic key ordering and no leaked values;
- missing required values;
- blank required string values;
- invalid port and TTL values;
- non-positive TTL values;
- successful loading of a complete `.env`;
- creation of the listener address from `APP_HOST` and `APP_PORT` rather than hardcoded values.

Environment-mutating tests must be serialized or isolated in subprocesses/temp directories so they cannot race with Rust's parallel test runner. Existing configuration and workspace tests must continue to pass.

## Out of scope

- Environment-specific files such as `.env.production` or `.env.local`.
- Supplying configuration solely through container or CI environment variables.
- Secret managers or runtime secret rotation.
- Defaults for required application settings.
