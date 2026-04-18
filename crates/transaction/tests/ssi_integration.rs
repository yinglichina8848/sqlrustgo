use sqlrustgo_transaction::{DistributedLockManager, SsiDetector, TxId};
use std::sync::Arc;

#[tokio::test]
async fn test_ssi_concurrent_read_no_conflict() {
    let locks = Arc::new(DistributedLockManager::new());
    let detector = SsiDetector::new(locks);

    let tx1 = TxId::new(1);
    detector.record_read(tx1, b"key1".to_vec()).await;

    let tx2 = TxId::new(2);
    detector.record_read(tx2, b"key1".to_vec()).await;

    assert!(detector.validate_commit(tx1).await.is_ok());
    assert!(detector.validate_commit(tx2).await.is_ok());
}

#[tokio::test]
async fn test_ssi_rw_conflict_detection() {
    let locks = Arc::new(DistributedLockManager::new());
    let detector = SsiDetector::new(locks);

    // Dangerous structure: T1: R(X), W(Y) and T2: R(Y), W(X)
    // Both will have RW-WR conflicts when validating
    let tx1 = TxId::new(1);
    detector.record_read(tx1, b"X".to_vec()).await;
    let _ = detector.record_write(tx1, b"Y".to_vec()).await;

    let tx2 = TxId::new(2);
    detector.record_read(tx2, b"Y".to_vec()).await;
    let _ = detector.record_write(tx2, b"X".to_vec()).await;

    // One of them should fail (whichever validates second finds the cycle)
    let result1 = detector.validate_commit(tx1).await;
    let result2 = detector.validate_commit(tx2).await;

    // At least one should fail due to cycle detection
    assert!(result1.is_err() || result2.is_err());

    // If both err, that's expected for this dangerous structure
    // If one ok and one err, the one that err'd found the conflict
}

#[tokio::test]
async fn test_ssi_write_no_conflict() {
    let locks = Arc::new(DistributedLockManager::new());
    let detector = SsiDetector::new(locks);

    let tx1 = TxId::new(1);
    let _ = detector.record_write(tx1, b"key1".to_vec()).await;

    let tx2 = TxId::new(2);
    let _ = detector.record_write(tx2, b"key2".to_vec()).await;

    assert!(detector.validate_commit(tx1).await.is_ok());
    assert!(detector.validate_commit(tx2).await.is_ok());
}