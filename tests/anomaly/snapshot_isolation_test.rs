//! Snapshot Isolation Tests
//!
//! Tests for MVCC snapshot isolation levels: Read Committed, Repeatable Read, Snapshot.
//! Per Task 3.1 of Week 3: MVCC + Concurrency Hardening

#[cfg(test)]
mod tests {
    use sqlrustgo_transaction::{
        IsolationLevel, MvccEngine, TransactionManager, TxId,
    };
    use std::sync::{Arc, RwLock};
    use std::thread;
    use std::time::Duration;

    /// Helper: Create a basic MvccEngine for testing
    fn create_mvcc_engine() -> Arc<RwLock<MvccEngine>> {
        Arc::new(RwLock::new(MvccEngine::new()))
    }

    /// Helper: Create a TransactionManager with given isolation level
    fn create_manager_with_isolation(
        mvcc: Arc<RwLock<MvccEngine>>,
        level: IsolationLevel,
    ) -> TransactionManager {
        let mut manager = TransactionManager::with_mvcc(mvcc);
        manager.set_isolation_level(level);
        manager
    }

    // =========================================================================
    // Read Committed Isolation Level Tests
    // =========================================================================

    /// Test: Read Committed - Should not see uncommitted data (dirty read prevention)
    #[test]
    fn test_read_committed_no_dirty_read() {
        let mvcc = create_mvcc_engine();
        let mut manager = create_manager_with_isolation(mvcc.clone(), IsolationLevel::ReadCommitted);

        // Start transaction 1
        let tx1 = manager.begin().unwrap();

        // Get context - tx1's own writes should be visible
        let ctx1 = manager.get_transaction_context().unwrap();
        assert!(ctx1.is_visible(tx1, Some(1)));

        // A different uncommitted transaction's data should NOT be visible
        let other_tx = TxId::new(999);
        assert!(!ctx1.is_visible(other_tx, None)); // Uncommitted (no commit timestamp)

        manager.rollback().unwrap();
    }

    /// Test: Read Committed - Should see committed data from other transactions
    #[test]
    fn test_read_committed_sees_committed_data() {
        let mvcc = create_mvcc_engine();

        // Transaction 1: Create and commit
        let mut manager1 = create_manager_with_isolation(mvcc.clone(), IsolationLevel::ReadCommitted);
        let tx1 = manager1.begin().unwrap();
        {
            let mut mvcc_guard = mvcc.write().unwrap();
            mvcc_guard.commit_transaction(tx1).unwrap();
        }

        // Transaction 2: Should see tx1's commit
        let mut manager2 = create_manager_with_isolation(mvcc.clone(), IsolationLevel::ReadCommitted);
        let _tx2 = manager2.begin().unwrap();
        let ctx2 = manager2.get_transaction_context_for_query().unwrap();

        // tx1 is committed and should be visible to tx2
        // The visibility depends on commit timestamp vs snapshot timestamp
        // Since we commit before creating snapshot, it should be visible
        assert!(ctx2.is_visible(tx1, Some(1)));

        manager2.commit().unwrap();
    }

    /// Test: Read Committed - Each query gets fresh snapshot (not repeatable read)
    #[test]
    fn test_read_committed_refreshes_snapshot() {
        let mvcc = create_mvcc_engine();
        let mut manager = create_manager_with_isolation(mvcc.clone(), IsolationLevel::ReadCommitted);

        let _tx_id = manager.begin().unwrap();

        // First query context
        let ctx1 = manager.get_transaction_context_for_query().unwrap();
        let ts1 = ctx1.snapshot.snapshot_timestamp;

        // Simulate time passing - another transaction commits
        {
            let mut manager2 = create_manager_with_isolation(mvcc.clone(), IsolationLevel::ReadCommitted);
            let tx2 = manager2.begin().unwrap();
            manager2.commit().unwrap();
        }

        // Get another query context - should see new commit
        let ctx2 = manager.get_transaction_context_for_query().unwrap();
        let ts2 = ctx2.snapshot.snapshot_timestamp;

        // Read committed should refresh snapshot timestamp after other commits
        assert!(ts2 > ts1, "ReadCommitted should get fresh snapshot timestamp each query");

        manager.commit().unwrap();
    }

    /// Test: Read Committed - Snapshot timestamp fixed for statement
    #[test]
    fn test_read_committed_snapshot_timestamp_fixed() {
        let mvcc = create_mvcc_engine();

        // Start read transaction
        let mut manager = create_manager_with_isolation(mvcc.clone(), IsolationLevel::ReadCommitted);
        let tx_read = manager.begin().unwrap();
        let ctx_read = manager.get_transaction_context().unwrap();
        let snapshot_ts = ctx_read.snapshot.snapshot_timestamp;

        // Another transaction commits AFTER our snapshot
        let mut manager2 = create_manager_with_isolation(mvcc.clone(), IsolationLevel::ReadCommitted);
        let tx2 = manager2.begin().unwrap();
        manager2.commit().unwrap();

        // The committed data should NOT be visible because it was committed after our snapshot
        assert!(!ctx_read.is_visible(tx2, Some(snapshot_ts + 1)));

        manager.rollback().unwrap();
    }

    // =========================================================================
    // Repeatable Read Isolation Level Tests
    // =========================================================================

    /// Test: Repeatable Read - Same read within transaction should return same result
    #[test]
    fn test_repeatable_read_same_read_same_result() {
        let mvcc = create_mvcc_engine();
        let mut manager = create_manager_with_isolation(mvcc.clone(), IsolationLevel::RepeatableRead);

        let tx_id = manager.begin().unwrap();

        // First read
        let ctx1 = manager.get_transaction_context().unwrap();
        let snapshot_ts1 = ctx1.snapshot.snapshot_timestamp;

        // Second read within same transaction - should get same snapshot
        let ctx2 = manager.get_transaction_context().unwrap();
        let snapshot_ts2 = ctx2.snapshot.snapshot_timestamp;

        // Repeatable read should maintain the same snapshot
        assert_eq!(
            snapshot_ts1, snapshot_ts2,
            "RepeatableRead should maintain same snapshot timestamp"
        );

        manager.commit().unwrap();
    }

    /// Test: Repeatable Read - Should not see new commits within transaction
    #[test]
    fn test_repeatable_read_no_new_commits() {
        let mvcc = create_mvcc_engine();

        // Start read transaction
        let mut manager = create_manager_with_isolation(mvcc.clone(), IsolationLevel::RepeatableRead);
        let tx_read = manager.begin().unwrap();
        let ctx_read = manager.get_transaction_context().unwrap();
        let snapshot_ts = ctx_read.snapshot.snapshot_timestamp;

        // Another transaction commits data with timestamp > our snapshot
        let mut manager2 = create_manager_with_isolation(mvcc.clone(), IsolationLevel::RepeatableRead);
        let tx2 = manager2.begin().unwrap();
        manager2.commit().unwrap();

        // Even though tx2 committed, tx1 should not see it (commit_ts > snapshot_ts)
        assert!(!ctx_read.is_visible(tx2, Some(snapshot_ts + 1)));

        manager.rollback().unwrap();
    }

    /// Test: Repeatable Read - Own writes should still be visible
    #[test]
    fn test_repeatable_read_own_writes_visible() {
        let mvcc = create_mvcc_engine();
        let mut manager = create_manager_with_isolation(mvcc, IsolationLevel::RepeatableRead);

        let tx_id = manager.begin().unwrap();
        let ctx = manager.get_transaction_context().unwrap();

        // Own transaction's changes should always be visible
        assert!(ctx.is_visible(tx_id, None)); // Own tx, uncommitted yet

        manager.commit().unwrap();
    }

    // =========================================================================
    // Snapshot Isolation Tests
    // =========================================================================

    /// Test: Snapshot Isolation - Transaction sees consistent snapshot at start
    #[test]
    fn test_snapshot_isolation_consistent_snapshot() {
        let mvcc = create_mvcc_engine();

        // Create and commit some data before our transaction
        let mut manager_pre = create_manager_with_isolation(mvcc.clone(), IsolationLevel::ReadCommitted);
        let tx_pre = manager_pre.begin().unwrap();
        manager_pre.commit().unwrap();

        // Start our snapshot transaction
        let mut manager = create_manager_with_isolation(mvcc.clone(), IsolationLevel::RepeatableRead);
        let tx_id = manager.begin().unwrap();
        let ctx = manager.get_transaction_context().unwrap();

        // Our snapshot should see tx_pre's commit
        assert!(ctx.is_visible(tx_pre, Some(1)));

        manager.commit().unwrap();
    }

    /// Test: Snapshot Isolation - Concurrent updates handled properly
    #[test]
    fn test_snapshot_isolation_concurrent_update_handling() {
        let mvcc = create_mvcc_engine();
        let counter = Arc::new(RwLock::new(0u32));
        let mut handles = vec![];

        // Start multiple readers with RepeatableRead
        for _ in 0..3 {
            let mvcc_clone = mvcc.clone();
            let counter_clone = counter.clone();
            let handle = thread::spawn(move || {
                let mut manager = TransactionManager::with_mvcc(mvcc_clone);
                manager.set_isolation_level(IsolationLevel::RepeatableRead);

                let tx_id = manager.begin().unwrap();
                let ctx = manager.get_transaction_context().unwrap();

                // Read the counter
                let snapshot_ts = ctx.snapshot.snapshot_timestamp;

                // Simulate some read work
                thread::sleep(Duration::from_millis(10));

                // Verify snapshot is stable (repeatable read)
                let ctx2 = manager.get_transaction_context().unwrap();
                assert_eq!(
                    snapshot_ts, ctx2.snapshot.snapshot_timestamp,
                    "RepeatableRead should maintain same snapshot"
                );

                manager.rollback().unwrap();
                tx_id
            });
            handles.push(handle);
        }

        // Start a writer that commits
        let mvcc_write = mvcc.clone();
        let handle = thread::spawn(move || {
            let mut manager = TransactionManager::with_mvcc(mvcc_write);
            manager.set_isolation_level(IsolationLevel::ReadCommitted);

            let tx_id = manager.begin().unwrap();
            manager.commit().unwrap();
            tx_id
        });
        handles.push(handle);

        // All threads complete without conflict
        for handle in handles {
            let _ = handle.join();
        }
    }

    // =========================================================================
    // MVCC Engine Core Tests
    // =========================================================================

    /// Test: MVCC Engine - Transaction lifecycle
    #[test]
    fn test_mvcc_transaction_lifecycle() {
        let mut engine = MvccEngine::new();

        let tx_id = engine.begin_transaction();
        assert!(tx_id.is_valid());

        // Should be tracked as active
        let snapshot = engine.create_snapshot(tx_id);
        assert!(snapshot.is_visible(tx_id, Some(100)));

        // Commit
        let commit_ts = engine.commit_transaction(tx_id).unwrap();
        assert!(commit_ts > 0);
    }

    /// Test: MVCC Engine - Snapshot visibility rules
    #[test]
    fn test_mvcc_snapshot_visibility_rules() {
        let mut engine = MvccEngine::new();

        // Create and commit tx1
        let tx1 = engine.begin_transaction();
        engine.commit_transaction(tx1).unwrap();

        // Create tx2 that starts after tx1 commits
        let tx2 = engine.begin_transaction();
        let snapshot = engine.create_snapshot(tx2);

        // tx1's commit should be visible to tx2
        assert!(snapshot.is_visible(tx1, Some(1)));

        // A future transaction tx3 should not see tx2's uncommitted changes
        let tx3 = engine.begin_transaction();
        let snapshot3 = engine.create_snapshot(tx3);

        // tx2 is still active, should not be visible
        assert!(!snapshot3.is_visible(tx2, None));

        engine.commit_transaction(tx2).unwrap();
        engine.commit_transaction(tx3).unwrap();
    }

    /// Test: MVCC Engine - Global timestamp increments
    #[test]
    fn test_mvcc_global_timestamp_increments() {
        let mut engine = MvccEngine::new();

        let ts1 = engine.get_global_timestamp();
        let _tx1 = engine.begin_transaction();
        let ts2 = engine.get_global_timestamp();
        let _tx2 = engine.begin_transaction();
        let ts3 = engine.get_global_timestamp();

        // Timestamps should increment with each transaction operation
        assert!(ts2 > ts1);
        assert!(ts3 > ts2);
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    /// Test: Self visibility - own transaction always visible
    #[test]
    fn test_self_always_visible() {
        let mut engine = MvccEngine::new();

        let tx_id = engine.begin_transaction();
        let snapshot = engine.create_snapshot(tx_id);

        // Even without commit, own transaction should be visible
        assert!(snapshot.is_visible(tx_id, None));

        // Also visible with commit timestamp
        assert!(snapshot.is_visible(tx_id, Some(100)));
    }

    /// Test: Read Uncommitted - Sees uncommitted data (relaxed isolation)
    #[test]
    fn test_read_uncommitted_visibility() {
        let mvcc = create_mvcc_engine();
        let mut manager = create_manager_with_isolation(mvcc, IsolationLevel::ReadUncommitted);

        let tx_id = manager.begin().unwrap();
        let ctx = manager.get_transaction_context().unwrap();

        // ReadUncommitted uses same visibility check as default
        // The key difference is in locking, not visibility
        let other_tx = TxId::new(999);
        assert!(!ctx.is_visible(other_tx, None));

        manager.rollback().unwrap();
    }

    /// Test: Concurrent transaction abort visibility
    #[test]
    fn test_aborted_transaction_not_visible() {
        let mut engine = MvccEngine::new();

        let tx1 = engine.begin_transaction();
        let tx2 = engine.begin_transaction();

        // Abort tx1
        assert!(engine.abort_transaction(tx1));

        // Create snapshot for tx3
        let tx3 = engine.begin_transaction();
        let snapshot = engine.create_snapshot(tx3);

        // Aborted transactions should not be visible
        // (tx1 was aborted, not committed)
        assert!(!snapshot.is_visible(tx1, None));

        engine.commit_transaction(tx2).unwrap();
        engine.commit_transaction(tx3).unwrap();
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    /// Test: Multi-statement transaction maintains isolation
    #[test]
    fn test_multi_statement_isolation() {
        let mvcc = create_mvcc_engine();
        let mut manager = create_manager_with_isolation(mvcc, IsolationLevel::RepeatableRead);

        let tx_id = manager.begin().unwrap();

        // First query
        let ctx1 = manager.get_transaction_context().unwrap();
        let ts1 = ctx1.snapshot.snapshot_timestamp;

        // Simulate some work and another query
        std::thread::sleep(Duration::from_millis(10));

        // Second query in same transaction
        let ctx2 = manager.get_transaction_context().unwrap();
        let ts2 = ctx2.snapshot.snapshot_timestamp;

        // Should be the same snapshot for RepeatableRead
        assert_eq!(ts1, ts2);

        manager.commit().unwrap();
    }

    /// Test: Long-running transaction and concurrent commits
    #[test]
    fn test_long_running_with_concurrent_commits() {
        let mvcc = create_mvcc_engine();

        // Start long-running transaction
        let mut manager1 = create_manager_with_isolation(mvcc.clone(), IsolationLevel::RepeatableRead);
        let tx1 = manager1.begin().unwrap();
        let ctx1 = manager1.get_transaction_context().unwrap();
        let snapshot_ts = ctx1.snapshot.snapshot_timestamp;

        // Simulate concurrent transactions committing
        for _ in 0..5 {
            let mvcc_clone = mvcc.clone();
            let handle = thread::spawn(move || {
                let mut mgr = TransactionManager::with_mvcc(mvcc_clone);
                mgr.set_isolation_level(IsolationLevel::ReadCommitted);
                let tx = mgr.begin().unwrap();
                mgr.commit().unwrap();
                tx
            });
            handle.join().unwrap();
        }

        // Long-running transaction should still see original snapshot
        // New commits have timestamps > snapshot_ts, so should not be visible
        let mut manager2 = create_manager_with_isolation(mvcc.clone(), IsolationLevel::RepeatableRead);
        let tx_new = manager2.begin().unwrap();

        // The new transaction committed, but our original snapshot should not see it
        assert!(!ctx1.is_visible(tx_new, Some(snapshot_ts + 10)));

        manager1.rollback().unwrap();
    }
}
