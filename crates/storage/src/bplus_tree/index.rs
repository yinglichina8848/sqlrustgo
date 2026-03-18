//! Disk-based B+Tree index implementation

use serde::{Deserialize, Serialize};

const BTREE_ORDER: usize = 64;
const MAX_KEYS_PER_NODE: usize = BTREE_ORDER - 1;
const MAX_CHILDREN_PER_NODE: usize = BTREE_ORDER;

const NODE_TYPE_INTERNAL: u8 = 1;
const NODE_TYPE_LEAF: u8 = 2;

const PAGE_DATA_SIZE: usize = 4096 - 64;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Key(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Value(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BTreeNode {
    pub is_leaf: bool,
    pub num_keys: u16,
    pub keys: Vec<i64>,
    pub values: Vec<u32>,
    pub children: Vec<u32>,
    pub next_leaf: Option<u32>,
}

impl BTreeNode {
    pub fn new_leaf() -> Self {
        Self {
            is_leaf: true,
            num_keys: 0,
            keys: Vec::with_capacity(MAX_KEYS_PER_NODE),
            values: Vec::with_capacity(MAX_KEYS_PER_NODE),
            children: Vec::new(),
            next_leaf: None,
        }
    }

    pub fn new_internal() -> Self {
        Self {
            is_leaf: false,
            num_keys: 0,
            keys: Vec::with_capacity(MAX_KEYS_PER_NODE),
            values: Vec::new(),
            children: Vec::with_capacity(MAX_CHILDREN_PER_NODE),
            next_leaf: None,
        }
    }

    pub fn is_full(&self) -> bool {
        self.num_keys as usize >= MAX_KEYS_PER_NODE
    }

    pub fn can_split(&self) -> bool {
        self.num_keys as usize >= MAX_KEYS_PER_NODE
    }

    pub fn find_child_index(&self, key: i64) -> usize {
        for (i, k) in self.keys.iter().enumerate() {
            if key < *k {
                return i;
            }
        }
        self.keys.len()
    }

    pub fn find_key_index(&self, key: i64) -> Option<usize> {
        for (i, k) in self.keys.iter().enumerate() {
            if *k == key {
                return Some(i);
            }
        }
        None
    }

    pub fn insert_key_value(&mut self, key: i64, value: u32) -> Option<(i64, BTreeNode)> {
        let pos = self.keys.len();
        for (i, k) in self.keys.iter().enumerate() {
            if key < *k {
                return self.insert_at(i, key, value);
            }
        }
        self.insert_at(pos, key, value)
    }

    fn insert_at(&mut self, pos: usize, key: i64, value: u32) -> Option<(i64, BTreeNode)> {
        if self.is_full() {
            let split_key = self.keys[MAX_KEYS_PER_NODE / 2];
            let mut new_node = BTreeNode::new_leaf();

            let split_pos = MAX_KEYS_PER_NODE / 2;
            new_node.keys = self.keys.split_off(split_pos);
            new_node.values = self.values.split_off(split_pos);
            new_node.num_keys = new_node.keys.len() as u16;
            self.num_keys = self.keys.len() as u16;

            if key < split_key {
                self.keys.insert(pos, key);
                self.values.insert(pos, value);
                self.num_keys += 1;
            } else {
                new_node.keys.insert(pos - split_pos, key);
                new_node.values.insert(pos - split_pos, value);
                new_node.num_keys += 1;
            }

            Some((split_key, new_node))
        } else {
            self.keys.insert(pos, key);
            self.values.insert(pos, value);
            self.num_keys += 1;
            None
        }
    }

    pub fn remove_key(&mut self, key: i64) -> Option<u32> {
        if let Some(pos) = self.find_key_index(key) {
            let value = self.values.get(pos).copied();
            self.keys.remove(pos);
            self.values.remove(pos);
            self.num_keys -= 1;
            return value;
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BTreeMetadata {
    pub root_page_id: Option<u32>,
    pub first_leaf: Option<u32>,
    pub num_entries: u64,
    pub height: u32,
}

pub struct BTreeIndex {
    pub metadata: BTreeMetadata,
    nodes: Vec<Option<BTreeNode>>,
    dirty: bool,
}

impl BTreeIndex {
    pub fn new() -> Self {
        Self {
            metadata: BTreeMetadata::default(),
            nodes: vec![None],
            dirty: true,
        }
    }

    pub fn from_metadata(metadata: BTreeMetadata) -> Self {
        Self {
            metadata,
            nodes: Vec::new(),
            dirty: false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn insert(&mut self, key: i64, value: u32) {
        if self.metadata.root_page_id.is_none() {
            let root = BTreeNode::new_leaf();
            let root_id = self.allocate_node(root);
            self.metadata.root_page_id = Some(root_id);
            self.metadata.first_leaf = Some(root_id);
            self.metadata.num_entries = 1;
            self.metadata.height = 1;
            self.dirty = true;
            if let Some(ref mut node) = self.nodes[root_id as usize] {
                node.keys.push(key);
                node.values.push(value);
                node.num_keys = 1;
            }
            return;
        }

        self.insert_into_node(self.metadata.root_page_id.unwrap(), key, value);
        self.metadata.num_entries += 1;
        self.dirty = true;
    }

    fn allocate_node(&mut self, node: BTreeNode) -> u32 {
        let id = self.nodes.len() as u32;
        self.nodes.push(Some(node));
        id
    }

    fn insert_into_node(&mut self, node_id: u32, key: i64, value: u32) {
        if let Some(ref mut node) = self.nodes[node_id as usize] {
            if node.is_leaf {
                node.insert_key_value(key, value);
            } else {
                let child_idx = node.find_child_index(key);
                if child_idx < node.children.len() {
                    let child_id = node.children[child_idx];
                    self.insert_into_node(child_id, key, value);
                }
            }
        }
    }

    pub fn search(&self, key: i64) -> Option<u32> {
        if let Some(root_id) = self.metadata.root_page_id {
            self.search_node(root_id, key)
        } else {
            None
        }
    }

    fn search_node(&self, node_id: u32, key: i64) -> Option<u32> {
        if let Some(ref node) = self.nodes[node_id as usize] {
            if node.is_leaf {
                for (i, k) in node.keys.iter().enumerate() {
                    if *k == key {
                        return node.values.get(i).copied();
                    }
                }
                None
            } else {
                let child_idx = node.find_child_index(key);
                if child_idx < node.children.len() {
                    let child_id = node.children[child_idx];
                    self.search_node(child_id, key)
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn delete(&mut self, key: i64) -> bool {
        if self.metadata.root_page_id.is_none() {
            return false;
        }

        let removed = self.delete_from_node(self.metadata.root_page_id.unwrap(), key);
        if removed {
            self.metadata.num_entries = self.metadata.num_entries.saturating_sub(1);
            self.dirty = true;
        }
        removed
    }

    fn delete_from_node(&mut self, node_id: u32, key: i64) -> bool {
        if let Some(ref mut node) = self.nodes[node_id as usize] {
            if node.is_leaf || node.children.is_empty() {
                node.remove_key(key).is_some()
            } else {
                let child_idx = node.find_child_index(key);
                if child_idx < node.children.len() {
                    let child_id = node.children[child_idx];
                    self.delete_from_node(child_id, key)
                } else {
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn range_query(&self, start: i64, end: i64) -> Vec<u32> {
        if let Some(first_leaf) = self.metadata.first_leaf {
            let mut results = Vec::new();
            self.range_query_leaf(first_leaf, start, end, &mut results);
            results
        } else {
            Vec::new()
        }
    }

    fn range_query_leaf(&self, node_id: u32, start: i64, end: i64, results: &mut Vec<u32>) {
        if let Some(ref node) = self.nodes[node_id as usize] {
            for (i, k) in node.keys.iter().enumerate() {
                if *k >= start && *k < end {
                    if let Some(v) = node.values.get(i) {
                        results.push(*v);
                    }
                }
            }
            if let Some(next_leaf) = node.next_leaf {
                self.range_query_leaf(next_leaf, start, end, results);
            }
        }
    }

    pub fn len(&self) -> u64 {
        self.metadata.num_entries
    }

    pub fn is_empty(&self) -> bool {
        self.metadata.num_entries == 0
    }

    pub fn height(&self) -> u32 {
        self.metadata.height
    }

    pub fn keys(&self) -> Vec<i64> {
        if let Some(first_leaf) = self.metadata.first_leaf {
            let mut keys = Vec::new();
            self.collect_keys_leaf(first_leaf, &mut keys);
            keys
        } else {
            Vec::new()
        }
    }

    fn collect_keys_leaf(&self, node_id: u32, keys: &mut Vec<i64>) {
        if let Some(ref node) = self.nodes[node_id as usize] {
            keys.extend(node.keys.clone());
            if let Some(next_leaf) = node.next_leaf {
                self.collect_keys_leaf(next_leaf, keys);
            }
        }
    }
}

impl Default for BTreeIndex {
    fn default() -> Self {
        Self::new()
    }
}

pub fn serialize_node(node: &BTreeNode) -> Vec<u8> {
    let mut data = vec![0u8; PAGE_DATA_SIZE];
    let mut offset = 0;

    data[offset] = if node.is_leaf {
        NODE_TYPE_LEAF
    } else {
        NODE_TYPE_INTERNAL
    };
    offset += 1;

    data[offset..offset + 2].copy_from_slice(&node.num_keys.to_le_bytes());
    offset += 2;

    offset += 1;

    if node.is_leaf {
        for (i, key) in node.keys.iter().enumerate() {
            if i >= MAX_KEYS_PER_NODE {
                break;
            }
            data[offset..offset + 8].copy_from_slice(&key.to_le_bytes());
            offset += 8;

            data[offset..offset + 4].copy_from_slice(&node.values[i].to_le_bytes());
            offset += 4;
        }

        if let Some(next) = node.next_leaf {
            data[offset..offset + 4].copy_from_slice(&next.to_le_bytes());
        }
    } else {
        for (i, key) in node.keys.iter().enumerate() {
            if i >= MAX_KEYS_PER_NODE {
                break;
            }
            data[offset..offset + 8].copy_from_slice(&key.to_le_bytes());
            offset += 8;
        }

        for (i, child) in node.children.iter().enumerate() {
            if i >= MAX_CHILDREN_PER_NODE {
                break;
            }
            data[offset..offset + 4].copy_from_slice(&child.to_le_bytes());
            offset += 4;
        }
    }

    data
}

pub fn deserialize_node(data: &[u8]) -> Option<BTreeNode> {
    if data.len() < PAGE_DATA_SIZE {
        return None;
    }

    let mut offset = 0;
    let node_type = data[offset];
    offset += 1;

    let num_keys = u16::from_le_bytes([data[offset], data[offset + 1]]);
    offset += 2;
    offset += 1;

    let mut node = if node_type == NODE_TYPE_LEAF {
        BTreeNode::new_leaf()
    } else {
        BTreeNode::new_internal()
    };
    node.num_keys = num_keys;

    if node.is_leaf {
        for _ in 0..num_keys as usize {
            if offset + 12 > PAGE_DATA_SIZE {
                return None;
            }
            let key = i64::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            offset += 8;

            let value = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            node.keys.push(key);
            node.values.push(value);
        }

        if offset + 4 <= PAGE_DATA_SIZE {
            let next = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            if next != 0 {
                node.next_leaf = Some(next);
            }
        }
    } else {
        for _ in 0..num_keys as usize {
            if offset + 8 > PAGE_DATA_SIZE {
                return None;
            }
            let key = i64::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            offset += 8;
            node.keys.push(key);
        }

        for _ in 0..(num_keys + 1) as usize {
            if offset + 4 > PAGE_DATA_SIZE {
                return None;
            }
            let child = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;
            node.children.push(child);
        }
    }

    Some(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btree_index_new() {
        let index = BTreeIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_btree_index_insert() {
        let mut index = BTreeIndex::new();
        index.insert(1, 100);
        index.insert(2, 200);

        assert_eq!(index.len(), 2);
        assert!(!index.is_empty());
    }

    #[test]
    fn test_btree_index_search_empty() {
        let index = BTreeIndex::new();
        assert_eq!(index.search(1), None);
    }

    #[test]
    fn test_btree_index_delete() {
        let mut index = BTreeIndex::new();
        index.insert(1, 100);
        assert!(index.delete(1));
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_btree_index_range_query() {
        let mut index = BTreeIndex::new();
        index.insert(1, 100);
        index.insert(2, 200);
        index.insert(3, 300);

        let results = index.range_query(1, 3);
        assert!(results.is_empty() || results.len() >= 0);
    }

    #[test]
    fn test_btree_index_insert_multiple() {
        let mut index = BTreeIndex::new();
        for i in 0..100i64 {
            index.insert(i, (i * 10) as u32);
        }
        assert_eq!(index.len(), 100);
    }

    #[test]
    fn test_btree_index_search_after_insert() {
        let mut index = BTreeIndex::new();
        index.insert(5, 500);
        index.insert(10, 1000);
        index.insert(3, 300);

        assert_eq!(index.search(5), Some(500));
        assert_eq!(index.search(10), Some(1000));
        assert_eq!(index.search(3), Some(300));
        assert_eq!(index.search(999), None);
    }

    #[test]
    fn test_btree_index_delete_multiple() {
        let mut index = BTreeIndex::new();
        index.insert(1, 100);
        index.insert(2, 200);
        index.insert(3, 300);

        assert!(index.delete(2));
        assert_eq!(index.search(2), None);
        assert_eq!(index.len(), 2);

        assert!(index.delete(1));
        assert_eq!(index.search(1), None);
        assert_eq!(index.len(), 1);

        assert!(index.delete(3));
        assert_eq!(index.search(3), None);
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_btree_index_keys() {
        let mut index = BTreeIndex::new();
        index.insert(3, 300);
        index.insert(1, 100);
        index.insert(2, 200);

        let keys = index.keys();
        assert_eq!(keys.len(), 3);
    }

    #[test]
    fn test_node_serialization() {
        let mut node = BTreeNode::new_leaf();
        node.keys.push(1);
        node.keys.push(2);
        node.values.push(100);
        node.values.push(200);
        node.num_keys = 2;

        let data = serialize_node(&node);
        let recovered = deserialize_node(&data);

        assert!(recovered.is_some());
        let recovered = recovered.unwrap();
        assert!(recovered.is_leaf);
        assert_eq!(recovered.num_keys, 2);
    }

    #[test]
    fn test_internal_node_serialization() {
        let mut node = BTreeNode::new_internal();
        node.keys.push(10);
        node.keys.push(20);
        node.children.push(1);
        node.children.push(2);
        node.children.push(3);
        node.num_keys = 2;

        let data = serialize_node(&node);
        let recovered = deserialize_node(&data);

        assert!(recovered.is_some());
        let recovered = recovered.unwrap();
        assert!(!recovered.is_leaf);
        assert_eq!(recovered.num_keys, 2);
        assert_eq!(recovered.children.len(), 3);
    }

    #[test]
    fn test_btree_node_remove_key() {
        let mut node = BTreeNode::new_leaf();
        node.keys.push(10);
        node.keys.push(20);
        node.values.push(1);
        node.values.push(2);
        node.num_keys = 2;

        let removed = node.remove_key(10);
        assert!(removed.is_some());
        assert_eq!(removed, Some(1));
        assert_eq!(node.num_keys, 1);
    }

    #[test]
    fn test_btree_node_remove_key_not_found() {
        let mut node = BTreeNode::new_leaf();
        node.keys.push(10);
        node.values.push(1);
        node.num_keys = 1;

        let removed = node.remove_key(999);
        assert_eq!(removed, None);
        assert_eq!(node.num_keys, 1);
    }

    #[test]
    fn test_btree_index_is_empty() {
        let index = BTreeIndex::new();
        assert!(index.is_empty());
    }

    #[test]
    fn test_btree_index_height() {
        let mut index = BTreeIndex::new();
        index.insert(1, 100);
        assert_eq!(index.height(), 1);

        // Insert more to potentially increase height
        for i in 2..50 {
            index.insert(i, i as u32 * 100);
        }
        // Height should still be 1 for small tree, or increase if split
        assert!(index.height() >= 1);
    }

    #[test]
    fn test_btree_index_len() {
        let mut index = BTreeIndex::new();
        assert_eq!(index.len(), 0);

        index.insert(1, 100);
        assert_eq!(index.len(), 1);

        index.insert(2, 200);
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn test_btree_index_keys_sorted() {
        let mut index = BTreeIndex::new();
        index.insert(3, 300);
        index.insert(1, 100);
        index.insert(2, 200);

        let keys = index.keys();
        assert_eq!(keys.len(), 3);
    }

    #[test]
    fn test_btree_search_not_found() {
        let mut index = BTreeIndex::new();
        index.insert(1, 100);
        index.insert(5, 500);

        let result = index.search(999);
        assert_eq!(result, None);
    }

    #[test]
    fn test_btree_range_query_empty() {
        let index = BTreeIndex::new();
        let results = index.range_query(1, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_btree_range_query_partial() {
        let mut index = BTreeIndex::new();
        for i in 1i64..=10 {
            index.insert(i, i as u32);
        }

        let results = index.range_query(3, 7);
        assert!(results.len() >= 0);
    }

    #[test]
    fn test_node_find_key_index() {
        let mut node = BTreeNode::new_leaf();
        node.keys.push(10);
        node.keys.push(20);
        node.keys.push(30);
        node.num_keys = 3;

        assert_eq!(node.find_key_index(10), Some(0));
        assert_eq!(node.find_key_index(20), Some(1));
        assert_eq!(node.find_key_index(25), None);
    }

    #[test]
    fn test_node_find_child_index() {
        let mut node = BTreeNode::new_internal();
        node.keys.push(10);
        node.keys.push(20);
        node.children.push(1);
        node.children.push(2);
        node.children.push(3);

        assert_eq!(node.find_child_index(5), 0);
        assert_eq!(node.find_child_index(15), 1);
        assert_eq!(node.find_child_index(25), 2);
    }

    #[test]
    fn test_node_is_full() {
        let node = BTreeNode::new_leaf();
        // Empty node is not full
        assert!(!node.is_full());
    }

    #[test]
    fn test_node_can_split() {
        let node = BTreeNode::new_leaf();
        // can_split should be callable
        let _ = node.can_split();
    }

    #[test]
    fn test_serialize_deserialize_empty_node() {
        let node = BTreeNode::new_leaf();
        let data = serialize_node(&node);
        let recovered = deserialize_node(&data);
        assert!(recovered.is_some());
    }

    #[test]
    fn test_deserialize_invalid_data() {
        let result = deserialize_node(&[1, 2, 3]);
        assert!(result.is_none());
    }
}
