use sqlrustgo_transaction::{MvccSsiStorageEngine, Snapshot, TxId};

#[test]
fn test_ssi_regression_basic_read_write() {
    let mut storage = MvccSsiStorageEngine::new();

    storage.write_version(b"key1".to_vec(), b"value1".to_vec(), TxId::new(1));

    let snapshot = Snapshot::new(TxId::new(2), 100, vec![]);
    assert_eq!(storage.read(b"key1", &snapshot), None);

    storage.commit_versions(TxId::new(1), 10).unwrap();

    assert_eq!(storage.read(b"key1", &snapshot), Some(b"value1".to_vec()));
}

#[test]
fn test_ssi_regression_no_conflict_independent_keys() {
    let mut storage = MvccSsiStorageEngine::new();

    let tx1 = TxId::new(1);
    storage.write_version(b"key1".to_vec(), b"v1".to_vec(), tx1);

    let tx2 = TxId::new(2);
    storage.write_version(b"key2".to_vec(), b"v2".to_vec(), tx2);

    assert!(storage.commit_versions(tx1, 10).is_ok());
    assert!(storage.commit_versions(tx2, 20).is_ok());
}

#[test]
fn test_ssi_regression_dangerous_structure_detection() {
    let mut storage = MvccSsiStorageEngine::new();

    let tx0 = TxId::new(0);
    storage.write_version(b"X".to_vec(), b"x0".to_vec(), tx0);
    storage.write_version(b"Y".to_vec(), b"y0".to_vec(), tx0);
    storage.commit_versions(tx0, 5).unwrap();

    let tx1 = TxId::new(1);
    storage.record_read(tx1, b"X".to_vec());
    storage.record_read(tx1, b"Y".to_vec());
    storage.write_version(b"X".to_vec(), b"x1".to_vec(), tx1);

    let tx2 = TxId::new(2);
    storage.record_read(tx2, b"X".to_vec());
    storage.record_read(tx2, b"Y".to_vec());
    storage.write_version(b"Y".to_vec(), b"y2".to_vec(), tx2);

    let result1 = storage.commit_versions(tx1, 10);
    let result2 = storage.commit_versions(tx2, 20);

    assert!(result1.is_ok() || result2.is_ok());
    assert!(result1.is_err() || result2.is_err());
}

#[test]
fn test_ssi_regression_snapshot_isolation() {
    let mut storage = MvccSsiStorageEngine::new();

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
fn test_ssi_regression_rollback() {
    let mut storage = MvccSsiStorageEngine::new();

    storage.write_version(b"key1".to_vec(), b"v1".to_vec(), TxId::new(1));
    storage.commit_versions(TxId::new(1), 10).unwrap();

    storage.write_version(b"key1".to_vec(), b"v2".to_vec(), TxId::new(2));

    storage.rollback_versions(TxId::new(2)).unwrap();

    let snapshot = Snapshot::new(TxId::new(3), 100, vec![]);
    assert_eq!(storage.read(b"key1", &snapshot), Some(b"v1".to_vec()));
}

#[test]
fn test_ssi_regression_delete() {
    let mut storage = MvccSsiStorageEngine::new();

    storage.write_version(b"key1".to_vec(), b"value1".to_vec(), TxId::new(1));
    storage.commit_versions(TxId::new(1), 10).unwrap();

    storage.write_version(b"key1".to_vec(), b"deleted".to_vec(), TxId::new(2));
    storage.commit_versions(TxId::new(2), 20).unwrap();

    let snapshot = Snapshot::new(TxId::new(3), 25, vec![]);
    assert_eq!(storage.read(b"key1", &snapshot), Some(b"deleted".to_vec()));
}
