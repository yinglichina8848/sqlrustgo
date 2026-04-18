//! MVCC Storage trait and implementation
//!
//! Provides the MVCCStorage trait for snapshot-isolated storage operations.

use crate::manager::TransactionError;
use crate::mvcc::{RowVersion, Snapshot, TxId};
use crate::ssi::SsiDetectorSync;
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

/// SSI-aware MVCC storage engine that detects serialization conflicts
pub struct MvccSsiStorageEngine {
    chains: RwLock<VersionChainMap>,
    ssi_detector: SsiDetectorSync,
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

impl MvccSsiStorageEngine {
    pub fn new() -> Self {
        Self {
            chains: RwLock::new(VersionChainMap::new()),
            ssi_detector: SsiDetectorSync::new(),
        }
    }

    pub fn record_read(&self, tx_id: TxId, key: Vec<u8>) {
        self.ssi_detector.record_read(tx_id, key);
    }

    pub fn record_write(&self, tx_id: TxId, key: Vec<u8>) {
        self.ssi_detector.record_write(tx_id, key);
    }

    pub fn read(&self, key: &[u8], snapshot: &Snapshot) -> Option<Vec<u8>> {
        let chains = self.chains.read().unwrap();
        chains.find_visible(key, snapshot)
    }

    pub fn write_version(&mut self, key: Vec<u8>, value: Vec<u8>, tx_id: TxId) {
        self.ssi_detector.record_write(tx_id, key.clone());
        let mut chains = self.chains.write().unwrap();
        chains.append(key, RowVersion::new(tx_id, value));
    }

    pub fn commit_versions(&mut self, tx_id: TxId, commit_ts: u64) -> Result<(), TransactionError> {
        if let Err(ssi_err) = self.ssi_detector.validate_commit(tx_id) {
            self.ssi_detector.release(tx_id);
            return Err(TransactionError::SerializationConflict(ssi_err.to_string()));
        }

        {
            let mut chains = self.chains.write().unwrap();
            chains.commit_versions(tx_id, commit_ts);
        }
        self.ssi_detector.release(tx_id);
        Ok(())
    }

    pub fn rollback_versions(&mut self, tx_id: TxId) -> Result<(), TransactionError> {
        self.ssi_detector.release(tx_id);
        let mut chains = self.chains.write().unwrap();
        chains.rollback_versions(tx_id);
        Ok(())
    }
}

impl Default for MvccSsiStorageEngine {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn test_mvcc_ssi_storage_read_write() {
        let mut storage = MvccSsiStorageEngine::new();

        storage.write_version(b"key1".to_vec(), b"value1".to_vec(), TxId::new(1));

        let snapshot = Snapshot::new(TxId::new(2), 100, vec![]);
        assert_eq!(storage.read(b"key1", &snapshot), None);

        storage.commit_versions(TxId::new(1), 10).unwrap();

        assert_eq!(storage.read(b"key1", &snapshot), Some(b"value1".to_vec()));
    }

    #[test]
    fn test_mvcc_ssi_storage_dangerous_structure() {
        let mut storage = MvccSsiStorageEngine::new();

        // 先提交一些初始数据建立版本链
        let tx0 = TxId::new(0);
        storage.write_version(b"X".to_vec(), b"x0".to_vec(), tx0);
        storage.write_version(b"Y".to_vec(), b"y0".to_vec(), tx0);
        storage.commit_versions(tx0, 5).unwrap();

        // T1: R(X), R(Y), W(X)
        let tx1 = TxId::new(1);
        storage.record_read(tx1, b"X".to_vec()); // T1 读取 X
        storage.record_read(tx1, b"Y".to_vec()); // T1 读取 Y
        storage.write_version(b"X".to_vec(), b"x1".to_vec(), tx1); // T1 写入 X

        // T2: R(X), R(Y), W(Y) - 读取 T1 即将写入的相同键
        let tx2 = TxId::new(2);
        storage.record_read(tx2, b"X".to_vec()); // T2 读取 X
        storage.record_read(tx2, b"Y".to_vec()); // T2 读取 Y
        storage.write_version(b"Y".to_vec(), b"y2".to_vec(), tx2); // T2 写入 Y

        // 两个事务都读取了相同的键 {X, Y}，但写入不同的键
        // 这形成了 RW-WR 循环：T1 读 X 后 T2 写了 X，T2 读 Y 后 T1 写了 Y
        // 至少有一个提交会失败
        let result1 = storage.commit_versions(tx1, 10);
        let result2 = storage.commit_versions(tx2, 20);

        // 只有一个应该成功，或者都失败（取决于检测顺序）
        assert!(
            result1.is_ok() || result2.is_ok(),
            "At least one should commit successfully"
        );
        assert!(
            result1.is_err() || result2.is_err(),
            "At least one should fail due to serialization conflict"
        );
    }

    #[test]
    fn test_mvcc_ssi_storage_no_conflict() {
        let mut storage = MvccSsiStorageEngine::new();

        let tx1 = TxId::new(1);
        storage.write_version(b"key1".to_vec(), b"v1".to_vec(), tx1);

        let tx2 = TxId::new(2);
        storage.write_version(b"key2".to_vec(), b"v2".to_vec(), tx2);

        assert!(storage.commit_versions(tx1, 10).is_ok());
        assert!(storage.commit_versions(tx2, 20).is_ok());
    }
}
