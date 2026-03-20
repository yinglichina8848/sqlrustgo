use sqlrustgo_executor::{CacheEntry, CacheKey, QueryCache, QueryCacheConfig};
use sqlrustgo_types::Value;
use std::time::Instant;

#[test]
fn test_cache_basic_get_put() {
    let config = QueryCacheConfig {
        max_entries: 10,
        max_memory_bytes: 1024 * 1024,
        ttl_seconds: 60,
        enabled: true,
        benchmark_mode: false,
    };
    let mut cache = QueryCache::new(config);

    let key = CacheKey {
        normalized_sql: "select * from t".to_string(),
        params_hash: 123,
    };
    let entry = CacheEntry {
        result: sqlrustgo_executor::ExecutorResult::new(vec![vec![Value::Integer(1)]], 1),
        tables: vec!["t".to_string()],
        created_at: Instant::now(),
        size_bytes: 100,
    };

    cache.put(key.clone(), entry.clone(), vec!["t".to_string()]);

    let result = cache.get(&key);
    assert!(result.is_some());
    assert_eq!(result.unwrap().rows.len(), 1);
}

#[test]
fn test_cache_invalidate_table() {
    let mut cache = QueryCache::new(QueryCacheConfig::default());

    let key1 = CacheKey {
        normalized_sql: "select * from t1".to_string(),
        params_hash: 1,
    };
    cache.put(key1, make_dummy_entry(), vec!["t1".to_string()]);

    let key2 = CacheKey {
        normalized_sql: "select * from t2".to_string(),
        params_hash: 2,
    };
    cache.put(key2, make_dummy_entry(), vec!["t2".to_string()]);

    cache.invalidate_table("t1");

    assert!(cache
        .get(&CacheKey {
            normalized_sql: "select * from t1".to_string(),
            params_hash: 1
        })
        .is_none());
    assert!(cache
        .get(&CacheKey {
            normalized_sql: "select * from t2".to_string(),
            params_hash: 2
        })
        .is_some());
}

#[test]
fn test_cache_lru_eviction() {
    let mut config = QueryCacheConfig {
        max_entries: 2,
        max_memory_bytes: 1024 * 1024,
        ttl_seconds: 60,
        enabled: true,
        benchmark_mode: false,
    };
    let mut cache = QueryCache::new(config);

    cache.put(
        CacheKey {
            normalized_sql: "q1".to_string(),
            params_hash: 1,
        },
        make_dummy_entry(),
        vec!["t".to_string()],
    );
    cache.put(
        CacheKey {
            normalized_sql: "q2".to_string(),
            params_hash: 2,
        },
        make_dummy_entry(),
        vec!["t".to_string()],
    );
    cache.put(
        CacheKey {
            normalized_sql: "q3".to_string(),
            params_hash: 3,
        },
        make_dummy_entry(),
        vec!["t".to_string()],
    );

    assert!(cache
        .get(&CacheKey {
            normalized_sql: "q1".to_string(),
            params_hash: 1
        })
        .is_none());
    assert!(cache
        .get(&CacheKey {
            normalized_sql: "q2".to_string(),
            params_hash: 2
        })
        .is_some());
    assert!(cache
        .get(&CacheKey {
            normalized_sql: "q3".to_string(),
            params_hash: 3
        })
        .is_some());
}

fn make_dummy_entry() -> CacheEntry {
    CacheEntry {
        result: sqlrustgo_executor::ExecutorResult::new(vec![vec![Value::Integer(1)]], 1),
        tables: vec![],
        created_at: Instant::now(),
        size_bytes: 16,
    }
}
