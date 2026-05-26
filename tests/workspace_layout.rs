#[test]
fn workspace_lists_expected_members() {
    let manifest = std::fs::read_to_string("Cargo.toml").expect("workspace manifest");

    assert!(manifest.contains("apps/api"));
    assert!(manifest.contains("crates/shared-kernel"));
    assert!(manifest.contains("crates/building-blocks"));
    assert!(manifest.contains("crates/modules/identity"));

    for path in [
        "src/lib.rs",
        "apps/api/Cargo.toml",
        "apps/api/src/lib.rs",
        "crates/shared-kernel/Cargo.toml",
        "crates/shared-kernel/src/lib.rs",
        "crates/building-blocks/Cargo.toml",
        "crates/building-blocks/src/lib.rs",
        "crates/modules/identity/Cargo.toml",
        "crates/modules/identity/src/lib.rs",
    ] {
        assert!(std::path::Path::new(path).exists(), "missing {path}");
    }
}
