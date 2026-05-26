#[test]
fn workspace_lists_expected_members() {
    let manifest = std::fs::read_to_string("Cargo.toml").expect("workspace manifest");

    assert!(manifest.contains("apps/api"));
    assert!(manifest.contains("crates/shared-kernel"));
    assert!(manifest.contains("crates/building-blocks"));
    assert!(manifest.contains("crates/modules/identity"));
}
