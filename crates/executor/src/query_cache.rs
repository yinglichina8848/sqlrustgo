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
        if !self.config.enabled {
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
        if !self.config.enabled {
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
