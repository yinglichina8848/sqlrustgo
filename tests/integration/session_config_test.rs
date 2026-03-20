use sqlrustgo_executor::session_config::SessionConfig;

#[test]
fn test_default_not_benchmark_mode() {
    std::env::remove_var("SQLRUSTGO_BENCHMARK_MODE");
    let config = SessionConfig::default();
    assert!(!config.benchmark_mode);
    assert!(config.cache_enabled);
    assert!(config.stats_enabled);
}

#[test]
fn test_benchmark_mode_from_env() {
    std::env::set_var("SQLRUSTGO_BENCHMARK_MODE", "1");
    let config = SessionConfig::default();
    assert!(config.benchmark_mode);
    assert!(!config.cache_enabled);
    assert!(!config.stats_enabled);
    std::env::remove_var("SQLRUSTGO_BENCHMARK_MODE");
}

#[test]
fn test_explicit_benchmark_mode() {
    let config = SessionConfig::new(true);
    assert!(config.benchmark_mode);
    assert!(!config.cache_enabled);
    assert!(!config.stats_enabled);
}

#[test]
fn test_explicit_normal_mode() {
    let config = SessionConfig::new(false);
    assert!(!config.benchmark_mode);
    assert!(config.cache_enabled);
    assert!(config.stats_enabled);
}
