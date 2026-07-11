use std::{
    fs, io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn create(prefix: &str) -> io::Result<Self> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be monotonic")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{stamp}-{}", std::process::id()));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
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

fn run_init_project(cwd: &Path) -> std::process::Output {
    Command::new("make")
        .args(["init-project", "name=demo-app"])
        .current_dir(cwd)
        .output()
        .expect("make should run")
}

fn make_fixture(prefix: &str) -> (TempWorkspace, PathBuf) {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = TempWorkspace::create(prefix).expect("workspace");
    let fixture = workspace.path().join("fixture");
    fs::create_dir_all(&fixture).expect("fixture dir");
    fs::copy(source_root.join("Makefile"), fixture.join("Makefile")).expect("copy Makefile");
    fs::copy(
        source_root.join(".env.example"),
        fixture.join(".env.example"),
    )
    .expect("copy .env.example");
    (workspace, fixture)
}

fn assert_no_legacy_identity_tokens(path: &Path) {
    let contents = read(path);
    assert!(
        !contents.contains("axum-boilerplate"),
        "{} still contains axum-boilerplate",
        path.display()
    );
    assert!(
        !contents.contains("axum_boilerplate"),
        "{} still contains axum_boilerplate",
        path.display()
    );
    assert!(
        !contents.contains("Axum Boilerplate"),
        "{} still contains Axum Boilerplate",
        path.display()
    );
}

#[test]
fn init_project_generates_a_clean_sibling_repo() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = TempWorkspace::create("axum-init-project").expect("workspace");
    let sandbox = workspace.path().join("source");
    copy_dir(&source_root, &sandbox).expect("sandbox copy");

    let output = run_init_project(&sandbox);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let generated = workspace.path().join("demo-app");
    assert!(
        generated.is_dir(),
        "generated repo should exist at {}",
        generated.display()
    );

    let cargo_toml = generated.join("Cargo.toml");
    let readme = generated.join("README.md");
    let env_example = generated.join(".env.example");
    let compose = generated.join("docker-compose.yml");

    assert!(read(&cargo_toml).contains("name = \"demo-app\""));
    assert!(read(&readme).contains("# Demo App"));
    let readme_contents = read(&readme);
    assert!(readme_contents.contains("interpolation is not supported"));
    assert!(readme_contents.contains("single quotes"));
    assert!(readme_contents.contains("escaping"));
    assert!(read(&env_example).contains("POSTGRES_DB=demo_app"));
    assert!(read(&env_example)
        .contains("DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/demo_app"));
    assert!(read(&env_example).contains("RUST_LOG=info,demo_app=debug"));

    let env_example_contents = read(&env_example);
    for key in [
        "APP_HOST",
        "APP_PORT",
        "POSTGRES_USER",
        "POSTGRES_PASSWORD",
        "POSTGRES_DB",
        "POSTGRES_PORT",
        "DATABASE_URL",
        "JWT_ACCESS_SECRET",
        "JWT_REFRESH_SECRET",
        "JWT_ACCESS_TTL_MINUTES",
        "JWT_REFRESH_TTL_DAYS",
        "RUST_LOG",
    ] {
        assert!(
            env_example_contents
                .lines()
                .any(|line| line.starts_with(&format!("{key}="))),
            ".env.example should contain {key}"
        );
    }

    let compose_contents = read(&compose);
    assert!(compose_contents.contains("${POSTGRES_USER:?set POSTGRES_USER in .env}"));
    assert!(compose_contents.contains("${POSTGRES_PASSWORD:?set POSTGRES_PASSWORD in .env}"));
    assert!(compose_contents.contains("${POSTGRES_DB:?set POSTGRES_DB in .env}"));
    assert!(compose_contents.contains("${POSTGRES_PORT:?set POSTGRES_PORT in .env}"));
    assert!(!compose_contents.contains(":-"));

    assert_no_legacy_identity_tokens(&cargo_toml);
    assert_no_legacy_identity_tokens(&readme);
    assert_no_legacy_identity_tokens(&env_example);
}

#[test]
fn init_project_refuses_to_overwrite_an_existing_target() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = TempWorkspace::create("axum-init-project-overwrite").expect("workspace");
    let sandbox = workspace.path().join("source");
    copy_dir(&source_root, &sandbox).expect("sandbox copy");

    let target = workspace.path().join("demo-app");
    fs::create_dir_all(&target).expect("target dir");

    let output = run_init_project(&sandbox);

    assert!(
        !output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("already exists"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn bare_make_runs_default_setup_without_an_existing_env() {
    let (_workspace, fixture) = make_fixture("axum-make-default");

    let output = Command::new("make")
        .current_dir(&fixture)
        .output()
        .expect("make should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        read(fixture.join(".env")),
        read(fixture.join(".env.example"))
    );
}

#[test]
fn mixed_exempt_and_non_exempt_make_goals_require_env() {
    let (_workspace, fixture) = make_fixture("axum-make-mixed-goals");

    let output = Command::new("make")
        .args(["-n", "init-project", "dev", "name=demo-app"])
        .current_dir(&fixture)
        .output()
        .expect("make should run");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing required .env; run 'make setup'"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.join(".env").exists());
}

#[test]
fn non_exempt_make_goal_requires_env() {
    let (_workspace, fixture) = make_fixture("axum-make-required-env");

    let output = Command::new("make")
        .args(["-n", "dev"])
        .current_dir(&fixture)
        .output()
        .expect("make should run");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing required .env; run 'make setup'"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn migrate_sources_database_url_from_dotenv_for_sqlx() {
    let (_workspace, fixture) = make_fixture("axum-make-migrate-env");
    let bin = fixture.join("bin");
    fs::create_dir(&bin).expect("bin dir");
    let sqlx = bin.join("sqlx");
    fs::write(
        &sqlx,
        "#!/bin/sh\nprintf '%s' \"$DATABASE_URL\" > \"$CAPTURE_FILE\"\n",
    )
    .expect("fake sqlx");
    fs::set_permissions(&sqlx, fs::Permissions::from_mode(0o755)).expect("executable fake sqlx");

    let database_url = "postgres://fixture:secret@localhost:5432/fixture";
    fs::write(
        fixture.join(".env"),
        format!("DATABASE_URL={database_url}\n"),
    )
    .expect("fixture dotenv");
    let capture = fixture.join("captured-database-url");
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let output = Command::new("make")
        .arg("migrate")
        .current_dir(&fixture)
        .env("PATH", path)
        .env("CAPTURE_FILE", &capture)
        .env_remove("DATABASE_URL")
        .output()
        .expect("make should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(read(capture), database_url);
}
