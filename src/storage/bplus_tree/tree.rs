//! B+ Tree implementation

/// B+ Tree index
#[derive(Debug)]
pub struct BPlusTree {
    entries: Vec<(u64, u32)>, // Simplified: just store entries
}

impl BPlusTree {
    /// Create a new B+ Tree
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Insert a key-value pair
    pub fn insert(&mut self, key: u64, value: u32) {
        self.entries.push((key, value));
        self.entries.sort_by_key(|(k, _)| *k);
    }

    /// Search for a key
    pub fn search(&self, key: u64) -> Option<u32> {
        self.entries
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
    }

    /// Range query
    pub fn range_query(&self, start: u64, end: u64) -> Vec<u32> {
        self.entries
            .iter()
            .filter(|(k, _)| *k >= start && *k < end)
            .map(|(_, v)| *v)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bplus_tree_insert() {
        let mut tree = BPlusTree::new();
        tree.insert(10, 100);
        tree.insert(20, 200);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_bplus_tree_search() {
        let mut tree = BPlusTree::new();
        tree.insert(10, 100);
        tree.insert(20, 200);
        assert_eq!(tree.search(10), Some(100));
        assert_eq!(tree.search(99), None);
    }
}
