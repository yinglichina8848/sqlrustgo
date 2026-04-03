//! Index information for catalog tables

use serde::{Deserialize, Serialize};

/// Index type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    /// B+ Tree index (default)
    BTree,
    /// Hash index
    Hash,
    /// Full-text search index
    FullText,
}

/// Index information for a table index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    /// Index name (unique within schema)
    pub name: String,
    /// Table name this index belongs to
    pub table_name: String,
    /// Column names that make up the index
    pub columns: Vec<String>,
    /// Index type
    pub index_type: IndexType,
    /// Whether the index is unique
    pub is_unique: bool,
    /// Whether the index is the primary key index
    pub is_primary_key: bool,
}

impl IndexInfo {
    /// Create a new index info
    pub fn new(
        name: impl Into<String>,
        table_name: impl Into<String>,
        columns: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            table_name: table_name.into(),
            columns,
            index_type: IndexType::BTree,
            is_unique: false,
            is_primary_key: false,
        }
    }

    /// Set as primary key index
    pub fn primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self.is_unique = true;
        self
    }

    /// Set as unique index
    pub fn unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    /// Set the index type
    pub fn with_type(mut self, index_type: IndexType) -> Self {
        self.index_type = index_type;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_creation() {
        let idx = IndexInfo::new("idx_name", "users", vec!["email".to_string()]);
        assert_eq!(idx.name, "idx_name");
        assert_eq!(idx.table_name, "users");
        assert_eq!(idx.columns, vec!["email"]);
        assert!(!idx.is_unique);
        assert!(!idx.is_primary_key);
    }

    #[test]
    fn test_primary_key_index() {
        let idx = IndexInfo::new("pk_users", "users", vec!["id".to_string()]).primary_key();
        assert!(idx.is_primary_key);
        assert!(idx.is_unique);
    }

    #[test]
    fn test_unique_index() {
        let idx = IndexInfo::new("idx_email", "users", vec!["email".to_string()]).unique();
        assert!(idx.is_unique);
        assert!(!idx.is_primary_key);
    }

    #[test]
    fn test_index_type_btree() {
        let idx = IndexInfo::new("idx", "t", vec!["c".to_string()]).with_type(IndexType::BTree);
        assert_eq!(idx.index_type, IndexType::BTree);
    }

    #[test]
    fn test_index_type_hash() {
        let idx = IndexInfo::new("idx", "t", vec!["c".to_string()]).with_type(IndexType::Hash);
        assert_eq!(idx.index_type, IndexType::Hash);
    }

    #[test]
    fn test_index_type_fulltext() {
        let idx = IndexInfo::new("idx", "t", vec!["c".to_string()]).with_type(IndexType::FullText);
        assert_eq!(idx.index_type, IndexType::FullText);
    }

    #[test]
    fn test_index_with_multiple_columns() {
        let idx = IndexInfo::new(
            "idx",
            "users",
            vec!["last_name".to_string(), "first_name".to_string()],
        );
        assert_eq!(idx.columns.len(), 2);
        assert_eq!(idx.columns[0], "last_name");
        assert_eq!(idx.columns[1], "first_name");
    }

    #[test]
    fn test_index_type_enum() {
        assert_eq!(format!("{:?}", IndexType::BTree), "BTree");
        assert_eq!(format!("{:?}", IndexType::Hash), "Hash");
        assert_eq!(format!("{:?}", IndexType::FullText), "FullText");
    }

    #[test]
    fn test_index_clone() {
        let idx1 = IndexInfo::new("idx", "t", vec!["c".to_string()]);
        let idx2 = idx1.clone();
        assert_eq!(idx1.name, idx2.name);
        assert_eq!(idx1.table_name, idx2.table_name);
    }

    #[test]
    fn test_index_debug() {
        let idx = IndexInfo::new("idx", "t", vec!["c".to_string()]);
        let debug = format!("{:?}", idx);
        assert!(debug.contains("idx"));
        assert!(debug.contains("t"));
    }
}
