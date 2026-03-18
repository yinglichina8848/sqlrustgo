//! B+ Tree implementation with page-based persistence
//!
//! This module provides a disk-based B+Tree index that uses the page storage
//! infrastructure for persistence.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod index;

pub use index::{
    deserialize_node, serialize_node, BTreeIndex, BTreeMetadata, BTreeNode, Key, Value,
};

pub type BPlusTree = SimpleBPlusTree;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SimpleBPlusTree {
    map: BTreeMap<i64, u32>,
}

impl SimpleBPlusTree {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn insert(&mut self, key: i64, value: u32) {
        self.map.insert(key, value);
    }

    pub fn search(&self, key: i64) -> Option<u32> {
        self.map.get(&key).copied()
    }

    pub fn range_query(&self, start: i64, end: i64) -> Vec<u32> {
        self.map.range(start..end).map(|(_, &v)| v).collect()
    }

    pub fn keys(&self) -> Vec<i64> {
        self.map.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_bplus_tree_insert_and_search() {
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
    fn test_simple_bplus_tree_range_query() {
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
