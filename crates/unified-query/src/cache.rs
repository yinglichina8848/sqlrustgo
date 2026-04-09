use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub struct QueryCache {
    cache: Arc<RwLock<HashMap<String, Vec<i32>>>>,
    order: Arc<RwLock<Vec<String>>>,
    capacity: usize,
}

impl QueryCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            order: Arc::new(RwLock::new(Vec::new())),
            capacity,
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<i32>> {
        let cache = self.cache.read();
        cache.get(key).cloned()
    }

    pub fn insert(&self, key: String, value: Vec<i32>) {
        let mut cache = self.cache.write();
        let mut order = self.order.write();

        if cache.len() >= self.capacity {
            // Remove oldest entry (first element) for LRU
            if !order.is_empty() {
                let oldest_key = order.remove(0);
                cache.remove(&oldest_key);
            }
        }

        order.push(key.clone());
        cache.insert(key, value);
    }

    pub fn clear(&self) {
        self.cache.write().clear();
        self.order.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic() {
        let cache = QueryCache::new(2);
        cache.insert("key1".to_string(), vec![1, 2, 3]);
        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = QueryCache::new(2);
        cache.insert("key1".to_string(), vec![1]);
        cache.insert("key2".to_string(), vec![2]);
        cache.insert("key3".to_string(), vec![3]);
        assert!(cache.get("key1").is_none());
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
    }

    #[test]
    fn test_cache_miss() {
        let cache = QueryCache::new(1);
        assert!(cache.get("nonexistent").is_none());
    }
}
