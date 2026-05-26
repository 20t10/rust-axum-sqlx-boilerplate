# `make init-project` Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Makefile-backed bootstrap command that copies this Axum boilerplate into a new sibling project, rewrites legacy identity tokens and Rust crate names, and verifies the generated repo is self-contained.

**Architecture:** `make init-project` is the public entrypoint. A repo-local Bash script performs the copy, rewrites tracked text files in place, regenerates the lockfile, and runs lightweight verification in the generated project. A Rust integration test exercises the command end-to-end from a temporary source copy so the template repo itself is never mutated during verification.

**Tech Stack:** GNU Make, Bash, Rust integration tests, `cargo`, standard library filesystem APIs.

---

### Task 1: Add an end-to-end bootstrap smoke test

**Files:**
- Create: `tests/init_project.rs`

- [ ] **Step 1: Write the failing test**

Add a test that copies the current repo into a temporary sandbox, runs `make init-project name=demo-app`, and asserts that the generated repo exists and no longer contains the template slug in the core identity files.

```rust
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn unique_dir(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{stamp}-{}", std::process::id()))
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if name == ".git" || name == "target" {
            continue;
        }
        let target = dst.join(name);
        if path.is_dir() {
            copy_dir(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("file should be readable")
}

#[test]
fn init_project_rewrites_template_identity() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sandbox = unique_dir("axum-init-sandbox");
    copy_dir(&source_root, &sandbox).expect("sandbox copy");

    let output = Command::new("make")
        .args(["init-project", "name=demo-app"])
        .current_dir(&sandbox)
        .output()
        .expect("make should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let generated = sandbox.parent().expect("sandbox parent").join("demo-app");
    let cargo = read(generated.join("Cargo.toml"));
    let readme = read(generated.join("README.md"));
    let env = read(generated.join(".env.example"));

    assert!(cargo.contains("name = \"demo-app\""));
    assert!(readme.contains("# Demo App"));
    assert!(env.contains("POSTGRES_DB=demo-app"));
    assert!(env.contains("RUST_LOG=info,demo_app=debug"));
    assert!(!cargo.contains("axum-boilerplate"));
    assert!(!readme.contains("axum-boilerplate"));
    assert!(!env.contains("axum-boilerplate"));
}

#[test]
fn init_project_refuses_to_overwrite_existing_directory() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sandbox = unique_dir("axum-init-overwrite");
    copy_dir(&source_root, &sandbox).expect("sandbox copy");

    let target = sandbox.parent().expect("sandbox parent").join("demo-app");
    fs::create_dir_all(&target).expect("target dir");

    let output = Command::new("make")
        .args(["init-project", "name=demo-app"])
        .current_dir(&sandbox)
        .output()
        .expect("make should run");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("already exists"));
}
```

- [ ] **Step 2: Run the test to confirm it fails before implementation**

Run: `cargo test --test init_project -- --nocapture`
Expected: fail because `make init-project` is not implemented yet.

- [ ] **Step 3: Keep the scope limited to the bootstrap behavior**

Do not add extra dependencies unless the test cannot be expressed with the standard library. The smoke test only needs to validate copy, rewrite, overwrite protection, and the absence of the legacy slug in the generated repo.

---

### Task 2: Implement the bootstrap script and Makefile entrypoint

**Files:**
- Create: `tools/init_project.sh`
- Modify: `Makefile`

- [ ] **Step 1: Write the failing command surface**

Add the Makefile target so the test can invoke a real command path before the script exists.

```make
.PHONY: setup dev up down reset-db migrate seed fmt lint test test-unit test-integration check docs ci create-module init-project

init-project:
	./tools/init_project.sh $(name)
```

- [ ] **Step 2: Implement the script to copy, rewrite, and verify**

Create a repo-local Bash script that:
- requires `name=...`
- copies the current repository to the sibling directory `../<name>`
- refuses to overwrite an existing target directory
- removes copied `.git` metadata and build output
- rewrites `axum-boilerplate`, `axum_boilerplate`, and `Axum Boilerplate`
- rewrites `DATABASE_URL`, `POSTGRES_DB`, and `RUST_LOG` defaults in the generated repo
- regenerates `Cargo.lock`
- runs `cargo fmt --all --check`
- runs `cargo test --workspace --no-run`
- fails if any legacy slug remains in tracked text files

```bash
#!/usr/bin/env bash
set -euo pipefail

project_name="${1:-}"
if [[ -z "$project_name" ]]; then
  echo "usage: ./tools/init_project.sh <project-name>" >&2
  exit 1
fi

source_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
parent_dir="$(cd "$source_root/.." && pwd)"
target_dir="$parent_dir/$project_name"
project_snake="${project_name//-/_}"
project_title="$(printf '%s' "$project_name" | tr '-_' '  ' | awk '{
  for (i = 1; i <= NF; i++) {
    $i = toupper(substr($i, 1, 1)) substr($i, 2)
  }
  print
}')"

if [[ -e "$target_dir" ]]; then
  echo "target already exists: $target_dir" >&2
  exit 1
fi

cp -a "$source_root" "$target_dir"
rm -rf "$target_dir/.git" "$target_dir/target"

rewrite_file() {
  local file="$1"
  perl -0pi -e "s/axum-boilerplate/$project_name/g; s/axum_boilerplate/$project_snake/g; s/Axum Boilerplate/$project_title/g" "$file"
  perl -0pi -e "s/POSTGRES_DB=axum_boilerplate/POSTGRES_DB=$project_name/g; s#DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/axum_boilerplate#DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/$project_name#g; s#RUST_LOG=info,axum_boilerplate=debug#RUST_LOG=info,$project_snake=debug#g" "$file"
}

while IFS= read -r -d '' file; do
  case "$file" in
    */target/*|*/.git/*) continue ;;
  esac
  rewrite_file "$file"
done < <(find "$target_dir" -type f -print0)

cd "$target_dir"
cargo generate-lockfile
cargo fmt --all --check
cargo test --workspace --no-run

if rg -n "axum-boilerplate|axum_boilerplate" .; then
  echo "legacy template slug still present" >&2
  exit 1
fi
```

- [ ] **Step 3: Run the command to verify it fails before the script exists**

Run: `make init-project name=demo-app`
Expected: fail with a missing-script or missing-target error until the script is added.

- [ ] **Step 4: Keep the Makefile thin**

The Makefile should only dispatch to the script; it should not duplicate rename logic or filesystem traversal.

---

### Task 3: Document the bootstrap workflow and generated-project expectations

**Files:**
- Modify: `README.md`
- Modify: `.env.example`

- [ ] **Step 1: Add the failing documentation presence check**

Update the README with a short bootstrap section that explains:
- `make init-project name=demo-app` creates a sibling project
- the generated project is independent of the template repo
- the command rewrites the old slug `axum-boilerplate`
- `make create-module` remains the module-level generator inside a generated project

Use this exact wording as the baseline:

```md
## Bootstrap

Create a new project from this template:

```bash
make init-project name=demo-app
```

This copies the template into `../demo-app`, rewrites the legacy `axum-boilerplate` identity, and leaves a standalone project ready for local setup.
```

- [ ] **Step 2: Update the example env file to match the rewritten bootstrap output**

Keep `.env.example` aligned with the bootstrap script so the generated project starts with env-driven defaults instead of template-specific literals.

```dotenv
APP_HOST=0.0.0.0
APP_PORT=3000
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_DB=axum_boilerplate
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/axum_boilerplate
JWT_ACCESS_SECRET=change-me-access
JWT_REFRESH_SECRET=change-me-refresh
JWT_ACCESS_TTL_MINUTES=15
JWT_REFRESH_TTL_DAYS=30
RUST_LOG=info,axum_boilerplate=debug
```

- [ ] **Step 3: Run the docs check before the README is updated**

Run: `rg -n "make init-project|axum-boilerplate" README.md .env.example`
Expected: fail until the bootstrap docs and example env content are added.

- [ ] **Step 4: Keep the user-facing docs concise**

The goal is to explain the one-command bootstrap, not to duplicate the full implementation details from the script.

---

### Task 4: Run the generated-project verification and clean up the bootstrap path

**Files:**
- Modify: `tests/init_project.rs`
- Modify: `tools/init_project.sh`

- [ ] **Step 1: Add the final verification behavior to the script**

After the copy and rewrite step, ensure the generated repo is self-consistent by checking:
- `cargo generate-lockfile`
- `cargo fmt --all --check`
- `cargo test --workspace --no-run`
- `rg -n "axum-boilerplate|axum_boilerplate" .`

If any check fails, the script should exit nonzero and print the generated repository path so the caller can inspect the partial output.

- [ ] **Step 2: Run the bootstrap smoke test**

Run: `cargo test --test init_project -- --nocapture`
Expected: PASS, with the generated project at `../demo-app` proving the old slug was removed from the copied repo.

- [ ] **Step 3: Manually inspect the generated repo once**

Run:

```bash
cd ../demo-app
rg -n "axum-boilerplate|axum_boilerplate" .
cargo test --workspace --no-run
```

Expected:
- the search returns no matches
- the workspace compiles without executing DB-backed tests

- [ ] **Step 4: Commit the bootstrap feature**

```bash
git add Makefile README.md .env.example tools/init_project.sh tests/init_project.rs
git commit -m "feat: add project bootstrap generator"
```

## Self-Review

Coverage check:

- copy current template repo into a sibling project: Task 2
- rewrite legacy `axum-boilerplate` tokens: Tasks 1, 2, 3, 4
- rename Rust crate/package identifiers to the new project name: Task 2
- refuse to overwrite existing targets: Task 1 and Task 2
- verify generated repo is self-contained: Task 4
- keep `make` as the public UX: Task 2
- document the bootstrap workflow for users: Task 3

No placeholders remain, the scope is limited to one bootstrap command, and the verification steps are explicit enough to run without extra context.
