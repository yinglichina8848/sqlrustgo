//! B+ Tree implementation - minimal version for storage crate

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
