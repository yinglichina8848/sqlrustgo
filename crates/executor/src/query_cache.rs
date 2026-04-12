use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::query_cache_config::{CacheEntry, CacheKey, QueryCacheConfig};

pub struct QueryCache {
    config: QueryCacheConfig,
    cache: HashMap<CacheKey, CacheEntry>,
    table_index: HashMap<String, HashSet<CacheKey>>,
    current_memory_bytes: usize,
    access_counter: u64,
}

impl QueryCache {
    pub fn new(config: QueryCacheConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
            table_index: HashMap::new(),
            current_memory_bytes: 0,
            access_counter: 0,
        }
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<crate::ExecutorResult> {
        if self.config.benchmark_mode || !self.config.enabled {
            return None;
        }

        let ttl = Duration::from_secs(self.config.ttl_seconds);
        let access_order = self.next_access_order();

        let result = {
            let entry = self.cache.get_mut(key)?;
            if entry.is_expired(ttl) {
                let _ = entry;
                self.remove(key);
                return None;
            }
            entry.last_access = access_order;
            entry.result.clone()
        };

        Some(result)
    }

    pub fn put(&mut self, key: CacheKey, mut entry: CacheEntry, tables: Vec<String>) {
        if self.config.benchmark_mode || !self.config.enabled {
            return;
        }

        let size = entry.estimate_size();

        if let Some(old_entry) = self.cache.remove(&key) {
            self.current_memory_bytes = self
                .current_memory_bytes
                .saturating_sub(old_entry.size_bytes);
            for t in &old_entry.tables {
                if let Some(keys) = self.table_index.get_mut(t) {
                    keys.remove(&key);
                }
            }
        }

        while self.should_evict(size) {
            if let Some(evict_key) = self.find_lru_entry() {
                self.remove(&evict_key);
            } else {
                break;
            }
        }

        entry.last_access = self.next_access_order();
        self.cache.insert(key.clone(), entry);
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
                    self.current_memory_bytes =
                        self.current_memory_bytes.saturating_sub(entry.size_bytes);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.table_index.clear();
        self.current_memory_bytes = 0;
        self.access_counter = 0;
    }

    pub fn stats(&self) -> QueryCacheStats {
        QueryCacheStats {
            entries: self.cache.len(),
            memory_bytes: self.current_memory_bytes,
            table_count: self.table_index.len(),
        }
    }

    fn next_access_order(&mut self) -> u64 {
        self.access_counter = self.access_counter.wrapping_add(1);
        self.access_counter
    }

    fn should_evict(&self, new_size: usize) -> bool {
        self.cache.len() >= self.config.max_entries
            || self.current_memory_bytes + new_size > self.config.max_memory_bytes
    }

    fn find_lru_entry(&self) -> Option<CacheKey> {
        self.cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_access)
            .map(|(k, _)| k.clone())
    }

    fn remove(&mut self, key: &CacheKey) {
        if let Some(entry) = self.cache.remove(key) {
            self.current_memory_bytes = self.current_memory_bytes.saturating_sub(entry.size_bytes);
        }

        for table_keys in self.table_index.values_mut() {
            table_keys.remove(key);
        }
    }
}

const MAX_RESULT_SIZE_BYTES: usize = 1024 * 1024;
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
        }
        if size > MAX_RESULT_SIZE_BYTES {
            return false;
        }
    }

    true
}

#[derive(Debug, Clone)]
pub struct QueryCacheStats {
    pub entries: usize,
    pub memory_bytes: usize,
    pub table_count: usize,
}

unsafe impl Send for QueryCache {}
unsafe impl Sync for QueryCache {}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_types::Value;

    fn make_key(sql: &str, id: i64) -> CacheKey {
        CacheKey {
            normalized_sql: sql.to_string(),
            params_hash: id as u64,
        }
    }

    fn make_entry(id: i64) -> CacheEntry {
        CacheEntry {
            result: crate::ExecutorResult::new(vec![vec![Value::Integer(id)]], 1),
            tables: vec![],
            created_at: std::time::Instant::now(),
            size_bytes: 64,
            last_access: 0,
        }
    }

    #[test]
    fn test_cache_basic() {
        let config = QueryCacheConfig::default();
        let mut cache = QueryCache::new(config);

        cache.put(make_key("SELECT 1", 1), make_entry(1), vec![]);
        assert!(cache.get(&make_key("SELECT 1", 1)).is_some());
    }

    #[test]
    fn test_cache_lru() {
        let config = QueryCacheConfig {
            max_entries: 2,
            ..Default::default()
        };
        let mut cache = QueryCache::new(config);

        cache.put(make_key("q1", 1), make_entry(1), vec![]);
        cache.put(make_key("q2", 2), make_entry(2), vec![]);

        cache.get(&make_key("q1", 1));

        cache.put(make_key("q3", 3), make_entry(3), vec![]);

        assert!(cache.get(&make_key("q1", 1)).is_some());
        assert!(cache.get(&make_key("q3", 3)).is_some());
    }
}
