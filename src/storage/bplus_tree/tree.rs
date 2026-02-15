//! B+ Tree implementation with proper node structure

use serde::{Deserialize, Serialize};

/// Maximum keys per node (fanout)
const MAX_KEYS: usize = 4;
/// Minimum keys per node (for split)
const MIN_KEYS: usize = 2;

/// B+ Tree index
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

    /// Insert a key-value pair
    pub fn insert(&mut self, key: i64, value: u32) {
        let key = Key(key);

        if self.root.is_none() {
            let mut leaf = LeafNode::new_leaf();
            leaf.keys.push(key);
            leaf.values.push(value);
            self.root = Some(Node::Leaf(leaf));
            self.len = 1;
            return;
        }

        // Check if root is Leaf or Internal
        let is_leaf = matches!(self.root, Some(Node::Leaf(_)));

        if is_leaf {
            if let Some(Node::Leaf(leaf)) = self.root.as_mut() {
                if leaf.keys.len() < MAX_KEYS {
                    leaf.insert_sorted(key, value);
                    self.len += 1;
                    return;
                }
            }
            // Need to split root leaf
            let old_root = self.root.take().unwrap();
            if let Node::Leaf(leaf) = old_root {
                let mut keys = leaf.keys;
                let mut values = leaf.values;
                keys.push(key);
                values.push(value);

                let mut pairs: Vec<_> = keys.into_iter().zip(values.into_iter()).collect();
                pairs.sort_by_key(|(k, _)| *k);
                let keys: Vec<Key> = pairs.iter().map(|(k, _)| k.clone()).collect();
                let values: Vec<u32> = pairs.iter().map(|(_, v)| *v).collect();

                let mid = (keys.len() + 1) / 2;
                let left_keys: Vec<Key> = keys[..mid].to_vec();
                let left_values: Vec<u32> = values[..mid].to_vec();
                let right_keys: Vec<Key> = keys[mid..].to_vec();
                let right_values: Vec<u32> = values[mid..].to_vec();

                let mut left_leaf = LeafNode::new_leaf();
                left_leaf.keys = left_keys;
                left_leaf.values = left_values;

                let mut right_leaf = LeafNode::new_leaf();
                right_leaf.keys = right_keys;
                right_leaf.values = right_values;

                let mid_key = right_leaf.keys[0].clone();

                let internal = InternalNode::new_internal_with_children(
                    mid_key,
                    NodeBox::Leaf(left_leaf),
                    NodeBox::Leaf(right_leaf),
                );
                self.root = Some(Node::Internal(internal));
                self.len += 1;
            }
        } else {
            // Root is Internal - take out, process, then put back
            let mut internal_node = self.root.take().unwrap();
            if let Node::Internal(internal) = &mut internal_node {
                self.insert_internal(internal, key, value);
            }
            self.root = Some(internal_node);
        }
    }

    /// Insert into internal node
    fn insert_internal(&mut self, internal: &mut InternalNode, key: Key, value: u32) {
        // Find the child to descend into
        let pos = internal.find_child_position(&key);

        match &mut internal.children[pos] {
            NodeBox::Leaf(child) => {
                if child.keys.len() < MAX_KEYS {
                    child.insert_sorted(key, value);
                    self.len += 1;
                } else {
                    // Split leaf child
                    let mut keys = std::mem::take(&mut child.keys);
                    let mut values = std::mem::take(&mut child.values);
                    keys.push(key);
                    values.push(value);

                    let mut pairs: Vec<_> = keys.into_iter().zip(values.into_iter()).collect();
                    pairs.sort_by_key(|(k, _)| *k);
                    let keys: Vec<Key> = pairs.iter().map(|(k, _)| k.clone()).collect();
                    let values: Vec<u32> = pairs.iter().map(|(_, v)| *v).collect();

                    let mid = (keys.len() + 1) / 2;
                    let left_keys: Vec<Key> = keys[..mid].to_vec();
                    let left_values: Vec<u32> = values[..mid].to_vec();
                    let right_keys: Vec<Key> = keys[mid..].to_vec();
                    let right_values: Vec<u32> = values[mid..].to_vec();

                    let mut left_leaf = LeafNode::new_leaf();
                    left_leaf.keys = left_keys;
                    left_leaf.values = left_values;

                    let mut right_leaf = LeafNode::new_leaf();
                    right_leaf.keys = right_keys;
                    right_leaf.values = right_values;

                    let mid_key = right_leaf.keys[0].clone();

                    // Replace child with two children
                    internal.keys.insert(pos, mid_key);
                    internal.children.remove(pos);
                    internal.children.insert(pos, NodeBox::Leaf(left_leaf));
                    internal.children.insert(pos + 1, NodeBox::Leaf(right_leaf));
                    self.len += 1;
                }
            }
            NodeBox::Internal(child) => {
                self.insert_internal(child, key, value);
            }
        }
    }

    /// Search for a key
    pub fn search(&self, key: i64) -> Option<u32> {
        let key = Key(key);
        self.search_node(self.root.as_ref()?, &key)
    }

    fn search_node(&self, node: &Node, key: &Key) -> Option<u32> {
        match node {
            Node::Leaf(leaf) => leaf.search(key),
            Node::Internal(internal) => {
                let pos = internal.find_child_position(key);
                self.search_node(&internal.children[pos].as_node(), key)
            }
        }
    }

    /// Range query: [start, end)
    pub fn range_query(&self, start: i64, end: i64) -> Vec<u32> {
        let start_key = Key(start);
        let end_key = Key(end);

        if self.root.is_none() {
            return vec![];
        }

        self.range_query_node(self.root.as_ref().unwrap(), &start_key, &end_key)
    }

    fn range_query_node(&self, node: &Node, start: &Key, end: &Key) -> Vec<u32> {
        match node {
            Node::Leaf(leaf) => leaf.range_query(start, end),
            Node::Internal(internal) => {
                let mut results = Vec::new();
                for child in &internal.children {
                    results.extend(self.range_query_node(&child.as_node(), start, end));
                }
                results
            }
        }
    }

    /// Get all keys in order
    pub fn keys(&self) -> Vec<i64> {
        if let Some(root) = &self.root {
            self.collect_keys(root)
        } else {
            vec![]
        }
    }

    fn collect_keys(&self, node: &Node) -> Vec<i64> {
        match node {
            Node::Leaf(leaf) => leaf.keys.iter().map(|k| k.0).collect(),
            Node::Internal(internal) => {
                let mut keys = Vec::new();
                for child in &internal.children {
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

/// Wrapper for key comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Key(i64);

/// B+ Tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
enum Node {
    Leaf(LeafNode),
    Internal(InternalNode),
}

/// Leaf node - stores actual data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LeafNode {
    keys: Vec<Key>,
    values: Vec<u32>,
    next: Option<usize>, // For linked list of leaf nodes
}

impl LeafNode {
    fn new_leaf() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            next: None,
        }
    }

    fn insert_sorted(&mut self, key: Key, value: u32) {
        // Binary search for position
        let pos = self.keys.binary_search(&key).unwrap_or_else(|e| e);
        self.keys.insert(pos, key);
        self.values.insert(pos, value);
    }

    fn search(&self, key: &Key) -> Option<u32> {
        self.keys
            .binary_search(key)
            .ok()
            .and_then(|i| self.values.get(i).copied())
    }

    fn range_query(&self, start: &Key, end: &Key) -> Vec<u32> {
        let start_pos = self.keys.binary_search(start).unwrap_or_else(|e| e);
        let end_pos = self.keys.binary_search(end).unwrap_or_else(|e| e);
        self.values[start_pos..end_pos].to_vec()
    }
}

/// Internal node - points to child nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InternalNode {
    keys: Vec<Key>,           // Separator keys
    children: Vec<NodeBox>,  // Child pointers
}

impl InternalNode {
    fn new_internal() -> Self {
        Self {
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    fn new_internal_with_children(mid_key: Key, left: NodeBox, right: NodeBox) -> Self {
        Self {
            keys: vec![mid_key],
            children: vec![left, right],
        }
    }

    fn find_child_position(&self, key: &Key) -> usize {
        // Find first key > key
        self.keys.iter().position(|k| k > key).unwrap_or(self.keys.len())
    }
}

/// Boxed node for type erasure
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
}
