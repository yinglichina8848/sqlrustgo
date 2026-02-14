#[test]
fn test_project_structure() {
    sqlrustgo::init();
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
