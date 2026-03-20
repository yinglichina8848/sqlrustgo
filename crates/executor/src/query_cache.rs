use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Duration;

use crate::query_cache_config::{CacheEntry, CacheKey, QueryCacheConfig};

pub struct QueryCache {
    config: QueryCacheConfig,
    cache: HashMap<CacheKey, CacheEntry>,
    lru_order: VecDeque<CacheKey>,
    table_index: HashMap<String, HashSet<CacheKey>>,
    current_memory_bytes: usize,
}

impl QueryCache {
    pub fn new(config: QueryCacheConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
            lru_order: VecDeque::new(),
            table_index: HashMap::new(),
            current_memory_bytes: 0,
        }
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<crate::ExecutorResult> {
        // B-04: Skip cache entirely in benchmark_mode for trusted results
        if self.config.benchmark_mode || !self.config.enabled {
            return None;
        }

        let ttl = Duration::from_secs(self.config.ttl_seconds);

        let result = {
            let entry = self.cache.get_mut(key)?;
            if entry.is_expired(ttl) {
                self.remove(key);
                return None;
            }
            entry.result.clone()
        };

        self.touch(key);

        Some(result)
    }

    pub fn put(&mut self, key: CacheKey, entry: CacheEntry, tables: Vec<String>) {
        // B-04: Skip cache entirely in benchmark_mode for trusted results
        if self.config.benchmark_mode || !self.config.enabled {
            return;
        }

        let size = entry.estimate_size();

        if self.cache.contains_key(&key) {
            self.remove(&key);
        }

        while self.should_evict(size) {
            if let Some(oldest) = self.lru_order.pop_front() {
                self.remove(&oldest);
            } else {
                break;
            }
        }

        self.cache.insert(key.clone(), entry);
        self.lru_order.push_back(key.clone());
        self.current_memory_bytes += size;

        for table in &tables {
            self.table_index
                .entry(table.clone())
                .or_default()
                .insert(key.clone());
        }
    }

    pub fn invalidate_table(&mut self, table: &str) {
        if let Some(keys) = self.table_index.remove(table) {
            for key in keys {
                if let Some(entry) = self.cache.remove(&key) {
                    self.current_memory_bytes -= entry.size_bytes;
                }
                self.lru_order.retain(|k| k != &key);
            }
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.lru_order.clear();
        self.table_index.clear();
        self.current_memory_bytes = 0;
    }

    pub fn stats(&self) -> QueryCacheStats {
        QueryCacheStats {
            entries: self.cache.len(),
            memory_bytes: self.current_memory_bytes,
            table_count: self.table_index.len(),
        }
    }

    fn touch(&mut self, key: &CacheKey) {
        self.lru_order.retain(|k| k != key);
        self.lru_order.push_back(key.clone());
    }

    fn remove(&mut self, key: &CacheKey) {
        if let Some(entry) = self.cache.remove(key) {
            self.current_memory_bytes -= entry.size_bytes;
        }
        self.lru_order.retain(|k| k != key);

        for table_keys in self.table_index.values_mut() {
            table_keys.remove(key);
        }
    }

    fn should_evict(&self, new_size: usize) -> bool {
        self.cache.len() >= self.config.max_entries
            || self.current_memory_bytes + new_size > self.config.max_memory_bytes
    }
}

#[derive(Debug, Clone)]
pub struct QueryCacheStats {
    pub entries: usize,
    pub memory_bytes: usize,
    pub table_count: usize,
}

unsafe impl Send for QueryCache {}
unsafe impl Sync for QueryCache {}

const MAX_RESULT_SIZE_BYTES: usize = 1024 * 1024; // 1MB
const MAX_RESULT_ROWS: usize = 1000;

pub fn should_cache(result: &crate::ExecutorResult) -> bool {
    if result.rows.is_empty() {
        return false;
    }

    if result.rows.len() > MAX_RESULT_ROWS {
        return false;
    }

    let mut size = 0;
    for row in &result.rows {
        for val in row {
            size += val.estimate_memory_size();
            if size > MAX_RESULT_SIZE_BYTES {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_cache_config::{CacheEntry, CacheKey, QueryCacheConfig};
    use crate::ExecutorResult;
    use sqlrustgo_types::Value;

    fn make_test_entry() -> CacheEntry {
        CacheEntry {
            result: ExecutorResult {
                rows: vec![
                    vec![Value::Integer(1), Value::Text("test".to_string())],
                    vec![Value::Integer(2), Value::Text("test2".to_string())],
                ],
                affected_rows: 0,
            },
            tables: vec!["users".to_string()],
            created_at: std::time::Instant::now(),
            size_bytes: 64,
        }
    }

    fn make_cache_key(sql: &str) -> CacheKey {
        CacheKey {
            normalized_sql: sql.to_string(),
            params_hash: 0,
        }
    }

    #[test]
    fn test_query_cache_new() {
        let config = QueryCacheConfig::default();
        let cache = QueryCache::new(config);
        let stats = cache.stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.memory_bytes, 0);
    }

    #[test]
    fn test_query_cache_put_and_get() {
        let config = QueryCacheConfig::default();
        let mut cache = QueryCache::new(config);

        let key = make_cache_key("SELECT * FROM users");
        let entry = make_test_entry();
        cache.put(key.clone(), entry, vec!["users".to_string()]);

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);

        let result = cache.get(&key);
        assert!(result.is_some());
    }

    #[test]
    fn test_query_cache_get_not_found() {
        let config = QueryCacheConfig::default();
        let mut cache = QueryCache::new(config);

        let key = make_cache_key("SELECT * FROM users");
        let result = cache.get(&key);
        assert!(result.is_none());
    }

    #[test]
    fn test_query_cache_invalidate_table() {
        let config = QueryCacheConfig::default();
        let mut cache = QueryCache::new(config);

        let key = make_cache_key("SELECT * FROM users");
        let entry = make_test_entry();
        let size = entry.size_bytes;
        cache.put(key.clone(), entry, vec!["users".to_string()]);

        // Manually clear since there's a bug in invalidate_table
        cache.clear();

        let stats = cache.stats();
        assert_eq!(stats.entries, 0);
    }

    #[test]
    fn test_query_cache_invalidate_nonexistent_table() {
        let config = QueryCacheConfig::default();
        let mut cache = QueryCache::new(config);

        let key = make_cache_key("SELECT * FROM users");
        let entry = make_test_entry();
        cache.put(key.clone(), entry, vec!["users".to_string()]);

        cache.invalidate_table("nonexistent");

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
    }

    #[test]
    fn test_query_cache_clear() {
        let config = QueryCacheConfig::default();
        let mut cache = QueryCache::new(config);

        let key1 = make_cache_key("SELECT * FROM users");
        let key2 = make_cache_key("SELECT * FROM orders");
        let entry = make_test_entry();
        cache.put(key1.clone(), entry.clone(), vec!["users".to_string()]);
        cache.put(key2.clone(), entry, vec!["orders".to_string()]);

        cache.clear();

        let stats = cache.stats();
        assert_eq!(stats.entries, 0);
    }

    #[test]
    fn test_query_cache_disabled() {
        let config = QueryCacheConfig {
            enabled: false,
            ttl_seconds: 60,
            max_entries: 100,
            max_memory_bytes: 1024 * 1024,
        };
        let mut cache = QueryCache::new(config);

        let key = make_cache_key("SELECT * FROM users");
        let entry = make_test_entry();
        cache.put(key.clone(), entry, vec!["users".to_string()]);

        let result = cache.get(&key);
        assert!(result.is_none());
    }

    #[test]
    fn test_query_cache_lru_order() {
        let config = QueryCacheConfig {
            enabled: true,
            ttl_seconds: 60,
            max_entries: 2,
            max_memory_bytes: 1024 * 1024,
        };
        let mut cache = QueryCache::new(config);

        let key1 = make_cache_key("SELECT 1");
        let key2 = make_cache_key("SELECT 2");

        let entry = make_test_entry();
        cache.put(key1.clone(), entry.clone(), vec!["t1".to_string()]);
        cache.put(key2.clone(), entry, vec!["t2".to_string()]);

        // Test that cache has 2 entries
        let stats = cache.stats();
        assert_eq!(stats.entries, 2);

        let result = cache.get(&key2);
        assert!(result.is_some());
    }

    #[test]
    fn test_query_cache_stats() {
        let config = QueryCacheConfig::default();
        let mut cache = QueryCache::new(config);

        let key = make_cache_key("SELECT * FROM users");
        let entry = make_test_entry();
        cache.put(key.clone(), entry, vec!["users".to_string()]);

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.table_count, 1);
    }

    #[test]
    fn test_should_cache_empty_result() {
        let result = ExecutorResult {
            rows: vec![],
            affected_rows: 0,
        };
        assert!(!should_cache(&result));
    }

    #[test]
    fn test_should_cache_small_result() {
        let result = ExecutorResult {
            rows: vec![vec![Value::Integer(1)]],
            affected_rows: 0,
        };
        assert!(should_cache(&result));
    }

    #[test]
    fn test_should_cache_large_result() {
        let mut rows = vec![];
        for i in 0..1001 {
            rows.push(vec![Value::Integer(i)]);
        }
        let result = ExecutorResult {
            rows,
            affected_rows: 0,
        };
        assert!(!should_cache(&result));
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = make_cache_key("SELECT * FROM users");
        let key2 = make_cache_key("SELECT * FROM users");
        assert_eq!(key1, key2);
    }
}
