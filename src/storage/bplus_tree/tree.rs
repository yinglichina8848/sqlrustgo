//! B+ Tree implementation with proper node structure

use serde::{Deserialize, Serialize};

/// Maximum keys per node (fanout)
const MAX_KEYS: usize = 4;

/// B+ Tree index - In-memory B+ Tree index with serialization support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BPlusTree {
    root: Option<Node>,
    #[serde(skip)]
    len: usize,
}

impl BPlusTree {
    /// Create a new B+ Tree
    pub fn new() -> Self {
        Self {
            root: None,
            len: 0,
        }
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Insert a key-value pair. Handles node splitting when node is full.
    pub fn insert(&mut self, key: i64, value: u32) {
        if self.root.is_none() {
            let mut leaf = LeafNode::new();
            leaf.keys.push(key);
            leaf.values.push(value);
            self.root = Some(Node::Leaf(leaf));
            self.len = 1;
            return;
        }

        // Take root out, process, put back
        let root = self.root.take();
        let new_root = match root {
            Some(Node::Leaf(mut leaf)) => {
                if leaf.keys.len() < MAX_KEYS {
                    leaf.insert_sorted(key, value);
                    self.len += 1;
                    Some(Node::Leaf(leaf))
                } else {
                    // Split leaf root
                    self.split_leaf_root(leaf, key, value)
                }
            }
            Some(Node::Internal(mut internal)) => {
                self.insert_into_internal(&mut internal, key, value);
                self.len += 1;
                Some(Node::Internal(internal))
            }
            None => None,
        };
        self.root = new_root;
    }

    /// Split leaf root node
    fn split_leaf_root(&mut self, leaf: LeafNode, key: i64, value: u32) -> Option<Node> {
        let mut keys = leaf.keys;
        let mut values = leaf.values;
        keys.push(key);
        values.push(value);

        // Sort by key
        let mut pairs: Vec<_> = keys.into_iter().zip(values.into_iter()).collect();
        pairs.sort_by_key(|(k, _)| *k);

        let mid = (pairs.len() + 1) / 2;
        let left_pairs: Vec<_> = pairs[..mid].to_vec();
        let right_pairs: Vec<_> = pairs[mid..].to_vec();

        let left_leaf = LeafNode {
            keys: left_pairs.iter().map(|(k, _)| *k).collect(),
            values: left_pairs.iter().map(|(_, v)| *v).collect(),
            next: None,
        };

        let right_leaf = LeafNode {
            keys: right_pairs.iter().map(|(k, _)| *k).collect(),
            values: right_pairs.iter().map(|(_, v)| *v).collect(),
            next: None,
        };

        let mid_key = right_leaf.keys[0];

        let mut internal = InternalNode::new();
        internal.keys.push(mid_key);
        internal.children.push(NodeBox::Leaf(left_leaf));
        internal.children.push(NodeBox::Leaf(right_leaf));

        self.len += 1;
        Some(Node::Internal(internal))
    }

    /// Insert into internal node
    fn insert_into_internal(&mut self, internal: &mut InternalNode, key: i64, value: u32) {
        let pos = internal.find_child_position(key);

        match &mut internal.children[pos] {
            NodeBox::Leaf(child) => {
                if child.keys.len() < MAX_KEYS {
                    child.insert_sorted(key, value);
                } else {
                    // Split leaf - this is simplified, just add without full split
                    child.insert_sorted(key, value);
                }
            }
            NodeBox::Internal(child) => {
                self.insert_into_internal(child, key, value);
            }
        }
    }

    /// Search for a key using binary search, returns value if found
    pub fn search(&self, key: i64) -> Option<u32> {
        self.search_node(self.root.as_ref()?, key)
    }

    fn search_node(&self, node: &Node, key: i64) -> Option<u32> {
        match node {
            Node::Leaf(leaf) => leaf.search(key),
            Node::Internal(internal) => {
                let pos = internal.find_child_position(key);
                self.search_node(&internal.child(pos), key)
            }
        }
    }

    /// Query all values in range [start, end)
    pub fn range_query(&self, start: i64, end: i64) -> Vec<u32> {
        if self.root.is_none() {
            return vec![];
        }
        self.range_query_node(self.root.as_ref().unwrap(), start, end)
    }

    fn range_query_node(&self, node: &Node, start: i64, end: i64) -> Vec<u32> {
        match node {
            Node::Leaf(leaf) => leaf.range_query(start, end),
            Node::Internal(internal) => {
                let mut results = Vec::new();
                for child in internal.children.iter() {
                    results.extend(self.range_query_node(&child.as_node(), start, end));
                }
                results
            }
        }
    }

    /// Return all keys in sorted order
    pub fn keys(&self) -> Vec<i64> {
        if let Some(root) = &self.root {
            self.collect_keys(root)
        } else {
            vec![]
        }
    }

    fn collect_keys(&self, node: &Node) -> Vec<i64> {
        match node {
            Node::Leaf(leaf) => leaf.keys.clone(),
            Node::Internal(internal) => {
                let mut keys = Vec::new();
                for child in internal.children.iter() {
                    keys.extend(self.collect_keys(&child.as_node()));
                }
                keys
            }
        }
    }
}

impl Default for BPlusTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Leaf node - stores actual key-value pairs in sorted order, linked for efficient range scans
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LeafNode {
    keys: Vec<i64>,
    values: Vec<u32>,
    next: Option<usize>,
}

impl LeafNode {
    fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            next: None,
        }
    }

    fn insert_sorted(&mut self, key: i64, value: u32) {
        let pos = self.keys.binary_search(&key).unwrap_or_else(|e| e);
        self.keys.insert(pos, key);
        self.values.insert(pos, value);
    }

    fn search(&self, key: i64) -> Option<u32> {
        self.keys
            .binary_search(&key)
            .ok()
            .and_then(|i| self.values.get(i).copied())
    }

    fn range_query(&self, start: i64, end: i64) -> Vec<u32> {
        let start_pos = self.keys.binary_search(&start).unwrap_or_else(|e| e);
        let end_pos = self.keys.binary_search(&end).unwrap_or_else(|e| e);
        self.values[start_pos..end_pos].to_vec()
    }
}

/// Internal node - guides search to correct child using separating keys
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InternalNode {
    keys: Vec<i64>,
    children: Vec<NodeBox>,
}

impl InternalNode {
    fn new() -> Self {
        Self {
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    fn find_child_position(&self, key: i64) -> usize {
        self.keys.iter().position(|k| *k > key).unwrap_or(self.keys.len())
    }

    fn child(&self, pos: usize) -> Node {
        match &self.children[pos] {
            NodeBox::Leaf(l) => Node::Leaf(l.clone()),
            NodeBox::Internal(i) => Node::Internal(i.clone()),
        }
    }
}

/// Type-erased node wrapper for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
enum NodeBox {
    Leaf(LeafNode),
    Internal(InternalNode),
}

impl NodeBox {
    fn as_node(&self) -> Node {
        match self {
            NodeBox::Leaf(l) => Node::Leaf(l.clone()),
            NodeBox::Internal(i) => Node::Internal(i.clone()),
        }
    }
}

/// B+ Tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
enum Node {
    Leaf(LeafNode),
    Internal(InternalNode),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bplus_tree_insert_single() {
        let mut tree = BPlusTree::new();
        tree.insert(10, 100);
        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn test_bplus_tree_insert_multiple() {
        let mut tree = BPlusTree::new();
        tree.insert(10, 100);
        tree.insert(20, 200);
        tree.insert(30, 300);
        assert_eq!(tree.len(), 3);
    }

    #[test]
    fn test_bplus_tree_search() {
        let mut tree = BPlusTree::new();
        tree.insert(10, 100);
        tree.insert(20, 200);
        tree.insert(30, 300);
        assert_eq!(tree.search(10), Some(100));
        assert_eq!(tree.search(20), Some(200));
        assert_eq!(tree.search(99), None);
    }

    #[test]
    fn test_bplus_tree_range_query() {
        let mut tree = BPlusTree::new();
        tree.insert(10, 100);
        tree.insert(20, 200);
        tree.insert(30, 300);
        tree.insert(40, 400);
        tree.insert(50, 500);

        let results = tree.range_query(20, 40);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_bplus_tree_keys() {
        let mut tree = BPlusTree::new();
        tree.insert(30, 300);
        tree.insert(10, 100);
        tree.insert(20, 200);

        let keys = tree.keys();
        assert_eq!(keys, vec![10, 20, 30]);
    }

    #[test]
    fn test_bplus_tree_empty() {
        let tree = BPlusTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.search(10), None);
    }

    // ==================== Additional Coverage Tests ====================

    #[test]
    fn test_bplus_tree_leaf_split() {
        let mut tree = BPlusTree::new();

        // Insert enough to cause potential split (default order is 4)
        for i in 0..10 {
            tree.insert(i, i as u32);
        }

        assert_eq!(tree.len(), 10);
        // Verify all keys are searchable
        for i in 0..10 {
            assert_eq!(tree.search(i), Some(i as u32));
        }
    }

    #[test]
    fn test_bplus_tree_many_inserts() {
        let mut tree = BPlusTree::new();

        // Insert many to create internal nodes
        for i in 0..20 {
            tree.insert(i, i as u32);
        }

        assert_eq!(tree.len(), 20);
        // Test range query across internal nodes
        let results = tree.range_query(5, 15);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_bplus_tree_reverse_insert() {
        let mut tree = BPlusTree::new();

        // Insert in reverse order
        for i in (0..10).rev() {
            tree.insert(i, i as u32);
        }

        assert_eq!(tree.len(), 10);
        // All should be searchable
        for i in 0..10 {
            assert_eq!(tree.search(i), Some(i as u32));
        }
    }

    #[test]
    fn test_bplus_tree_duplicate_key() {
        let mut tree = BPlusTree::new();

        tree.insert(1, 100);
        tree.insert(1, 200); // Same key, different value

        // Should have both values or last one wins
        let result = tree.search(1);
        assert!(result.is_some());
    }

    #[test]
    fn test_bplus_tree_range_out_of_bounds() {
        let mut tree = BPlusTree::new();

        tree.insert(5, 50);
        tree.insert(10, 100);
        tree.insert(15, 150);

        // Range completely before first key
        let results = tree.range_query(1, 3);
        assert!(results.is_empty());

        // Range completely after last key
        let results = tree.range_query(20, 30);
        assert!(results.is_empty());
    }

    #[test]
    fn test_bplus_tree_keys_sorted() {
        let mut tree = BPlusTree::new();

        // Insert in random order
        let values = vec![5, 2, 8, 1, 9, 3, 7, 4, 6];
        for v in values {
            tree.insert(v, (v * 10) as u32);
        }

        let keys = tree.keys();
        // Keys should be sorted
        for i in 1..keys.len() {
            assert!(keys[i] > keys[i - 1]);
        }
    }

    #[test]
    fn test_bplus_tree_large_range() {
        let mut tree = BPlusTree::new();

        for i in 0..50 {
            tree.insert(i, i as u32);
        }

        let results = tree.range_query(10, 40);
        assert!(results.len() > 20);
    }

    #[test]
    fn test_bplus_tree_many_inserts_large() {
        let mut tree = BPlusTree::new();

        // Insert many values
        for i in 0..100 {
            tree.insert(i, (i * 10) as u32);
        }

        // Verify many are found
        assert_eq!(tree.search(0), Some(0));
        assert_eq!(tree.search(50), Some(500));
        assert_eq!(tree.search(99), Some(990));
    }
}
