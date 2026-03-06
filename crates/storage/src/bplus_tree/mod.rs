//! B+ Tree implementation - simplified version for storage crate

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// B+ Tree index - simplified for storage crate
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BPlusTree {
    /// Map from key to value (sorted by key)
    map: BTreeMap<i64, u32>,
}

impl BPlusTree {
    /// Create a new B+ Tree
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Insert a key-value pair
    pub fn insert(&mut self, key: i64, value: u32) {
        self.map.insert(key, value);
    }

    /// Search for a key, returns value if found
    pub fn search(&self, key: i64) -> Option<u32> {
        self.map.get(&key).copied()
    }

    /// Query all values in range [start, end)
    pub fn range_query(&self, start: i64, end: i64) -> Vec<u32> {
        self.map
            .range(start..end)
            .map(|(_, &v)| v)
            .collect()
    }

    /// Return all keys in sorted order
    pub fn keys(&self) -> Vec<i64> {
        self.map.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bplus_tree_insert_and_search() {
        let mut tree = BPlusTree::new();
        tree.insert(1, 100);
        tree.insert(2, 200);
        tree.insert(3, 300);

        assert_eq!(tree.search(1), Some(100));
        assert_eq!(tree.search(2), Some(200));
        assert_eq!(tree.search(3), Some(300));
        assert_eq!(tree.search(4), None);
    }

    #[test]
    fn test_bplus_tree_range_query() {
        let mut tree = BPlusTree::new();
        tree.insert(1, 100);
        tree.insert(2, 200);
        tree.insert(3, 300);
        tree.insert(4, 400);

        let results = tree.range_query(2, 4);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&200));
        assert!(results.contains(&300));
    }
}
