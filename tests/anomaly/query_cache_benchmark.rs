//! Query Cache QPS Benchmark Tests
//!
//! Validates Query Cache performance and correctness

#[cfg(test)]
mod tests {
    use sqlrustgo_executor::query_cache::QueryCache;
    use sqlrustgo_executor::query_cache_config::{CacheEntry, CacheKey, QueryCacheConfig};
    use sqlrustgo_types::Value;
    use std::time::Instant;

    const BENCHMARK_QUERIES: usize = 100000;
    const CACHE_SIZE_LIMIT: usize = 1000;

    fn create_test_entry(id: i64) -> CacheEntry {
        CacheEntry {
            result: sqlrustgo_executor::ExecutorResult::new(
                vec![vec![
                    Value::Integer(id),
                    Value::Text(format!("data_{}", id)),
                    Value::Integer(id * 2),
                ]],
                1,
            ),
            tables: vec!["test_table".to_string()],
            created_at: std::time::Instant::now(),
            size_bytes: 128,
        }
    }

    fn make_cache_key(sql: &str, id: i64) -> CacheKey {
        CacheKey {
            normalized_sql: sql.to_string(),
            params_hash: id as u64,
        }
    }

    #[test]
    fn test_query_cache_throughput() {
        let config = QueryCacheConfig {
            max_entries: CACHE_SIZE_LIMIT,
            max_memory_bytes: 100 * 1024 * 1024,
            ttl_seconds: 60,
            enabled: true,
            benchmark_mode: false,
        };
        let mut cache = QueryCache::new(config);

        for i in 0..CACHE_SIZE_LIMIT {
            let key = make_cache_key("SELECT * FROM test_table WHERE id = ?", i as i64);
            let entry = create_test_entry(i as i64);
            cache.put(key, entry, vec!["test_table".to_string()]);
        }

        let start = Instant::now();

        for i in 0..BENCHMARK_QUERIES {
            let key = make_cache_key(
                "SELECT * FROM test_table WHERE id = ?",
                (i % CACHE_SIZE_LIMIT) as i64,
            );
            let _ = cache.get(&key);
        }

        let elapsed = start.elapsed();
        let qps = BENCHMARK_QUERIES as f64 / elapsed.as_secs_f64();

        println!();
        println!("========================================");
        println!("Query Cache Throughput Benchmark");
        println!("========================================");
        println!("Total queries:    {}", BENCHMARK_QUERIES);
        println!("Cache size:       {}", CACHE_SIZE_LIMIT);
        println!("Time elapsed:      {:?}", elapsed);
        println!("QPS:              {:.2} ops/sec", qps);
        println!("========================================");

        let stats = cache.stats();
        println!("Cache entries:     {}", stats.entries);
        println!("Cache memory:     {} bytes", stats.memory_bytes);
        println!("========================================");

        assert!(qps > 0.0, "QPS should be positive");
    }

    #[test]
    fn test_query_cache_miss_overhead() {
        let config = QueryCacheConfig {
            max_entries: CACHE_SIZE_LIMIT,
            max_memory_bytes: 100 * 1024 * 1024,
            ttl_seconds: 60,
            enabled: true,
            benchmark_mode: false,
        };
        let mut cache = QueryCache::new(config);

        let start = Instant::now();

        for i in 0..BENCHMARK_QUERIES {
            let key = make_cache_key(
                "SELECT * FROM test_table WHERE id = ?",
                (i + CACHE_SIZE_LIMIT + 1) as i64,
            );
            let _ = cache.get(&key);
        }

        let elapsed = start.elapsed();
        let qps = BENCHMARK_QUERIES as f64 / elapsed.as_secs_f64();

        println!();
        println!("========================================");
        println!("Query Cache Miss Overhead Benchmark");
        println!("========================================");
        println!("Total queries:    {}", BENCHMARK_QUERIES);
        println!("Cache misses:     100%");
        println!("Time elapsed:      {:?}", elapsed);
        println!("QPS:              {:.2} ops/sec", qps);
        println!("========================================");

        assert!(qps > 0.0, "QPS should be positive");
    }

    #[test]
    fn test_query_cache_hit_rate_simulation() {
        let config = QueryCacheConfig {
            max_entries: 100,
            max_memory_bytes: 10 * 1024 * 1024,
            ttl_seconds: 60,
            enabled: true,
            benchmark_mode: false,
        };
        let mut cache = QueryCache::new(config);

        let repeated_queries = vec![1i64, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let total_queries = 10000;
        let mut hits = 0;
        let mut misses = 0;

        for &id in &repeated_queries {
            let key = make_cache_key("SELECT * FROM users WHERE id = ?", id);
            let entry = create_test_entry(id);
            cache.put(key, entry, vec!["users".to_string()]);
        }

        for i in 0..total_queries {
            if i % 5 == 0 {
                let key = make_cache_key("SELECT * FROM users WHERE id = ?", (1000 + i) as i64);
                if cache.get(&key).is_none() {
                    misses += 1;
                    let entry = create_test_entry((1000 + i) as i64);
                    cache.put(key, entry, vec!["users".to_string()]);
                } else {
                    hits += 1;
                }
            } else {
                let id = repeated_queries[(i as usize) % repeated_queries.len()];
                let key = make_cache_key("SELECT * FROM users WHERE id = ?", id);
                if cache.get(&key).is_some() {
                    hits += 1;
                } else {
                    misses += 1;
                }
            }
        }

        let hit_rate = hits as f64 / (hits + misses) as f64 * 100.0;

        println!();
        println!("========================================");
        println!("Query Cache Hit Rate Simulation");
        println!("========================================");
        println!("Total queries:    {}", total_queries);
        println!("Cache hits:       {}", hits);
        println!("Cache misses:     {}", misses);
        println!("Hit rate:         {:.2}%", hit_rate);
        println!("========================================");

        assert!(
            hit_rate > 70.0,
            "Hit rate should be > 70% for 80% repeated queries"
        );
    }

    #[test]
    fn test_query_cache_lru_behavior() {
        let config = QueryCacheConfig {
            max_entries: 50,
            max_memory_bytes: 10 * 1024 * 1024,
            ttl_seconds: 60,
            enabled: true,
            benchmark_mode: false,
        };
        let mut cache = QueryCache::new(config);

        for i in 0..25 {
            let key = make_cache_key("SELECT ?", i);
            let entry = create_test_entry(i);
            cache.put(key, entry, vec![]);
        }

        for i in 0..10 {
            let key = make_cache_key("SELECT ?", i);
            let _ = cache.get(&key);
        }

        for i in 25..50 {
            let key = make_cache_key("SELECT ?", i);
            let entry = create_test_entry(i);
            cache.put(key, entry, vec![]);
        }

        let stats = cache.stats();
        println!();
        println!("========================================");
        println!("Query Cache LRU Behavior");
        println!("========================================");
        println!("Max entries:       {}", 50);
        println!("Final entries:     {}", stats.entries);
        println!("========================================");

        assert_eq!(stats.entries, 50, "Should have exactly 50 entries");
    }

    #[test]
    fn test_query_cache_memory_usage() {
        let config = QueryCacheConfig {
            max_entries: 100,
            max_memory_bytes: 1024 * 1024,
            ttl_seconds: 60,
            enabled: true,
            benchmark_mode: false,
        };
        let mut cache = QueryCache::new(config);

        let initial_stats = cache.stats();
        println!();
        println!("========================================");
        println!("Query Cache Memory Usage");
        println!("========================================");
        println!("Initial entries:  {}", initial_stats.entries);
        println!("Initial memory:  {} bytes", initial_stats.memory_bytes);

        for i in 0..100 {
            let key = make_cache_key("SELECT * FROM t WHERE id = ?", i);
            let entry = create_test_entry(i);
            cache.put(key, entry, vec!["t".to_string()]);
        }

        let after_stats = cache.stats();
        println!("After 100 entries: {}", after_stats.entries);
        println!("After 100 memory:   {} bytes", after_stats.memory_bytes);
        println!("========================================");

        assert!(after_stats.entries <= 100, "Should not exceed max_entries");
    }
}
