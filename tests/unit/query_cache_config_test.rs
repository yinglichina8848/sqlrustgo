// Query Cache Config Tests
use sqlrustgo_executor::query_cache_config::QueryCacheConfig;

#[test]
fn test_query_cache_config_default() {
    let config = QueryCacheConfig::default();
    assert!(config.enabled);
    assert_eq!(config.max_entries, 1000);
    assert_eq!(config.ttl_seconds, 30);
    assert_eq!(config.max_memory_bytes, 100 * 1024 * 1024);
}

#[test]
fn test_query_cache_config_builder() {
    let config = QueryCacheConfig {
        enabled: false,
        max_entries: 500,
        ttl_seconds: 600,
        max_memory_bytes: 50 * 1024 * 1024,
        benchmark_mode: false,
    };

    assert!(!config.enabled);
    assert_eq!(config.max_entries, 500);
    assert_eq!(config.ttl_seconds, 600);
}
