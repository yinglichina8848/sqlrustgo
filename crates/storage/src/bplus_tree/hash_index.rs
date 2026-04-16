//! Hash Index Implementation
//!
//! Provides O(1) point lookup using std::collections::HashMap.
//! Suitable for equality queries on primary/unique keys.

use parking_lot::RwLock;
use std::collections::HashMap;

/// Hash index for fast point lookups
/// Provides O(1) average lookup time for equality predicates
pub struct HashIndex<K, V> {
    /// Internal hash map
    map: RwLock<HashMap<K, V>>,
    /// Number of entries
    len: RwLock<usize>,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> HashIndex<K, V> {
    /// Create a new empty HashIndex
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
            len: RwLock::new(0),
        }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: RwLock::new(HashMap::with_capacity(capacity)),
            len: RwLock::new(0),
        }
    }

    /// Insert a key-value pair
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let mut map = self.map.write();
        let old = map.insert(key.clone(), value);
        if old.is_none() {
            *self.len.write() += 1;
        }
        old
    }

    /// Get value by key - O(1) average
    pub fn get(&self, key: &K) -> Option<V> {
        self.map.read().get(key).cloned()
    }

    /// Check if key exists
    pub fn contains(&self, key: &K) -> bool {
        self.map.read().contains_key(key)
    }

    /// Remove a key
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut map = self.map.write();
        let removed = map.remove(key);
        if removed.is_some() {
            *self.len.write() -= 1;
        }
        removed
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        *self.len.read()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over all keys
    pub fn keys(&self) -> Vec<K> {
        self.map.read().keys().cloned().collect()
    }

    /// Iterate over all values
    pub fn values(&self) -> Vec<V> {
        self.map.read().values().cloned().collect()
    }

    /// Get all entries as vector
    pub fn entries(&self) -> Vec<(K, V)> {
        self.map
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Batch insert from iterator
    pub fn batch_insert(&self, entries: impl IntoIterator<Item = (K, V)>) -> usize {
        let mut map = self.map.write();
        let mut inserted = 0;
        for (key, value) in entries {
            if map.insert(key, value).is_none() {
                inserted += 1;
            }
        }
        *self.len.write() = map.len();
        inserted
    }

    /// Find all keys matching a predicate
    pub fn filter<F>(&self, predicate: F) -> Vec<V>
    where
        F: Fn(&K) -> bool,
    {
        self.map
            .read()
            .iter()
            .filter(|(k, _)| predicate(k))
            .map(|(_, v)| v.clone())
            .collect()
    }
}

impl<K: std::hash::Hash + Eq, V> Default for HashIndex<K, V> {
    fn default() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
            len: RwLock::new(0),
        }
    }
}

/// Thread-safe wrapper for HashIndex
pub type SharedHashIndex<K, V> = std::sync::Arc<HashIndex<K, V>>;

/// Constructor for SharedHashIndex
pub fn new_shared_hash_index<K: std::hash::Hash + Eq + Clone, V: Clone>() -> SharedHashIndex<K, V> {
    std::sync::Arc::new(HashIndex::new())
}

/// Constructor with capacity for SharedHashIndex
pub fn shared_hash_index_with_capacity<K: std::hash::Hash + Eq + Clone, V: Clone>(
    capacity: usize,
) -> SharedHashIndex<K, V> {
    std::sync::Arc::new(HashIndex::with_capacity(capacity))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_index_basic() {
        let index = HashIndex::new();

        index.insert(1, "one".to_string());
        index.insert(2, "two".to_string());
        index.insert(3, "three".to_string());

        assert_eq!(index.len(), 3);
        assert_eq!(index.get(&1), Some("one".to_string()));
        assert_eq!(index.get(&2), Some("two".to_string()));
        assert!(index.contains(&3));
        assert!(!index.contains(&4));
    }

    #[test]
    fn test_hash_index_update() {
        let index = HashIndex::new();

        index.insert(1, "one".to_string());
        let old = index.insert(1, "ONE".to_string());

        assert_eq!(old, Some("one".to_string()));
        assert_eq!(index.get(&1), Some("ONE".to_string()));
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_hash_index_remove() {
        let index = HashIndex::new();

        index.insert(1, "one".to_string());
        assert!(index.contains(&1));

        let removed = index.remove(&1);
        assert_eq!(removed, Some("one".to_string()));
        assert!(!index.contains(&1));
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_hash_index_batch_insert() {
        let index = HashIndex::new();

        let entries = vec![
            (1, "one".to_string()),
            (2, "two".to_string()),
            (3, "three".to_string()),
            (4, "four".to_string()),
        ];

        let inserted = index.batch_insert(entries);
        assert_eq!(inserted, 4);
        assert_eq!(index.len(), 4);
    }

    #[test]
    fn test_hash_index_filter() {
        let index = HashIndex::new();

        index.insert(1, "one".to_string());
        index.insert(2, "two".to_string());
        index.insert(3, "three".to_string());
        index.insert(4, "four".to_string());

        // Filter keys > 2
        let filtered: Vec<String> = index.filter(|k| *k > 2);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_hash_index_string_keys() {
        let index: HashIndex<String, i32> = HashIndex::new();

        index.insert("apple".to_string(), 1);
        index.insert("banana".to_string(), 2);

        assert_eq!(index.get(&"apple".to_string()), Some(1));
        assert_eq!(index.get(&"banana".to_string()), Some(2));
    }

    #[test]
    fn test_shared_hash_index() {
        let index: SharedHashIndex<i32, String> = new_shared_hash_index();

        index.insert(1, "one".to_string());

        assert_eq!(index.get(&1), Some("one".to_string()));
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_hash_index_entries() {
        let index = HashIndex::new();

        index.insert(1, "one".to_string());
        index.insert(2, "two".to_string());

        let entries = index.entries();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&(1, "one".to_string())));
    }

    #[test]
    fn test_hash_index_is_empty() {
        let index = HashIndex::new();
        assert!(index.is_empty());

        index.insert(1, "one".to_string());
        assert!(!index.is_empty());
    }

    #[test]
    fn test_hash_index_remove_nonexistent() {
        let index = HashIndex::new();

        index.insert(1, "one".to_string());
        let removed = index.remove(&999);
        assert!(removed.is_none());
        assert_eq!(index.len(), 1);
    }
}
