use std::{
    fs, io,
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

    assert!(read(&cargo_toml).contains("name = \"demo-app\""));
    assert!(read(&readme).contains("# Demo App"));
    assert!(read(&env_example).contains("POSTGRES_DB=demo_app"));
    assert!(read(&env_example)
        .contains("DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/demo_app"));
    assert!(read(&env_example).contains("RUST_LOG=info,demo_app=debug"));

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
