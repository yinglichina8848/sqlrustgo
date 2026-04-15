//! MVCC Storage trait and implementation
//!
//! Provides the MVCCStorage trait for snapshot-isolated storage operations.

use crate::manager::TransactionError;
use crate::mvcc::{RowVersion, Snapshot, TxId};
use crate::version_chain::VersionChainMap;
use std::sync::RwLock;

pub trait MVCCStorage: Send + Sync {
    fn read(&self, key: &[u8], snapshot: &Snapshot) -> Option<Vec<u8>>;
    fn write_version(&mut self, key: Vec<u8>, value: Vec<u8>, tx_id: TxId);
    fn delete_version(&mut self, key: Vec<u8>, tx_id: TxId);
    fn commit_versions(&mut self, tx_id: TxId, commit_ts: u64) -> Result<(), TransactionError>;
    fn rollback_versions(&mut self, tx_id: TxId) -> Result<(), TransactionError>;
}

pub struct MVCCStorageEngine {
    chains: RwLock<VersionChainMap>,
}

impl Default for MVCCStorageEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MVCCStorageEngine {
    pub fn new() -> Self {
        Self {
            chains: RwLock::new(VersionChainMap::new()),
        }
    }
}

impl MVCCStorage for MVCCStorageEngine {
    fn read(&self, key: &[u8], snapshot: &Snapshot) -> Option<Vec<u8>> {
        let chains = self.chains.read().unwrap();
        chains.find_visible(key, snapshot)
    }

    fn write_version(&mut self, key: Vec<u8>, value: Vec<u8>, tx_id: TxId) {
        let mut chains = self.chains.write().unwrap();
        chains.append(key, RowVersion::new(tx_id, value));
    }

    fn delete_version(&mut self, key: Vec<u8>, tx_id: TxId) {
        let mut chains = self.chains.write().unwrap();
        chains.append(key, RowVersion::new_deleted(tx_id));
    }

    fn commit_versions(&mut self, tx_id: TxId, commit_ts: u64) -> Result<(), TransactionError> {
        let mut chains = self.chains.write().unwrap();
        chains.commit_versions(tx_id, commit_ts);
        Ok(())
    }

    fn rollback_versions(&mut self, tx_id: TxId) -> Result<(), TransactionError> {
        let mut chains = self.chains.write().unwrap();
        chains.rollback_versions(tx_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mvcc_storage_read_write() {
        let mut storage = MVCCStorageEngine::new();

        storage.write_version(b"key1".to_vec(), b"value1".to_vec(), TxId::new(1));

        let snapshot = Snapshot::new(TxId::new(2), 100, vec![]);
        assert_eq!(storage.read(b"key1", &snapshot), None);

        storage.commit_versions(TxId::new(1), 10).unwrap();

        assert_eq!(storage.read(b"key1", &snapshot), Some(b"value1".to_vec()));
    }

    #[test]
    fn test_mvcc_storage_snapshot_isolation() {
        let mut storage = MVCCStorageEngine::new();

        storage.write_version(b"key1".to_vec(), b"v1".to_vec(), TxId::new(1));
        storage.commit_versions(TxId::new(1), 10).unwrap();

        storage.write_version(b"key1".to_vec(), b"v2".to_vec(), TxId::new(2));
        storage.commit_versions(TxId::new(2), 20).unwrap();

        let snapshot15 = Snapshot::new(TxId::new(3), 15, vec![]);
        assert_eq!(storage.read(b"key1", &snapshot15), Some(b"v1".to_vec()));

        let snapshot25 = Snapshot::new(TxId::new(4), 25, vec![]);
        assert_eq!(storage.read(b"key1", &snapshot25), Some(b"v2".to_vec()));
    }

    #[test]
    fn test_mvcc_storage_delete() {
        let mut storage = MVCCStorageEngine::new();

        storage.write_version(b"key1".to_vec(), b"value1".to_vec(), TxId::new(1));
        storage.commit_versions(TxId::new(1), 10).unwrap();

        storage.delete_version(b"key1".to_vec(), TxId::new(2));
        storage.commit_versions(TxId::new(2), 20).unwrap();

        let snapshot15 = Snapshot::new(TxId::new(3), 15, vec![]);
        assert_eq!(storage.read(b"key1", &snapshot15), Some(b"value1".to_vec()));

        let snapshot25 = Snapshot::new(TxId::new(4), 25, vec![]);
        assert_eq!(storage.read(b"key1", &snapshot25), None);
    }

    #[test]
    fn test_mvcc_storage_rollback() {
        let mut storage = MVCCStorageEngine::new();

        storage.write_version(b"key1".to_vec(), b"v1".to_vec(), TxId::new(1));
        storage.commit_versions(TxId::new(1), 10).unwrap();

        storage.write_version(b"key1".to_vec(), b"v2".to_vec(), TxId::new(2));

        storage.rollback_versions(TxId::new(2)).unwrap();

        let snapshot = Snapshot::new(TxId::new(3), 100, vec![]);
        assert_eq!(storage.read(b"key1", &snapshot), Some(b"v1".to_vec()));
    }
}
