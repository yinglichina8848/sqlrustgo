//! Secondary index implementation for clustered tables.
//!
//! This module provides secondary index support that stores references to
//! cluster keys instead of row IDs. This enables efficient coordination
//! between secondary indexes and the clustered index.
//!
//! # Key Design
//!
//! - Secondary indexes store `(indexed_value, cluster_key)` pairs
//! - When searching via secondary index, we first get cluster_keys, then look up in clustered index
//! - Index-only scans are supported when all required columns are in the secondary index

use crate::row_format::types::ClusterKey;
use serde::{Deserialize, Serialize};
use sqlrustgo_types::Value;
use std::collections::BTreeMap;
use thiserror::Error;

/// Unique constraint violation error for secondary indexes
#[derive(Debug, Clone, Error)]
#[error("unique constraint violation on secondary index '{index_name}': key already exists")]
pub struct SecondaryIndexUniqueViolation {
    /// Name of the index that caused the violation
    pub index_name: String,
    /// The duplicate key that caused the violation
    pub key: Value,
}

/// Metadata for a secondary index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryIndexMetadata {
    /// Name of the index
    pub name: String,
    /// Name of the table this index belongs to
    pub table_name: String,
    /// Column names that are indexed (in order for composite indexes)
    pub columns: Vec<String>,
    /// Whether this is a unique index
    pub is_unique: bool,
    /// Number of entries in the index
    pub num_entries: u64,
}

impl SecondaryIndexMetadata {
    /// Create new metadata for a secondary index
    pub fn new(name: String, table_name: String, columns: Vec<String>, is_unique: bool) -> Self {
        Self {
            name,
            table_name,
            columns,
            is_unique,
            num_entries: 0,
        }
    }

    /// Check if this index covers a query (all required columns are indexed)
    pub fn covers_query(&self, required_columns: &[String]) -> bool {
        // For a query to be covered, all required columns must be in the index
        // The index columns are a superset of the required columns
        required_columns
            .iter()
            .all(|col| self.columns.contains(col))
    }

    /// Get the number of columns in this index
    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }
}

/// A secondary index entry mapping indexed value to cluster key.
/// Since we support composite indexes, we use a Vec<Value> for the key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecondaryIndexKey {
    /// Column values forming the index key
    pub values: Vec<Value>,
}

impl SecondaryIndexKey {
    /// Create a new secondary index key from values
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    /// Create from a single value (for single-column indexes)
    pub fn from_single(value: Value) -> Self {
        Self {
            values: vec![value],
        }
    }

    /// Check if this key is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Compare two secondary index keys with total order.
    /// Null is considered less than any other value.
    fn compare_values(lhs: &Value, rhs: &Value) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        fn type_order(v: &Value) -> u8 {
            match v {
                Value::Null => 0,
                Value::Boolean(_) => 1,
                Value::Integer(_) => 2,
                Value::Float(_) => 3,
                Value::Text(_) => 4,
                Value::Blob(_) => 5,
            }
        }

        match (lhs, rhs) {
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Null, _) => Ordering::Less,
            (_, Value::Null) => Ordering::Greater,
            _ => {
                let type_cmp = type_order(lhs).cmp(&type_order(rhs));
                if type_cmp != Ordering::Equal {
                    return type_cmp;
                }
                match lhs.partial_cmp(rhs) {
                    Some(cmp) => cmp,
                    None => Ordering::Equal,
                }
            }
        }
    }
}

impl Ord for SecondaryIndexKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for (lhs, rhs) in self.values.iter().zip(other.values.iter()) {
            let cmp = Self::compare_values(lhs, rhs);
            if cmp != std::cmp::Ordering::Equal {
                return cmp;
            }
        }
        self.values.len().cmp(&other.values.len())
    }
}

impl PartialOrd for SecondaryIndexKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Secondary index that stores references to cluster keys.
/// This enables efficient coordination between secondary indexes and clustered indexes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryIndex {
    /// Metadata about this index
    pub metadata: SecondaryIndexMetadata,
    /// The actual index data: key -> cluster key(s)
    /// For unique indexes, each key maps to at most one cluster key
    /// For non-unique indexes, each key can map to multiple cluster keys
    data: BTreeMap<SecondaryIndexKey, Vec<ClusterKey>>,
    /// Dirty flag for persistence
    dirty: bool,
}

impl SecondaryIndex {
    /// Create a new secondary index
    pub fn new(name: String, table_name: String, columns: Vec<String>, is_unique: bool) -> Self {
        Self {
            metadata: SecondaryIndexMetadata::new(name, table_name, columns, is_unique),
            data: BTreeMap::new(),
            dirty: true,
        }
    }

    /// Create from existing metadata
    pub fn from_metadata(metadata: SecondaryIndexMetadata) -> Self {
        Self {
            metadata,
            data: BTreeMap::new(),
            dirty: false,
        }
    }

    /// Check if the index is dirty (needs to be persisted)
    pub fn is_dirty(&self) -> bool {
        self.dirty || self.metadata.num_entries == 0
    }

    /// Mark the index as dirty
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Insert a value-key pair into the secondary index.
    /// For unique indexes, returns error if key already exists.
    pub fn insert(
        &mut self,
        indexed_value: Value,
        cluster_key: ClusterKey,
    ) -> Result<(), SecondaryIndexUniqueViolation> {
        let key = SecondaryIndexKey::from_single(indexed_value.clone());

        // For unique indexes, check if key already exists
        if self.metadata.is_unique {
            if let Some(existing) = self.data.get(&key) {
                if !existing.is_empty() {
                    return Err(SecondaryIndexUniqueViolation {
                        index_name: self.metadata.name.clone(),
                        key: indexed_value,
                    });
                }
            }
        }

        // Insert the cluster key
        let entries = self.data.entry(key).or_default();
        entries.push(cluster_key);
        self.metadata.num_entries += 1;
        self.dirty = true;

        Ok(())
    }

    /// Insert with a composite key (for multi-column indexes)
    pub fn insert_composite(
        &mut self,
        indexed_values: Vec<Value>,
        cluster_key: ClusterKey,
    ) -> Result<(), SecondaryIndexUniqueViolation> {
        let key = SecondaryIndexKey::new(indexed_values.clone());

        // For unique indexes, check if key already exists
        if self.metadata.is_unique {
            if let Some(existing) = self.data.get(&key) {
                if !existing.is_empty() {
                    return Err(SecondaryIndexUniqueViolation {
                        index_name: self.metadata.name.clone(),
                        key: indexed_values.into_iter().next().unwrap_or(Value::Null),
                    });
                }
            }
        }

        // Insert the cluster key
        let entries = self.data.entry(key).or_default();
        entries.push(cluster_key);
        self.metadata.num_entries += 1;
        self.dirty = true;

        Ok(())
    }

    /// Search for a single value, returning all matching cluster keys.
    pub fn search(&self, value: &Value) -> Vec<ClusterKey> {
        let key = SecondaryIndexKey::from_single(value.clone());
        self.data.get(&key).cloned().unwrap_or_default()
    }

    /// Search for a composite key
    pub fn search_composite(&self, values: &[Value]) -> Vec<ClusterKey> {
        let key = SecondaryIndexKey::new(values.to_vec());
        self.data.get(&key).cloned().unwrap_or_default()
    }

    /// Search for a unique index, returning at most one cluster key.
    /// Returns None if the key doesn't exist.
    pub fn search_unique(&self, value: &Value) -> Option<ClusterKey> {
        if !self.metadata.is_unique {
            panic!("search_unique called on non-unique index");
        }
        let key = SecondaryIndexKey::from_single(value.clone());
        self.data.get(&key).and_then(|v| v.first().cloned())
    }

    /// Range query for single-column indexes.
    /// Returns all cluster keys where start <= key < end.
    pub fn range_query(&self, start: &Value, end: &Value) -> Vec<ClusterKey> {
        let start_key = SecondaryIndexKey::from_single(start.clone());
        let end_key = SecondaryIndexKey::from_single(end.clone());

        self.data
            .range(start_key..end_key)
            .flat_map(|(_, cluster_keys)| cluster_keys.clone())
            .collect()
    }

    /// Delete entries by cluster key.
    /// Returns the number of entries removed.
    pub fn delete(&mut self, cluster_key: &ClusterKey) -> usize {
        let mut removed = 0;

        // Iterate through all entries and remove matching cluster keys
        for (_key, cluster_keys) in self.data.iter_mut() {
            let original_len = cluster_keys.len();
            cluster_keys.retain(|ck| ck != cluster_key);
            removed += original_len - cluster_keys.len();
        }

        // Clean up empty entries
        self.data.retain(|_, v| !v.is_empty());

        if removed > 0 {
            self.metadata.num_entries = self.metadata.num_entries.saturating_sub(removed as u64);
            self.dirty = true;
        }

        removed
    }

    /// Delete entries by indexed value and cluster key.
    /// Returns true if an entry was removed.
    pub fn delete_by_value(&mut self, value: &Value, cluster_key: &ClusterKey) -> bool {
        let key = SecondaryIndexKey::from_single(value.clone());

        // First check if the key exists and get original length
        let original_len = match self.data.get(&key) {
            Some(cluster_keys) => cluster_keys.len(),
            None => return false,
        };

        // Now do the mutation
        let cluster_keys = self.data.get_mut(&key).unwrap();
        cluster_keys.retain(|ck| ck != cluster_key);
        let new_len = cluster_keys.len();
        let removed = original_len - new_len;

        if new_len == 0 {
            self.data.remove(&key);
        }

        if removed > 0 {
            self.metadata.num_entries = self.metadata.num_entries.saturating_sub(removed as u64);
            self.dirty = true;
            return true;
        }

        false
    }

    /// Check if this index can be used for an index-only scan.
    /// Returns true if all required columns are covered by this index.
    pub fn covers_query(&self, required_columns: &[String]) -> bool {
        self.metadata.covers_query(required_columns)
    }

    /// Get the number of entries in the index
    pub fn len(&self) -> u64 {
        self.metadata.num_entries
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.metadata.num_entries == 0
    }

    /// Get all entries (for debugging/testing)
    pub fn iter(&self) -> impl Iterator<Item = (&SecondaryIndexKey, &[ClusterKey])> {
        self.data.iter().map(|(k, v)| (k, v.as_slice()))
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.data.clear();
        self.metadata.num_entries = 0;
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secondary_index_insert_and_search() {
        let mut index = SecondaryIndex::new(
            "idx_name".to_string(),
            "users".to_string(),
            vec!["name".to_string()],
            false,
        );

        let ck1 = ClusterKey::PrimaryKey(Value::Integer(1));
        let ck2 = ClusterKey::PrimaryKey(Value::Integer(2));

        index
            .insert(Value::Text("Alice".to_string()), ck1.clone())
            .unwrap();
        index
            .insert(Value::Text("Bob".to_string()), ck2.clone())
            .unwrap();

        // Search for Alice
        let results = index.search(&Value::Text("Alice".to_string()));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ck1);

        // Search for Bob
        let results = index.search(&Value::Text("Bob".to_string()));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ck2);

        // Search for non-existent
        let results = index.search(&Value::Text("Charlie".to_string()));
        assert!(results.is_empty());
    }

    #[test]
    fn test_secondary_index_unique_constraint() {
        let mut index = SecondaryIndex::new(
            "idx_email".to_string(),
            "users".to_string(),
            vec!["email".to_string()],
            true, // unique index
        );

        let ck1 = ClusterKey::PrimaryKey(Value::Integer(1));
        let ck2 = ClusterKey::PrimaryKey(Value::Integer(2));

        // First insert should succeed
        index
            .insert(Value::Text("alice@example.com".to_string()), ck1.clone())
            .unwrap();

        // Second insert with same key should fail
        let result = index.insert(Value::Text("alice@example.com".to_string()), ck2.clone());
        assert!(result.is_err());
    }

    #[test]
    fn test_secondary_index_non_unique_allows_duplicates() {
        let mut index = SecondaryIndex::new(
            "idx_name".to_string(),
            "users".to_string(),
            vec!["name".to_string()],
            false, // non-unique index
        );

        let ck1 = ClusterKey::PrimaryKey(Value::Integer(1));
        let ck2 = ClusterKey::PrimaryKey(Value::Integer(2));
        let ck3 = ClusterKey::PrimaryKey(Value::Integer(3));

        // Multiple "Alice" entries
        index.insert(Value::Text("Alice".to_string()), ck1).unwrap();
        index.insert(Value::Text("Alice".to_string()), ck2).unwrap();
        index.insert(Value::Text("Alice".to_string()), ck3).unwrap();

        let results = index.search(&Value::Text("Alice".to_string()));
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_secondary_index_delete() {
        let mut index = SecondaryIndex::new(
            "idx_name".to_string(),
            "users".to_string(),
            vec!["name".to_string()],
            false,
        );

        let ck1 = ClusterKey::PrimaryKey(Value::Integer(1));
        let ck2 = ClusterKey::PrimaryKey(Value::Integer(2));

        index
            .insert(Value::Text("Alice".to_string()), ck1.clone())
            .unwrap();
        index
            .insert(Value::Text("Alice".to_string()), ck2.clone())
            .unwrap();

        // Delete one cluster key
        let removed = index.delete(&ck1);
        assert_eq!(removed, 1);

        // Alice should still have one entry
        let results = index.search(&Value::Text("Alice".to_string()));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ck2);
    }

    #[test]
    fn test_secondary_index_delete_by_value() {
        let mut index = SecondaryIndex::new(
            "idx_name".to_string(),
            "users".to_string(),
            vec!["name".to_string()],
            false,
        );

        let ck1 = ClusterKey::PrimaryKey(Value::Integer(1));
        let ck2 = ClusterKey::PrimaryKey(Value::Integer(2));

        index
            .insert(Value::Text("Alice".to_string()), ck1.clone())
            .unwrap();
        index
            .insert(Value::Text("Alice".to_string()), ck2.clone())
            .unwrap();

        // Delete Alice's entry with ck1
        let deleted = index.delete_by_value(&Value::Text("Alice".to_string()), &ck1);
        assert!(deleted);

        // Alice should still have one entry
        let results = index.search(&Value::Text("Alice".to_string()));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ck2);
    }

    #[test]
    fn test_secondary_index_range_query() {
        let mut index = SecondaryIndex::new(
            "idx_age".to_string(),
            "users".to_string(),
            vec!["age".to_string()],
            false,
        );

        // Insert ages 10, 20, 30, 40, 50
        for i in 1..=5 {
            let ck = ClusterKey::PrimaryKey(Value::Integer(i));
            index.insert(Value::Integer(i * 10), ck).unwrap();
        }

        // Range query: 15 to 45 should get 20, 30, 40
        let results = index.range_query(&Value::Integer(15), &Value::Integer(45));
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_secondary_index_covers_query() {
        let index = SecondaryIndex::new(
            "idx_name_age".to_string(),
            "users".to_string(),
            vec!["name".to_string(), "age".to_string()],
            false,
        );

        // Should cover queries needing name or age alone
        assert!(index.covers_query(&["name".to_string()]));
        assert!(index.covers_query(&["age".to_string()]));
        assert!(index.covers_query(&["name".to_string(), "age".to_string()]));

        // Should not cover queries needing other columns
        assert!(!index.covers_query(&["email".to_string()]));
        assert!(!index.covers_query(&["name".to_string(), "email".to_string()]));
    }

    #[test]
    fn test_secondary_index_composite_key() {
        let mut index = SecondaryIndex::new(
            "idx_name_age".to_string(),
            "users".to_string(),
            vec!["name".to_string(), "age".to_string()],
            true,
        );

        let ck1 = ClusterKey::PrimaryKey(Value::Integer(1));
        let ck2 = ClusterKey::PrimaryKey(Value::Integer(2));

        // Insert composite keys
        index
            .insert_composite(
                vec![Value::Text("Alice".to_string()), Value::Integer(30)],
                ck1.clone(),
            )
            .unwrap();
        index
            .insert_composite(
                vec![Value::Text("Bob".to_string()), Value::Integer(25)],
                ck2.clone(),
            )
            .unwrap();

        // Search for Alice, 30
        let results =
            index.search_composite(&[Value::Text("Alice".to_string()), Value::Integer(30)]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ck1);
    }

    #[test]
    fn test_secondary_index_len_and_empty() {
        let mut index = SecondaryIndex::new(
            "idx_name".to_string(),
            "users".to_string(),
            vec!["name".to_string()],
            false,
        );

        assert!(index.is_empty());
        assert_eq!(index.len(), 0);

        let ck = ClusterKey::PrimaryKey(Value::Integer(1));
        index.insert(Value::Text("Alice".to_string()), ck).unwrap();

        assert!(!index.is_empty());
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_secondary_index_clear() {
        let mut index = SecondaryIndex::new(
            "idx_name".to_string(),
            "users".to_string(),
            vec!["name".to_string()],
            false,
        );

        let ck = ClusterKey::PrimaryKey(Value::Integer(1));
        index.insert(Value::Text("Alice".to_string()), ck).unwrap();

        assert_eq!(index.len(), 1);

        index.clear();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_secondary_index_search_unique() {
        let mut index = SecondaryIndex::new(
            "idx_email".to_string(),
            "users".to_string(),
            vec!["email".to_string()],
            true,
        );

        let ck = ClusterKey::PrimaryKey(Value::Integer(1));
        index
            .insert(Value::Text("alice@example.com".to_string()), ck.clone())
            .unwrap();

        // search_unique should return the single entry
        let result = index.search_unique(&Value::Text("alice@example.com".to_string()));
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ck);

        // Non-existent should return None
        let result = index.search_unique(&Value::Text("bob@example.com".to_string()));
        assert!(result.is_none());
    }

    #[test]
    fn test_secondary_index_key_ordering() {
        use std::cmp::Ordering;

        let key1 = SecondaryIndexKey::from_single(Value::Integer(1));
        let key2 = SecondaryIndexKey::from_single(Value::Integer(2));
        let key3 = SecondaryIndexKey::from_single(Value::Integer(1));

        assert_eq!(key1.cmp(&key2), Ordering::Less);
        assert_eq!(key2.cmp(&key1), Ordering::Greater);
        assert_eq!(key1.cmp(&key3), Ordering::Equal);
    }

    #[test]
    fn test_secondary_index_null_handling() {
        let mut index = SecondaryIndex::new(
            "idx_name".to_string(),
            "users".to_string(),
            vec!["name".to_string()],
            false,
        );

        let ck = ClusterKey::PrimaryKey(Value::Integer(1));

        // Insert null value
        index.insert(Value::Null, ck.clone()).unwrap();

        // Search for null
        let results = index.search(&Value::Null);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ck);
    }

    #[test]
    fn test_secondary_index_metadata() {
        let metadata = SecondaryIndexMetadata::new(
            "idx_name".to_string(),
            "users".to_string(),
            vec!["name".to_string(), "email".to_string()],
            true,
        );

        assert_eq!(metadata.name, "idx_name");
        assert_eq!(metadata.table_name, "users");
        assert_eq!(metadata.columns.len(), 2);
        assert!(metadata.is_unique);
        assert_eq!(metadata.num_columns(), 2);

        // Test covers_query
        assert!(metadata.covers_query(&["name".to_string()]));
        assert!(metadata.covers_query(&["email".to_string()]));
        assert!(metadata.covers_query(&["name".to_string(), "email".to_string()]));
        assert!(!metadata.covers_query(&["phone".to_string()]));
    }
}
