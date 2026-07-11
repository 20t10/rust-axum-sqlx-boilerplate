# Task 1 report: strict dotenv loader and typed configuration

## Changes

- Added `dotenvy = "0.15"` to workspace dependencies and enabled it in `building-blocks`; `Cargo.lock` records the direct crate dependency.
- Replaced the configuration integration test with the six required temporary-file strict-loader cases.
- Added all required `ConfigError` variants and value-safe messages.
- Added the twelve-key required dotenv schema, deterministic parsing, sorted duplicate detection, sorted process-environment collision detection, missing/blank/numeric/positive validation, and IP/socket validation.
- Added pure `load_required_dotenv_from`, production `load_required_dotenv`, and `AppConfig::socket_addr` APIs.
- Preserved `AppConfig::from_env` while routing its fields through the same blank, numeric, positive, and host validation helpers.
- `dotenvy::Error::LineParse` is destructured immediately; its source line is discarded and only the index is retained. No `dotenvy::Error` is stored in `ConfigError`, so `Display`, `Debug`, and `source()` cannot reveal a dotenv source line.

## RED evidence

Command:

`cargo test -p building-blocks --test config_loading`

Outcome: exit 101. Compilation failed for the intended missing feature: unresolved import `load_required_dotenv_from` plus absent `DotenvFile`, `DuplicateDotenvKeys`, `EnvironmentCollision`, and `DotenvSyntax` variants.

## GREEN and verification evidence

Focused command:

`cargo test -p building-blocks --test config_loading`

Outcome: exit 0; 6 passed, 0 failed.

Final command:

`cargo fmt --all -- --check && cargo test -p building-blocks && cargo clippy -p building-blocks --all-targets -- -D warnings && git diff --check`

Outcome: exit 0. Building-block tests: 6 integration tests passed, 0 failed; 0 unit and 0 doc tests failed. Clippy completed with no warnings. Formatting and diff whitespace checks passed.

## Self-review

- Scope review: only the five requested Task 1 paths are staged for the implementation commit.
- Ordering review: duplicate keys are rejected before collision/validation; collisions are rejected before any environment installation; values are installed once and only after complete validation.
- Determinism review: `BTreeMap`/`BTreeSet` provide sorted duplicate and collision key vectors.
- Compatibility review: `from_env` validates the seven `AppConfig` fields without forcing unrelated dotenv-only fields, while the strict loader separately validates all twelve required keys.
- Secret-safety review: error variants retain names, paths, parse indices, and value-independent standard parse errors only. The secret-bearing dotenv line is discarded on `LineParse`; duplicate/collision errors contain keys only.
- No concerns found.

## Commit

`48f873d5872da4a004b228225720d475f43d43b5` — `feat: add strict dotenv configuration loader`

## Review fix: reject dotenv interpolation before parsing

### Changes

- Added the typed, value-safe `ConfigError::DotenvInterpolation { path, index }` variant. It stores no source line or value and has no error source.
- Changed strict dotenv parsing to read the file into private memory, scan for dotenv interpolation syntax, and only then pass the content to `dotenvy::from_read_iter`.
- The scan rejects both `$NAME` and `${NAME}` forms before `dotenvy` can consult `std::env`. This deliberately makes interpolation unsupported by the strict loader and preserves the purity of `load_required_dotenv_from`.
- Added regression coverage for both interpolation forms, including `Display`, `Debug`, and `source()` assertions proving the ordinary source value is not retained or exposed.
- Added coverage documenting that a dollar sign which does not begin dotenv interpolation remains accepted.

### RED evidence

Command:

`cargo test -p building-blocks --test config_loading`

Outcome: exit 101. Compilation failed at the new regression assertion with `E0599`: `ConfigError::DotenvInterpolation` did not exist. This is the expected missing-behavior failure before production changes.

### GREEN and verification evidence

Focused command after implementation:

`cargo test -p building-blocks --test config_loading`

Outcome: exit 0; 8 passed, 0 failed.

Final required commands:

`cargo test -p building-blocks --test config_loading`

`cargo clippy -p building-blocks --all-targets -- -D warnings`

Outcome: both commands exited 0. The focused suite passed 8 tests with 0 failures; Clippy completed with no warnings. `cargo fmt --all -- --check` and `git diff --check` also exited 0.

### Review-fix self-review

- Purity: all potentially environment-resolving `$NAME` and `${NAME}` syntax is detected in the file content before constructing the dotenv iterator, so `load_required_dotenv_from` cannot resolve those values from ambient process state.
- Error safety: the interpolation error contains only the path and byte index. Tests check both formatted and debug representations and confirm there is no source error.
- Ordinary-value safety: neither the interpolation source line nor its complete ordinary value is copied into an error variant; existing syntax handling continues to discard source lines.
- Compatibility: ordinary files and non-interpolation dollar signs continue through the existing dotenv parser and validation pipeline.
