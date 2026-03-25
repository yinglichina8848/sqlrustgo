// Query Cache Tests
use sqlrustgo_executor::query_cache::QueryCache;
use sqlrustgo_executor::query_cache_config::{CacheEntry, CacheKey, QueryCacheConfig};
use sqlrustgo_executor::{ExecutorResult, QueryCacheStats};
use std::time::Instant;

#[test]
fn test_query_cache_new() {
    let config = QueryCacheConfig::default();
    let cache = QueryCache::new(config);
    let stats = cache.stats();
    assert_eq!(stats.entries, 0);
}

#[test]
fn test_query_cache_get_empty() {
    let config = QueryCacheConfig::default();
    let mut cache = QueryCache::new(config);

    let key = CacheKey {
        normalized_sql: "SELECT * FROM test".to_string(),
        params_hash: 0,
    };
    let result = cache.get(&key);
    assert!(result.is_none());
}

#[test]
fn test_query_cache_put_and_get() {
    let config = QueryCacheConfig::default();
    let mut cache = QueryCache::new(config);

    let key = CacheKey {
        normalized_sql: "SELECT * FROM test".to_string(),
        params_hash: 0,
    };
    let entry = CacheEntry {
        result: ExecutorResult::new(vec![], 0),
        tables: vec!["test".to_string()],
        created_at: Instant::now(),
        size_bytes: 0,
    };

    cache.put(key.clone(), entry, vec!["test".to_string()]);

    let result = cache.get(&key);
    assert!(result.is_some());
}

#[test]
fn test_query_cache_invalidate_table() {
    let config = QueryCacheConfig::default();
    let mut cache = QueryCache::new(config);

    let key = CacheKey {
        normalized_sql: "SELECT * FROM test".to_string(),
        params_hash: 0,
    };
    let entry = CacheEntry {
        result: ExecutorResult::new(vec![], 0),
        tables: vec!["test".to_string()],
        created_at: Instant::now(),
        size_bytes: 0,
    };

    cache.put(key.clone(), entry, vec!["test".to_string()]);

    cache.invalidate_table("test");

    let result = cache.get(&key);
    assert!(result.is_none());
}

#[test]
fn test_query_cache_clear() {
    let config = QueryCacheConfig::default();
    let mut cache = QueryCache::new(config);

    let key = CacheKey {
        normalized_sql: "SELECT * FROM test".to_string(),
        params_hash: 0,
    };
    let entry = CacheEntry {
        result: ExecutorResult::new(vec![], 0),
        tables: vec!["test".to_string()],
        created_at: Instant::now(),
        size_bytes: 0,
    };

    cache.put(key, entry, vec!["test".to_string()]);
    cache.clear();

    assert_eq!(cache.stats().entries, 0);
}

#[test]
fn test_cache_key_new() {
    let key = CacheKey {
        normalized_sql: "SELECT * FROM test".to_string(),
        params_hash: 0,
    };
    assert_eq!(key.normalized_sql, "SELECT * FROM test".to_string());
}

#[test]
fn test_query_cache_stats() {
    let config = QueryCacheConfig::default();
    let cache = QueryCache::new(config);

    let stats = cache.stats();
    assert_eq!(stats.entries, 0);
}

#[test]
fn test_cache_entry_is_expired() {
    let entry = CacheEntry {
        result: ExecutorResult::new(vec![], 0),
        tables: vec![],
        created_at: Instant::now(),
        size_bytes: 0,
    };

    // Entry created just now shouldn't be expired
    use std::time::Duration;
    assert!(!entry.is_expired(Duration::from_secs(60)));
}

#[test]
fn test_cache_entry_estimate_size() {
    use sqlrustgo_types::Value;

    let entry = CacheEntry {
        result: ExecutorResult::new(
            vec![vec![Value::Integer(1), Value::Text("hello".to_string())]],
            1,
        ),
        tables: vec![],
        created_at: Instant::now(),
        size_bytes: 0,
    };

    let size = entry.estimate_size();
    assert!(size > 0);
}
