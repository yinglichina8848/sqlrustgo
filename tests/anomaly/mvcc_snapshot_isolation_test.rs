//! MVCC Snapshot Isolation Integration Tests
//!
//! These tests verify end-to-end MVCC snapshot isolation behavior.

#[cfg(test)]
mod tests {
    use sqlrustgo_transaction::manager::TransactionManager;
    use sqlrustgo_transaction::mvcc::{Snapshot, TxId};
    use sqlrustgo_transaction::mvcc_storage::{MVCCStorage, MVCCStorageEngine};

    #[test]
    fn test_snapshot_isolation_read_consistency() {
        let mut storage = MVCCStorageEngine::new();
        let mut manager1 = TransactionManager::new();
        let mut manager2 = TransactionManager::new();

        let tx1 = manager1.begin().unwrap();
        storage.write_version(b"counter".to_vec(), b"1".to_vec(), tx1);
        let _commit_ts = manager1.mvcc_commit(&mut storage).unwrap().unwrap();

        let _tx2 = manager2.begin().unwrap();
        let ctx2 = manager2.get_transaction_context_for_query().unwrap();
        assert_eq!(
            storage.read(b"counter", &ctx2.snapshot),
            Some(b"1".to_vec())
        );

        let tx1 = manager1.begin().unwrap();
        storage.write_version(b"counter".to_vec(), b"2".to_vec(), tx1);
        let _ = manager1.mvcc_commit(&mut storage);

        assert_eq!(
            storage.read(b"counter", &ctx2.snapshot),
            Some(b"1".to_vec())
        );
    }

    #[test]
    fn test_no_dirty_read() {
        let mut storage = MVCCStorageEngine::new();
        let mut manager1 = TransactionManager::new();
        let mut manager2 = TransactionManager::new();

        let tx1 = manager1.begin().unwrap();
        storage.write_version(b"data".to_vec(), b"secret".to_vec(), tx1);

        let _tx2 = manager2.begin().unwrap();
        let ctx2 = manager2.get_transaction_context_for_query().unwrap();

        let result = storage.read(b"data", &ctx2.snapshot);
        assert_eq!(result, Some(b"secret".to_vec()));
    }

    #[test]
    fn test_rollback_discard() {
        let mut storage = MVCCStorageEngine::new();
        let mut manager = TransactionManager::new();

        let tx_id = manager.begin().unwrap();
        storage.write_version(b"data".to_vec(), b"temp".to_vec(), tx_id);

        manager.mvcc_rollback(&mut storage).unwrap();

        let snapshot = Snapshot::new_read_committed(TxId::new(999), 100);
        assert_eq!(storage.read(b"data", &snapshot), None);
    }

    #[test]
    fn test_concurrent_read_write_same_key() {
        let mut storage = MVCCStorageEngine::new();

        storage.write_version(b"key".to_vec(), b"v1".to_vec(), TxId::new(1));
        storage.commit_versions(TxId::new(1), 10).unwrap();

        let snapshot1 = Snapshot::new(TxId::new(100), 15, vec![]);
        assert_eq!(storage.read(b"key", &snapshot1), Some(b"v1".to_vec()));

        storage.write_version(b"key".to_vec(), b"v2".to_vec(), TxId::new(2));
        storage.commit_versions(TxId::new(2), 20).unwrap();

        let snapshot2 = Snapshot::new(TxId::new(101), 25, vec![]);
        assert_eq!(storage.read(b"key", &snapshot2), Some(b"v2".to_vec()));

        assert_eq!(storage.read(b"key", &snapshot1), Some(b"v1".to_vec()));
    }

    #[test]
    fn test_delete_hides_previous_value() {
        let mut storage = MVCCStorageEngine::new();

        storage.write_version(b"key".to_vec(), b"value".to_vec(), TxId::new(1));
        storage.commit_versions(TxId::new(1), 10).unwrap();

        let snapshot_before_delete = Snapshot::new(TxId::new(2), 15, vec![]);
        assert_eq!(
            storage.read(b"key", &snapshot_before_delete),
            Some(b"value".to_vec())
        );

        storage.delete_version(b"key".to_vec(), TxId::new(2));
        storage.commit_versions(TxId::new(2), 20).unwrap();

        let snapshot_after_delete = Snapshot::new(TxId::new(3), 25, vec![]);
        assert_eq!(storage.read(b"key", &snapshot_after_delete), None);
    }
}
