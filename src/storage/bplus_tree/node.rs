//! B+ Tree Node structures

/// B+ Tree Node types
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Internal(InternalNode),
    Leaf(LeafNode),
}

/// Internal node (for branching in the tree)
#[derive(Debug, Clone, PartialEq)]
pub struct InternalNode {
    pub keys: Vec<u64>,      // Separating keys
    pub children: Vec<u32>,   // Child page IDs
    pub is_root: bool,
}

impl InternalNode {
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            children: Vec::new(),
            is_root: false,
        }
    }
    
    pub fn is_leaf(&self) -> bool {
        false
    }
}

/// Leaf node (contains actual key-value pairs)
#[derive(Debug, Clone, PartialEq)]
pub struct LeafNode {
    pub keys: Vec<u64>,
    pub values: Vec<u32>,    // Record IDs or row pointers
    pub next: Option<u32>,   // Linked list for range scans
    pub is_root: bool,
}

impl LeafNode {
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            next: None,
            is_root: false,
        }
    }
    
    pub fn is_leaf(&self) -> bool {
        true
    }
}

/// Maximum keys per node (adjust based on page size)
pub const MAX_KEYS: usize = 100;
pub const MIN_KEYS: usize = 50;
