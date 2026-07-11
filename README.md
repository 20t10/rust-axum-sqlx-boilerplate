# Axum Boilerplate

Initial workspace skeleton for an Axum boilerplate project in progress.

## Setup
0. `make init-project name=my-project`
1. `make setup`
2. `make up`
3. `make migrate`
4. `make seed`
5. `make dev`

## Configuration

Run `make setup` to create the mandatory `.env` from `.env.example`. The API refuses to start when `.env` is missing, malformed, incomplete, or contains a key that is already exported in the process environment. Keep runtime values in `.env`; do not export the same keys in your shell. Errors list conflicting key names but never their values.

All values in `.env.example` are examples. Replace database credentials and JWT secrets before using the project outside local development.

Because `make seed` sources `.env` through the shell before invoking `psql`, keep `.env` entries shell-compatible (`KEY=VALUE`, with shell quoting or escaping where needed).

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


## Docs

Run the API and open `/docs`.
