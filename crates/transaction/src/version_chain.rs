//! Version Chain Map implementation for MVCC storage
//!
//! This module provides append-only version chain storage.
//! Each key maintains a chain of RowVersions (newest-first).

use crate::mvcc::{RowVersion, Snapshot, TxId};
use std::collections::HashMap;
use std::sync::RwLock;

/// VersionChainMap provides append-only version chain storage.
/// Each key maps to a vector of RowVersions, ordered from oldest to newest.
pub struct VersionChainMap {
    chains: RwLock<HashMap<Vec<u8>, Vec<RowVersion>>>,
}

impl VersionChainMap {
    /// Creates a new empty VersionChainMap
    pub fn new() -> Self {
        Self {
            chains: RwLock::new(HashMap::new()),
        }
    }

    /// Finds the visible version for a key given a snapshot.
    /// Searches from newest to oldest, returns first visible version.
    /// Returns None if the key is deleted (empty value) or not found.
    pub fn find_visible(&self, key: &[u8], snapshot: &Snapshot) -> Option<Vec<u8>> {
        let chains = self.chains.read().unwrap();

        if let Some(versions) = chains.get(key) {
            for version in versions.iter().rev() {
                if version.is_visible(snapshot) {
                    if version.value.is_empty() {
                        return None;
                    }
                    return Some(version.value.clone());
                }
            }
        }
        None
    }

    /// Appends a new version to the chain for the given key.
    pub fn append(&mut self, key: Vec<u8>, version: RowVersion) {
        let mut chains = self.chains.write().unwrap();
        chains.entry(key).or_insert_with(Vec::new).push(version);
    }

    /// Gets a clone of the version chain for a key.
    #[allow(dead_code)]
    pub fn get_chain(&self, key: &[u8]) -> Option<Vec<RowVersion>> {
        let chains = self.chains.read().unwrap();
        chains.get(key).cloned()
    }

    /// Commits all uncommitted versions created by the given transaction.
    /// Updates created_commit_ts for all versions matching tx_id.
    pub fn commit_versions(&mut self, tx_id: TxId, commit_ts: u64) {
        let mut chains = self.chains.write().unwrap();
        for versions in chains.values_mut() {
            for version in versions.iter_mut() {
                if version.created_by == tx_id && version.created_commit_ts.is_none() {
                    version.created_commit_ts = Some(commit_ts);
                }
            }
        }
    }

    /// Rolls back all uncommitted versions created by the given transaction.
    /// Removes versions that are still uncommitted (created_commit_ts is None).
    pub fn rollback_versions(&mut self, tx_id: TxId) {
        let mut chains = self.chains.write().unwrap();
        for versions in chains.values_mut() {
            versions.retain(|v| v.created_by != tx_id || v.created_commit_ts.is_some());
        }
    }

    /// Returns the number of keys in the map.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        let chains = self.chains.read().unwrap();
        chains.len()
    }

    /// Returns true if the map is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        let chains = self.chains.read().unwrap();
        chains.is_empty()
    }
}

impl Default for VersionChainMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_chain_append_and_find_visible() {
        let mut chain = VersionChainMap::new();

        // Add v1
        chain.append(
            b"key1".to_vec(),
            RowVersion::new(TxId::new(1), b"v1".to_vec()),
        );
        chain.commit_versions(TxId::new(1), 10);

        // Add v2
        chain.append(
            b"key1".to_vec(),
            RowVersion::new(TxId::new(2), b"v2".to_vec()),
        );
        chain.commit_versions(TxId::new(2), 20);

        // Snapshot at ts=15 should see v1
        let snapshot = Snapshot::new(TxId::new(3), 15, vec![]);
        assert_eq!(chain.find_visible(b"key1", &snapshot), Some(b"v1".to_vec()));

        // Snapshot at ts=25 should see v2
        let snapshot = Snapshot::new(TxId::new(3), 25, vec![]);
        assert_eq!(chain.find_visible(b"key1", &snapshot), Some(b"v2".to_vec()));
    }

    #[test]
    fn test_version_chain_rollback() {
        let mut chain = VersionChainMap::new();

        // Add v1 and commit
        chain.append(
            b"key1".to_vec(),
            RowVersion::new(TxId::new(1), b"v1".to_vec()),
        );
        chain.commit_versions(TxId::new(1), 10);

        // Add v2 but don't commit
        chain.append(
            b"key1".to_vec(),
            RowVersion::new(TxId::new(2), b"v2".to_vec()),
        );

        // Rollback tx2
        chain.rollback_versions(TxId::new(2));

        // Only v1 should remain
        let snapshot = Snapshot::new(TxId::new(3), 100, vec![]);
        assert_eq!(chain.find_visible(b"key1", &snapshot), Some(b"v1".to_vec()));
    }

    #[test]
    fn test_version_chain_not_found() {
        let chain = VersionChainMap::new();
        let snapshot = Snapshot::new(TxId::new(1), 100, vec![]);
        assert_eq!(chain.find_visible(b"nonexistent", &snapshot), None);
    }

    #[test]
    fn test_version_chain_uncommitted_visible_to_self() {
        let mut chain = VersionChainMap::new();

        // TX1 writes but doesn't commit
        chain.append(
            b"key1".to_vec(),
            RowVersion::new(TxId::new(1), b"v1".to_vec()),
        );

        // TX1's own snapshot should see its uncommitted write
        let snapshot = Snapshot::new(TxId::new(1), 100, vec![TxId::new(1)]);
        assert_eq!(chain.find_visible(b"key1", &snapshot), Some(b"v1".to_vec()));

        // TX2's snapshot should NOT see TX1's uncommitted write
        let snapshot = Snapshot::new(TxId::new(2), 100, vec![TxId::new(1)]);
        assert_eq!(chain.find_visible(b"key1", &snapshot), None);
    }

    #[test]
    fn test_version_chain_delete_visibility() {
        let mut chain = VersionChainMap::new();

        // Add v1 and commit
        chain.append(
            b"key1".to_vec(),
            RowVersion::new(TxId::new(1), b"v1".to_vec()),
        );
        chain.commit_versions(TxId::new(1), 10);

        // Mark as deleted at ts=20
        chain.append(b"key1".to_vec(), RowVersion::new_deleted(TxId::new(2)));
        chain.commit_versions(TxId::new(2), 20);

        // Snapshot at ts=15 should see v1
        let snapshot = Snapshot::new(TxId::new(3), 15, vec![]);
        assert_eq!(chain.find_visible(b"key1", &snapshot), Some(b"v1".to_vec()));

        // Snapshot at ts=25 should NOT see v1 (deleted)
        let snapshot = Snapshot::new(TxId::new(3), 25, vec![]);
        assert_eq!(chain.find_visible(b"key1", &snapshot), None);
    }
}
