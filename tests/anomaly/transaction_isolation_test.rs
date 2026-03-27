//! Transaction Isolation Level Tests
//!
//! P1 tests for transaction isolation levels per TEST_PLAN.md
//! Tests READ COMMITTED, REPEATABLE READ, and MVCC visibility

#[cfg(test)]
mod tests {
    use sqlrustgo_transaction::deadlock::DeadlockDetector;
    use sqlrustgo_transaction::manager::{IsolationLevel, TransactionManager};
    use sqlrustgo_transaction::mvcc::{MvccEngine, Snapshot, TransactionStatus, TxId};
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    #[test]
    fn test_read_committed_isolation() {
        let mvcc = Arc::new(RwLock::new(MvccEngine::new()));

        let tx1 = {
            let mut m = mvcc.write().unwrap();
            m.begin_transaction()
        };

        let snapshot = {
            let m = mvcc.read().unwrap();
            m.create_snapshot(tx1)
        };

        let context = sqlrustgo_transaction::manager::TransactionContext::new(
            tx1,
            snapshot,
            IsolationLevel::ReadCommitted,
        );

        let tx2 = TxId::new(100);

        assert!(!context.is_visible(tx2, None));
    }

    #[test]
    fn test_repeatable_read_isolation() {
        let mvcc = Arc::new(RwLock::new(MvccEngine::new()));

        let tx1 = {
            let mut m = mvcc.write().unwrap();
            m.begin_transaction()
        };

        let snapshot = {
            let m = mvcc.read().unwrap();
            m.create_snapshot(tx1)
        };

        let context = sqlrustgo_transaction::manager::TransactionContext::new(
            tx1,
            snapshot,
            IsolationLevel::RepeatableRead,
        );

        let tx2 = TxId::new(100);

        assert!(!context.is_visible(tx2, None));
    }

    #[test]
    fn test_mvcc_snapshot_creation() {
        let mut mvcc = MvccEngine::new();

        let tx1 = mvcc.begin_transaction();

        let snapshot = mvcc.create_snapshot(tx1);

        assert!(snapshot.active_transactions.contains(&tx1));
        assert!(snapshot.snapshot_timestamp >= tx1.as_u64());
    }

    #[test]
    fn test_mvcc_version_visibility() {
        let mut mvcc = MvccEngine::new();

        let tx1 = mvcc.begin_transaction();
        let commit_ts = mvcc.commit_transaction(tx1).unwrap();

        let tx2 = mvcc.begin_transaction();

        let snapshot = mvcc.create_snapshot(tx2);

        assert!(snapshot.is_visible(tx1, Some(commit_ts)));
    }

    #[test]
    fn test_mvcc_uncommitted_not_visible() {
        let mut mvcc = MvccEngine::new();

        let tx1 = mvcc.begin_transaction();

        let tx2 = mvcc.begin_transaction();

        let snapshot = mvcc.create_snapshot(tx2);

        assert!(!snapshot.is_visible(tx1, None));
    }

    #[test]
    fn test_snapshot_refresh() {
        let mut snapshot = Snapshot::new(TxId::new(1), 5, vec![TxId::new(1)]);

        snapshot.refresh_for_read_committed(10);

        assert_eq!(snapshot.snapshot_timestamp, 10);
        assert!(snapshot.active_transactions.is_empty());
    }

    #[test]
    fn test_isolation_level_enum_values() {
        assert_eq!(IsolationLevel::ReadCommitted as u32, 0);
        assert_eq!(IsolationLevel::ReadUncommitted as u32, 1);
        assert_eq!(IsolationLevel::RepeatableRead as u32, 2);
        assert_eq!(IsolationLevel::Serializable as u32, 3);
    }

    #[test]
    fn test_transaction_lifecycle() {
        let mut mvcc = MvccEngine::new();

        let tx_id = mvcc.begin_transaction();

        assert!(tx_id.is_valid());

        let commit_ts = mvcc.commit_transaction(tx_id);

        assert!(commit_ts.is_some());

        let tx = mvcc.get_transaction(tx_id).unwrap();
        assert!(tx.is_committed());
    }

    #[test]
    fn test_transaction_abort() {
        let mut mvcc = MvccEngine::new();

        let tx_id = mvcc.begin_transaction();

        assert!(mvcc.abort_transaction(tx_id));

        let tx = mvcc.get_transaction(tx_id).unwrap();
        assert!(tx.is_aborted());
    }

    #[test]
    fn test_snapshot_clone() {
        let snapshot = Snapshot {
            tx_id: TxId::new(1),
            snapshot_timestamp: 100,
            active_transactions: vec![TxId::new(1), TxId::new(2)],
        };

        let cloned = snapshot.clone();

        assert_eq!(cloned.snapshot_timestamp, snapshot.snapshot_timestamp);
    }

    #[test]
    fn test_mvcc_global_timestamp() {
        let mut mvcc = MvccEngine::new();

        let ts1 = mvcc.get_global_timestamp();

        let tx1 = mvcc.begin_transaction();

        let ts2 = mvcc.get_global_timestamp();

        assert!(ts2 > ts1);
    }

    #[test]
    fn test_mvcc_multiple_transactions() {
        let mut mvcc = MvccEngine::new();

        let tx1 = mvcc.begin_transaction();
        let tx2 = mvcc.begin_transaction();
        let tx3 = mvcc.begin_transaction();

        assert!(tx1.is_valid());
        assert!(tx2.is_valid());
        assert!(tx3.is_valid());

        assert_ne!(tx1, tx2);
        assert_ne!(tx2, tx3);
    }
}

#[cfg(test)]
mod deadlock_tests {
    use sqlrustgo_transaction::deadlock::DeadlockDetector;
    use sqlrustgo_transaction::mvcc::TxId;
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn test_deadlock_detector_creation() {
        let detector = DeadlockDetector::new();

        assert_eq!(detector.get_timeout(), Duration::from_secs(5));
    }

    #[test]
    fn test_deadlock_detector_with_timeout() {
        let detector = DeadlockDetector::with_timeout(Duration::from_secs(10));

        assert_eq!(detector.get_timeout(), Duration::from_secs(10));
    }

    #[test]
    fn test_deadlock_detector_add_edge() {
        let mut detector = DeadlockDetector::new();

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        detector.add_edge(tx1, tx2);

        let cycle = detector.detect_cycle(tx1);

        assert!(cycle.is_none());
    }

    #[test]
    fn test_deadlock_detect_simple_cycle() {
        let mut detector = DeadlockDetector::new();

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);
        let tx3 = TxId::new(3);

        detector.add_edge(tx1, tx2);
        detector.add_edge(tx2, tx3);
        detector.add_edge(tx3, tx1);

        let cycle = detector.detect_cycle(tx1);

        assert!(cycle.is_some());
        let cycle = cycle.unwrap();
        assert!(cycle.contains(&tx1));
        assert!(cycle.contains(&tx2));
        assert!(cycle.contains(&tx3));
    }

    #[test]
    fn test_deadlock_remove_edges() {
        let mut detector = DeadlockDetector::new();

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        detector.add_edge(tx1, tx2);

        detector.remove_edges_for(tx2);

        let cycle = detector.detect_cycle(tx1);

        assert!(cycle.is_none());
    }

    #[test]
    fn test_deadlock_multiple_waiters() {
        let mut detector = DeadlockDetector::new();

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);
        let tx3 = TxId::new(3);

        detector.add_edge(tx1, tx2);
        detector.add_edge(tx1, tx3);

        let cycle = detector.detect_cycle(tx1);

        assert!(cycle.is_none());
    }

    #[test]
    fn test_deadlock_chain_detection() {
        let mut detector = DeadlockDetector::new();

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);
        let tx3 = TxId::new(3);
        let tx4 = TxId::new(4);

        detector.add_edge(tx1, tx2);
        detector.add_edge(tx2, tx3);
        detector.add_edge(tx3, tx4);

        let cycle = detector.detect_cycle(tx1);

        assert!(cycle.is_none());

        detector.add_edge(tx4, tx1);

        let cycle = detector.detect_cycle(tx1);

        assert!(cycle.is_some());
    }

    #[test]
    fn test_deadlock_no_cycle() {
        let mut detector = DeadlockDetector::new();

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);
        let tx3 = TxId::new(3);

        detector.add_edge(tx1, tx2);
        detector.add_edge(tx2, tx3);

        let cycle = detector.detect_cycle(tx1);

        assert!(cycle.is_none());
    }

    #[test]
    fn test_txid_invalid() {
        let tx = TxId::invalid();

        assert!(!tx.is_valid());
        assert_eq!(tx.as_u64(), 0);
    }

    #[test]
    fn test_txid_valid() {
        let tx = TxId::new(42);

        assert!(tx.is_valid());
        assert_eq!(tx.as_u64(), 42);
    }
}
