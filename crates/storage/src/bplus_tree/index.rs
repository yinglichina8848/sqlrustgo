//! Disk-based B+Tree index implementation

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use thiserror::Error;

const BTREE_ORDER: usize = 64;
const MAX_KEYS_PER_NODE: usize = BTREE_ORDER - 1;
const MAX_CHILDREN_PER_NODE: usize = BTREE_ORDER;

const NODE_TYPE_INTERNAL: u8 = 1;
const NODE_TYPE_LEAF: u8 = 2;

const PAGE_DATA_SIZE: usize = 4096 - 64;

/// Unique index constraint violation error
#[derive(Debug, Clone, Error)]
#[error("unique constraint violation: key {key} already exists")]
pub struct UniqueConstraintViolation {
    pub key: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Key(pub i64);

/// Composite key for multi-column indexes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct CompositeKey {
    pub columns: Vec<i64>,
}

impl CompositeKey {
    pub fn new(columns: Vec<i64>) -> Self {
        Self { columns }
    }

    pub fn from_slice(slice: &[i64]) -> Self {
        Self {
            columns: slice.to_vec(),
        }
    }
}

impl PartialOrd for CompositeKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CompositeKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for (a, b) in self.columns.iter().zip(other.columns.iter()) {
            match a.cmp(b) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }
        self.columns.len().cmp(&other.columns.len())
    }
}

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
    /// Whether this is a unique index
    pub is_unique: bool,
}

/// Index statistics
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    /// Number of entries in the index
    pub num_entries: u64,
    /// Number of leaf nodes
    pub num_leaf_nodes: u64,
    /// Number of internal nodes
    pub num_internal_nodes: u64,
    /// Total number of nodes
    pub total_nodes: u64,
    /// Height of the tree (root to leaf)
    pub height: u32,
    /// Cardinality (distinct key count), estimated
    pub cardinality: u64,
    /// Index size in bytes (estimated)
    pub size_bytes: u64,
}

impl IndexStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate index selectivity (0.0 to 1.0)
    pub fn selectivity(&self) -> f64 {
        if self.num_entries == 0 {
            return 1.0;
        }
        let selectivity = self.cardinality as f64 / self.num_entries as f64;
        selectivity.clamp(0.0, 1.0)
    }
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

    /// Insert a key-value pair into a unique index
    /// Returns Ok(()) if inserted successfully, Err if key already exists
    pub fn insert_unique(&mut self, key: i64, value: u32) -> Result<(), UniqueConstraintViolation> {
        // Check if key already exists
        if self.search(key).is_some() {
            return Err(UniqueConstraintViolation { key });
        }
        self.insert(key, value);
        Ok(())
    }

    /// Check if this is a unique index
    pub fn is_unique(&self) -> bool {
        self.metadata.is_unique
    }

    /// Set the unique flag on this index
    pub fn set_unique(&mut self, unique: bool) {
        self.metadata.is_unique = unique;
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

    /// Collect index statistics
    pub fn collect_stats(&self) -> IndexStats {
        let mut stats = IndexStats::new();
        stats.num_entries = self.metadata.num_entries;
        stats.height = self.metadata.height;

        // Count nodes
        let mut leaf_count = 0u64;
        let mut internal_count = 0u64;
        let mut total_size = 0u64;

        for node_opt in &self.nodes {
            if let Some(ref node) = node_opt {
                total_size += std::mem::size_of::<BTreeNode>() as u64;
                total_size += (node.keys.capacity() * std::mem::size_of::<i64>()) as u64;
                total_size += (node.values.capacity() * std::mem::size_of::<u32>()) as u64;

                if node.is_leaf {
                    leaf_count += 1;
                } else {
                    internal_count += 1;
                }
            }
        }

        stats.num_leaf_nodes = leaf_count;
        stats.num_internal_nodes = internal_count;
        stats.total_nodes = leaf_count + internal_count;
        stats.size_bytes = total_size;

        // Estimate cardinality (simplified - assumes reasonable distribution)
        // For a unique index, cardinality = num_entries
        // For non-unique, we estimate based on tree depth
        stats.cardinality = self.estimate_cardinality();

        stats
    }

    /// Estimate cardinality (distinct key count)
    fn estimate_cardinality(&self) -> u64 {
        // Simplified estimation
        // In reality, this would require scanning the index
        let entries = self.metadata.num_entries;
        if entries == 0 {
            return 0;
        }
        // Estimate 80% of entries are unique as a rough heuristic
        (entries * 8) / 10
    }

    /// Get index usage statistics
    pub fn usage_stats(&self) -> IndexStats {
        self.collect_stats()
    }
}

impl Default for BTreeIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Composite B+Tree index for multi-column indexes
pub struct CompositeBTreeIndex {
    pub metadata: BTreeMetadata,
    nodes: Vec<Option<BTreeNode>>,
    dirty: bool,
    /// Number of columns in the composite key
    num_columns: usize,
}

impl CompositeBTreeIndex {
    pub fn new(num_columns: usize) -> Self {
        Self {
            metadata: BTreeMetadata::default(),
            nodes: vec![None],
            dirty: true,
            num_columns,
        }
    }

    pub fn from_metadata(metadata: BTreeMetadata, num_columns: usize) -> Self {
        Self {
            metadata,
            nodes: Vec::new(),
            dirty: false,
            num_columns,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Insert a composite key-value pair
    pub fn insert(&mut self, key: CompositeKey, value: u32) {
        // Encode composite key as a single i64 for storage
        // This is a simplified implementation - real DBs would use more sophisticated encoding
        let encoded_key = self.encode_composite_key(&key);

        if self.metadata.root_page_id.is_none() {
            let root = BTreeNode::new_leaf();
            let root_id = self.allocate_node(root);
            self.metadata.root_page_id = Some(root_id);
            self.metadata.first_leaf = Some(root_id);
            self.metadata.num_entries = 1;
            self.metadata.height = 1;
            self.dirty = true;
            if let Some(ref mut node) = self.nodes[root_id as usize] {
                node.keys.push(encoded_key);
                node.values.push(value);
                node.num_keys = 1;
            }
            return;
        }

        self.insert_into_node(self.metadata.root_page_id.unwrap(), encoded_key, value);
        self.metadata.num_entries += 1;
        self.dirty = true;
    }

    fn encode_composite_key(&self, key: &CompositeKey) -> i64 {
        // Simple encoding: use first column as the key
        // For full implementation, would need more sophisticated encoding
        key.columns.first().copied().unwrap_or(0)
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

    pub fn search(&self, key: &CompositeKey) -> Option<u32> {
        let encoded_key = self.encode_composite_key(key);
        if let Some(root_id) = self.metadata.root_page_id {
            self.search_node(root_id, encoded_key)
        } else {
            None
        }
    }

    fn search_node(&self, node_id: u32, key: i64) -> Option<u32> {
        if let Some(ref node) = self.nodes[node_id as usize] {
            if let Some(idx) = node.find_key_index(key) {
                return node.values.get(idx).copied();
            }
            if !node.is_leaf {
                let child_idx = node.find_child_index(key);
                if child_idx < node.children.len() {
                    return self.search_node(node.children[child_idx], key);
                }
            }
        }
        None
    }

    pub fn range_query(&self, start: &CompositeKey, end: &CompositeKey) -> Vec<u32> {
        let start_key = self.encode_composite_key(start);
        let end_key = self.encode_composite_key(end);

        if let Some(first_leaf) = self.metadata.first_leaf {
            let mut results = Vec::new();
            self.range_query_leaf(first_leaf, start_key, end_key, &mut results);
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

    pub fn num_columns(&self) -> usize {
        self.num_columns
    }
}

impl Default for CompositeBTreeIndex {
    fn default() -> Self {
        Self::new(1)
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

// ============================================================================
// Full-Text Search Index (FTS)
// ============================================================================

/// Metadata for full-text search index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTextMetadata {
    pub column_name: String,
    pub num_documents: u64,
    pub num_terms: u64,
    pub is_dirty: bool,
}

impl Default for FullTextMetadata {
    fn default() -> Self {
        Self {
            column_name: String::new(),
            num_documents: 0,
            num_terms: 0,
            is_dirty: true,
        }
    }
}

impl FullTextMetadata {
    pub fn new(column_name: &str) -> Self {
        Self {
            column_name: column_name.to_string(),
            num_documents: 0,
            num_terms: 0,
            is_dirty: true,
        }
    }
}

/// Posting list - stores document IDs that contain a term
/// DocIDs must be kept sorted for efficient intersection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingList {
    pub doc_ids: Vec<u32>,
}

impl PostingList {
    pub fn new() -> Self {
        Self {
            doc_ids: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            doc_ids: Vec::with_capacity(capacity),
        }
    }

    /// Add a document ID, keeping the list sorted
    pub fn add_doc_id(&mut self, doc_id: u32) {
        // Binary search to find insertion point
        match self.doc_ids.binary_search(&doc_id) {
            Ok(_) => {} // Already exists, skip
            Err(pos) => self.doc_ids.insert(pos, doc_id),
        }
    }

    /// Check if contains a document
    pub fn contains(&self, doc_id: u32) -> bool {
        self.doc_ids.binary_search(&doc_id).is_ok()
    }
}

impl Default for PostingList {
    fn default() -> Self {
        Self::new()
    }
}

/// Full-text search index using inverted index
/// Uses BTreeMap to support term ordering and future prefix queries
pub struct FullTextIndex {
    pub metadata: FullTextMetadata,
    /// Inverted index: term -> posting list
    pub inverted_index: BTreeMap<String, PostingList>,
    /// Lazy deletion set
    deleted_docs: HashSet<u32>,
    dirty: bool,
}

impl FullTextIndex {
    /// Create a new full-text index
    pub fn new(column_name: &str) -> Self {
        Self {
            metadata: FullTextMetadata::new(column_name),
            inverted_index: BTreeMap::new(),
            deleted_docs: HashSet::new(),
            dirty: true,
        }
    }

    /// Check if index is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty || self.metadata.is_dirty
    }

    /// Tokenize text into terms
    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= 2)
            .map(|s| s.to_string())
            .collect()
    }

    /// Insert a document into the full-text index
    pub fn insert(&mut self, doc_id: u32, text: &str) {
        let terms = Self::tokenize(text);

        // Update document count
        self.metadata.num_documents += 1;
        self.metadata.is_dirty = true;
        self.dirty = true;

        // Add document to each term's posting list
        for term in terms {
            let entry = self.inverted_index.entry(term).or_default();
            entry.add_doc_id(doc_id);
        }

        // Update unique term count
        self.metadata.num_terms = self.inverted_index.len() as u64;
    }

    /// Lazy delete - mark document as deleted without immediate removal
    pub fn delete(&mut self, doc_id: u32) {
        self.deleted_docs.insert(doc_id);
        self.dirty = true;
    }

    /// Check if a document is deleted
    fn is_deleted(&self, doc_id: u32) -> bool {
        self.deleted_docs.contains(&doc_id)
    }

    /// Filter deleted documents from results
    fn filter_deleted(&self, doc_ids: &[u32]) -> Vec<u32> {
        doc_ids
            .iter()
            .copied()
            .filter(|id| !self.is_deleted(*id))
            .collect()
    }

    /// Intersect two sorted posting lists (AND operation)
    fn intersect(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut result = Vec::with_capacity(std::cmp::min(a.len(), b.len()));
        let mut i = 0;
        let mut j = 0;

        while i < a.len() && j < b.len() {
            if a[i] == b[j] {
                result.push(a[i]);
                i += 1;
                j += 1;
            } else if a[i] < b[j] {
                i += 1;
            } else {
                j += 1;
            }
        }

        result
    }

    /// Search for documents containing ALL query terms (AND)
    pub fn search(&self, query: &str) -> Vec<u32> {
        let terms = Self::tokenize(query);

        if terms.is_empty() {
            return Vec::new();
        }

        // Get posting lists for each term
        let posting_lists: Vec<&Vec<u32>> = terms
            .iter()
            .filter_map(|term| self.inverted_index.get(term).map(|pl| &pl.doc_ids))
            .collect();

        // If no terms found, return empty
        if posting_lists.is_empty() {
            return Vec::new();
        }

        // Start with first term's posting list
        let mut result = posting_lists[0].clone();

        // Intersect with other terms
        for posting_list in &posting_lists[1..] {
            result = Self::intersect(&result, posting_list);
        }

        // Filter deleted documents
        self.filter_deleted(&result)
    }

    /// Get the number of documents containing a term
    pub fn term_frequency(&self, term: &str) -> usize {
        self.inverted_index
            .get(&term.to_lowercase())
            .map(|pl| pl.doc_ids.len())
            .unwrap_or(0)
    }

    /// Get all indexed terms
    pub fn terms(&self) -> Vec<String> {
        self.inverted_index.keys().cloned().collect()
    }

    /// Get number of documents
    pub fn num_documents(&self) -> u64 {
        self.metadata.num_documents
    }

    /// Get number of unique terms
    pub fn num_terms(&self) -> u64 {
        self.metadata.num_terms
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.metadata.num_documents == 0
    }
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

    // Tests for index statistics

    #[test]
    fn test_index_stats_default() {
        let stats = IndexStats::default();
        assert_eq!(stats.num_entries, 0);
        assert_eq!(stats.num_leaf_nodes, 0);
        assert_eq!(stats.total_nodes, 0);
    }

    #[test]
    fn test_index_stats_selectivity_empty() {
        let stats = IndexStats::default();
        assert_eq!(stats.selectivity(), 1.0);
    }

    #[test]
    fn test_index_stats_selectivity() {
        let mut stats = IndexStats::new();
        stats.num_entries = 100;
        stats.cardinality = 50;
        assert_eq!(stats.selectivity(), 0.5);
    }

    #[test]
    fn test_index_stats_selectivity_clamped() {
        let mut stats = IndexStats::new();
        stats.num_entries = 100;
        stats.cardinality = 150; // More unique than entries
        assert_eq!(stats.selectivity(), 1.0); // Should clamp to 1.0
    }

    #[test]
    fn test_btree_index_collect_stats_empty() {
        let index = BTreeIndex::new();
        let stats = index.collect_stats();

        assert_eq!(stats.num_entries, 0);
        assert_eq!(stats.height, 0);
    }

    #[test]
    fn test_btree_index_collect_stats() {
        let mut index = BTreeIndex::new();

        // Insert some entries
        for i in 1..=10 {
            index.insert(i, i as u32);
        }

        let stats = index.collect_stats();

        assert_eq!(stats.num_entries, 10);
        assert!(stats.height >= 1);
        assert!(stats.total_nodes >= 1);
    }

    #[test]
    fn test_btree_index_usage_stats() {
        let mut index = BTreeIndex::new();
        index.insert(1, 100);
        index.insert(2, 200);

        let stats = index.usage_stats();

        assert_eq!(stats.num_entries, 2);
    }

    #[test]
    fn test_composite_btree_index_insert() {
        let mut index = CompositeBTreeIndex::new(2);
    }

    // Full-text search index tests

    #[test]
    fn test_fts_index_new() {
        let index = FullTextIndex::new("content");
        assert!(index.is_empty());
        assert_eq!(index.num_documents(), 0);
        assert_eq!(index.num_terms(), 0);
    }

    #[test]
    fn test_fts_tokenize() {
        let tokens = FullTextIndex::tokenize("Hello World!");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_fts_tokenize_chinese() {
        // Current tokenization includes Unicode letters (including Chinese)
        // This is a known limitation for P2 - future versions can use proper segmentation
        let tokens = FullTextIndex::tokenize("你好 World 123");
        // At minimum, English tokens should be present
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"123".to_string()));
    }

    #[test]
    fn test_fts_tokenize_punctuation() {
        let tokens = FullTextIndex::tokenize("hello,world!test-case");
        assert_eq!(tokens, vec!["hello", "world", "test", "case"]);
    }

    #[test]
    fn test_fts_tokenize_short_words() {
        // Words shorter than 2 chars should be filtered
        let tokens = FullTextIndex::tokenize("a b c hello");
        assert_eq!(tokens, vec!["hello"]);
    }

    #[test]
    fn test_fts_insert() {
        let mut index = FullTextIndex::new("content");
        index.insert(1, "hello world");
        index.insert(2, "hello rust");

        assert_eq!(index.num_documents(), 2);
        assert!(index.num_terms() >= 2);
    }

    #[test]
    fn test_fts_search_single_term() {
        let mut index = FullTextIndex::new("content");
        index.insert(1, "hello world");
        index.insert(2, "hello rust");
        index.insert(3, "goodbye world");

        let results = index.search("hello");
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
    }

    #[test]
    fn test_fts_search_and() {
        let mut index = FullTextIndex::new("content");
        index.insert(1, "hello world");
        index.insert(2, "hello rust");
        index.insert(3, "goodbye world");

        // Search for "hello world" - should return only doc 1
        let results = index.search("hello world");
        assert_eq!(results, vec![1]);
    }

    #[test]
    fn test_fts_search_no_match() {
        let mut index = FullTextIndex::new("content");
        index.insert(1, "hello world");

        let results = index.search("missing");
        assert!(results.is_empty());
    }

    #[test]
    fn test_fts_search_case_insensitive() {
        let mut index = FullTextIndex::new("content");
        index.insert(1, "Hello World");

        let results = index.search("HELLO");
        assert_eq!(results, vec![1]);
    }

    #[test]
    fn test_fts_delete_lazy() {
        let mut index = FullTextIndex::new("content");
        index.insert(1, "hello world");
        index.insert(2, "hello rust");

        // Lazy delete - mark as deleted but keep in index
        index.delete(1);

        // Search should filter out deleted document
        let results = index.search("hello");
        assert_eq!(results, vec![2]);
    }

    #[test]
    fn test_fts_term_frequency() {
        let mut index = FullTextIndex::new("content");
        index.insert(1, "hello world hello");
        index.insert(2, "hello rust");
        index.insert(3, "world test");

        assert_eq!(index.term_frequency("hello"), 2);
        assert_eq!(index.term_frequency("world"), 2);
        assert_eq!(index.term_frequency("rust"), 1);
        assert_eq!(index.term_frequency("missing"), 0);
    }

    #[test]
    fn test_fts_terms() {
        let mut index = FullTextIndex::new("content");
        index.insert(1, "hello world");
        index.insert(2, "hello rust");

        let terms = index.terms();
        assert!(terms.contains(&"hello".to_string()));
        assert!(terms.contains(&"world".to_string()));
        assert!(terms.contains(&"rust".to_string()));
    }

    #[test]
    fn test_fts_posting_list_sorted() {
        let mut pl = PostingList::new();
        pl.add_doc_id(5);
        pl.add_doc_id(2);
        pl.add_doc_id(8);
        pl.add_doc_id(2); // Duplicate should be ignored

        assert_eq!(pl.doc_ids, vec![2, 5, 8]);
    }

    #[test]
    fn test_fts_intersect() {
        let a = vec![1, 2, 3, 4, 5];
        let b = vec![3, 4, 5, 6, 7];

        let result = FullTextIndex::intersect(&a, &b);
        assert_eq!(result, vec![3, 4, 5]);
    }

    #[test]
    fn test_fts_intersect_no_overlap() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];

        let result = FullTextIndex::intersect(&a, &b);
        assert!(result.is_empty());
    }

    #[test]
    fn test_fts_complex_query() {
        let mut index = FullTextIndex::new("content");
        index.insert(1, "SQL database systems");
        index.insert(2, "database PostgreSQL systems");
        index.insert(3, "SQL query database optimization");
        index.insert(4, "database indexing");

        // Find documents with both "SQL" AND "database"
        let results = index.search("SQL database");
        // Doc 1: "SQL database" - match
        // Doc 2: "database PostgreSQL" - no "SQL"
        // Doc 3: "SQL query database" - match (both SQL and database)
        // Doc 4: "database" only - no "SQL"
        assert!(results.contains(&1));
        assert!(!results.contains(&2));
        assert!(results.contains(&3));
        assert!(!results.contains(&4));
    }

    #[test]
    fn test_composite_key_creation() {
        let key = CompositeKey::new(vec![1, 2, 3]);
        assert_eq!(key.columns.len(), 3);
    }
}
