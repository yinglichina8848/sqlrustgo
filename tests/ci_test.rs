#[test]
fn test_github_workflows_exists() {
    assert!(std::path::Path::new(".github/workflows/ci.yml").exists());
}

#[test]
fn test_claude_config_exists() {
    assert!(std::path::Path::new(".claude/claude_desktop_config.json").exists());
}

#[test]
fn test_cargo_toolchain_exists() {
    assert!(std::path::Path::new("cargo-toolchain.toml").exists());
}

#[test]
fn test_claude_config_valid_json() {
    let content = std::fs::read_to_string(".claude/claude_desktop_config.json").unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");
}

#[test]
fn test_ci_workflows_has_build_job() {
    let content = std::fs::read_to_string(".github/workflows/ci.yml").unwrap();
    assert!(content.contains("cargo build"), "CI should have build step");
    assert!(content.contains("cargo test"), "CI should have test step");
    assert!(content.contains("cargo clippy"), "CI should have clippy step");
}
