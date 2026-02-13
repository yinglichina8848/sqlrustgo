#[test]
fn test_project_structure() {
    let result = sqlrustgo::greet();
    assert_eq!(result, "SQLRustGo Database System");
}

#[test]
fn test_cargo_toml_exists() {
    assert!(std::path::Path::new("Cargo.toml").exists());
}

#[test]
fn test_src_main_exists() {
    assert!(std::path::Path::new("src/main.rs").exists());
}

#[test]
fn test_src_lib_exists() {
    assert!(std::path::Path::new("src/lib.rs").exists());
}
