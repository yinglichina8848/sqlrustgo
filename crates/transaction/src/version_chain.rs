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

pub struct VersionChainStats {
    pub num_keys: usize,
    pub total_versions: usize,
    pub total_deleted: usize,
    pub max_versions_per_key: usize,
}

impl Default for VersionChainMap {
    fn default() -> Self {
        Self::new()
    }
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
        chains.entry(key).or_default().push(version);
    }

    /// Gets a clone of the version chain for a key.
    #[allow(dead_code)]
    pub fn get_chain(&self, key: &[u8]) -> Option<Vec<RowVersion>> {
        let chains = self.chains.read().unwrap();
        chains.get(key).cloned()
    }

    /// Commits all uncommitted versions created by the given transaction.
    /// Updates created_commit_ts for all versions matching tx_id.
    /// For delete markers (empty value), also sets deleted_commit_ts.
    pub fn commit_versions(&mut self, tx_id: TxId, commit_ts: u64) {
        let mut chains = self.chains.write().unwrap();
        for versions in chains.values_mut() {
            for version in versions.iter_mut() {
                if version.created_by == tx_id && version.created_commit_ts.is_none() {
                    version.created_commit_ts = Some(commit_ts);
                    // For delete markers, the creation commit also serves as the delete commit
                    if version.value.is_empty() {
                        version.deleted_commit_ts = Some(commit_ts);
                    }
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

    pub fn gc(&mut self, active_transactions: &[TxId], oldest_snapshot_ts: u64) -> usize {
        let mut removed = 0;
        let keys = {
            let chains = self.chains.read().unwrap();
            chains.keys().cloned().collect::<Vec<_>>()
        };
        for key in keys {
            removed += self.gc_key_versions(&key, active_transactions, oldest_snapshot_ts);
        }
        removed
    }

    const RECENT_GAP_THRESHOLD: u64 = 5;

    fn gc_key_versions(
        &mut self,
        key: &[u8],
        active_transactions: &[TxId],
        oldest_snapshot_ts: u64,
    ) -> usize {
        let mut chains = self.chains.write().unwrap();
        let versions = match chains.get_mut(key) {
            Some(v) => v,
            None => return 0,
        };
        let original_len = versions.len();
        let to_remove = versions
            .iter()
            .enumerate()
            .filter(|(_, v)| {
                if !v.value.is_empty() {
                    return false;
                }
                let deleted_ts = match v.deleted_commit_ts {
                    Some(ts) => ts,
                    None => return false,
                };
                // Preserve if any active transaction exists (conservative)
                if !active_transactions.is_empty() {
                    return false;
                }
                // Preserve if deletion is "recent" (within threshold of oldest snapshot)
                // gap = oldest_snapshot_ts - deleted_ts
                // if gap <= 5, deletion happened within 5 timesteps of oldest snapshot
                let gap = oldest_snapshot_ts.saturating_sub(deleted_ts);
                if gap <= Self::RECENT_GAP_THRESHOLD {
                    return false;
                }
                true
            })
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        for i in to_remove.into_iter().rev() {
            versions.remove(i);
        }
        original_len - versions.len()
    }

    #[allow(dead_code)]
    pub fn stats(&self) -> VersionChainStats {
        let chains = self.chains.read().unwrap();
        let mut total_versions = 0;
        let mut total_deleted = 0;
        let mut max_versions_per_key = 0;
        for versions in chains.values() {
            total_versions += versions.len();
            for v in versions {
                if v.value.is_empty() {
                    total_deleted += 1;
                }
            }
            max_versions_per_key = max_versions_per_key.max(versions.len());
        }
        VersionChainStats {
            num_keys: chains.len(),
            total_versions,
            total_deleted,
            max_versions_per_key,
        }
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

    #[test]
    fn test_version_chain_gc_collect_deleted() {
        let mut chain = VersionChainMap::new();
        chain.append(
            b"key1".to_vec(),
            RowVersion::new(TxId::new(1), b"v1".to_vec()),
        );
        chain.commit_versions(TxId::new(1), 10);
        chain.append(b"key1".to_vec(), RowVersion::new_deleted(TxId::new(2)));
        chain.commit_versions(TxId::new(2), 20);
        assert_eq!(chain.stats().total_versions, 2);
        let removed = chain.gc(&[], 30);
        assert_eq!(removed, 1);
        assert_eq!(chain.stats().total_versions, 1);
    }

    #[test]
    fn test_version_chain_gc_preserve_recent() {
        let mut chain = VersionChainMap::new();
        chain.append(
            b"key1".to_vec(),
            RowVersion::new(TxId::new(1), b"v1".to_vec()),
        );
        chain.commit_versions(TxId::new(1), 10);
        chain.append(b"key1".to_vec(), RowVersion::new_deleted(TxId::new(2)));
        chain.commit_versions(TxId::new(2), 20);
        let removed = chain.gc(&[], 25);
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_version_chain_gc_preserve_active_tx() {
        let mut chain = VersionChainMap::new();
        chain.append(
            b"key1".to_vec(),
            RowVersion::new(TxId::new(1), b"v1".to_vec()),
        );
        chain.commit_versions(TxId::new(1), 10);
        chain.append(b"key1".to_vec(), RowVersion::new_deleted(TxId::new(2)));
        chain.commit_versions(TxId::new(2), 20);
        let removed = chain.gc(&[TxId::new(1)], 30);
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_version_chain_stats() {
        let mut chain = VersionChainMap::new();
        chain.append(
            b"key1".to_vec(),
            RowVersion::new(TxId::new(1), b"v1".to_vec()),
        );
        chain.commit_versions(TxId::new(1), 10);
        chain.append(
            b"key1".to_vec(),
            RowVersion::new(TxId::new(2), b"v2".to_vec()),
        );
        chain.commit_versions(TxId::new(2), 20);
        chain.append(b"key2".to_vec(), RowVersion::new_deleted(TxId::new(3)));
        chain.commit_versions(TxId::new(3), 30);
        let stats = chain.stats();
        assert_eq!(stats.num_keys, 2);
        assert_eq!(stats.total_versions, 3);
        assert_eq!(stats.total_deleted, 1);
        assert_eq!(stats.max_versions_per_key, 2);
    }
}
