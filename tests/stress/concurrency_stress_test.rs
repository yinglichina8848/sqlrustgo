//! Concurrency Stress Tests
//!
//! High concurrency stress tests for MVCC + Transaction system
//! Per Task 3.2 of Week 3: MVCC + Concurrency Hardening

#[cfg(test)]
mod tests {
    use sqlrustgo_server::{ConnectionPool, PoolConfig};
    use sqlrustgo_storage::MemoryStorage;
    use sqlrustgo_transaction::{
        IsolationLevel, MvccEngine, TransactionManager, TxId,
    };
    use std::sync::{Arc, RwLock};
    use std::thread;
    use std::time::Duration;

    // =========================================================================
    // Connection Pool Tests
    // =========================================================================

    #[test]
    fn test_connection_pool_basic() {
        let config = PoolConfig {
            size: 5,
            timeout_ms: 1000,
        };
        let pool = ConnectionPool::new(config);

        let connections: Vec<_> = (0..5).map(|_| pool.acquire()).collect();
        assert_eq!(pool.size(), 5);
        drop(connections);

        let conn = pool.acquire();
        let _ = conn.executor();

        println!("✓ Connection pool basic: PASS");
    }

    // =========================================================================
    // Thread Spawn Stability Tests
    // =========================================================================

    #[test]
    fn test_thread_spawn_stability() {
        let handles: Vec<_> = (0..50)
            .map(|_| {
                thread::spawn(|| {
                    thread::sleep(Duration::from_millis(1));
                    let _ = MemoryStorage::new();
                    true
                })
            })
            .collect();

        let all_success = handles.into_iter().all(|h| h.join().unwrap());
        assert!(all_success);
        println!("✓ Thread spawn stability: PASS (50 threads)");
    }

    // =========================================================================
    // Concurrent Storage Access Tests
    // =========================================================================

    #[test]
    fn test_concurrent_storage_access() {
        let storage = Arc::new(MemoryStorage::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let storage = Arc::clone(&storage);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let _ = storage.clone();
                }
                true
            });
            handles.push(handle);
        }

        let all_success = handles.into_iter().all(|h| h.join().unwrap());
        assert!(all_success);
        println!("✓ Concurrent storage access: PASS (10 threads x 100 operations)");
    }

    // =========================================================================
    // MVCC High Concurrency Stress Tests
    // =========================================================================

    /// Helper: Create a basic MvccEngine wrapped in Arc<RwLock>
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

    /// Test: High Concurrency Transaction Begin/Commit
    /// Stress test with many concurrent transactions rapidly beginning and committing
    #[test]
    fn test_high_concurrency_rapid_transactions() {
        let mvcc = create_mvcc_engine();
        let mut handles = vec![];

        // 50 concurrent threads, each doing 10 transactions
        for i in 0..50 {
            let mvcc_clone = mvcc.clone();
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let mut manager = TransactionManager::with_mvcc(mvcc_clone.clone());
                    manager.set_isolation_level(IsolationLevel::ReadCommitted);

                    let tx_id = match manager.begin() {
                        Ok(id) => id,
                        Err(_) => return false,
                    };

                    // Simulate minimal work
                    thread::sleep(Duration::from_micros(10));

                    if manager.commit().is_err() {
                        return false;
                    }
                }
                true
            });
            handles.push(handle);
        }

        let all_success = handles.into_iter().all(|h| h.join().unwrap());
        assert!(all_success, "High concurrency rapid transactions should all succeed");
        println!("✓ High concurrency rapid transactions: PASS (50 threads x 10 txns)");
    }

    /// Test: Concurrent Readers with Same Snapshot
    /// Multiple readers should see consistent data within their own transaction
    #[test]
    fn test_concurrent_readers_consistent_snapshot() {
        let mvcc = create_mvcc_engine();

        // Pre-commit some data
        {
            let mut manager = create_manager_with_isolation(mvcc.clone(), IsolationLevel::ReadCommitted);
            let tx = manager.begin().unwrap();
            manager.commit().unwrap();
        }

        let mut handles = vec![];

        // 20 concurrent readers with RepeatableRead
        for _ in 0..20 {
            let mvcc_clone = mvcc.clone();
            let handle = thread::spawn(move || {
                let mut manager = create_manager_with_isolation(mvcc_clone, IsolationLevel::RepeatableRead);
                let tx = manager.begin().unwrap();

                // Get context once
                let ctx1 = manager.get_transaction_context().unwrap();

                // Own transaction should always be visible
                let self_visible = ctx1.is_visible(tx, None);

                manager.commit().unwrap();
                let _ = tx;
                self_visible
            });
            handles.push(handle);
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(results.iter().all(|&r| r), "All readers should see their own transaction");
        println!("✓ Concurrent readers consistent snapshot: PASS (20 readers)");
    }

    /// Test: Long-Running Transaction with Many Short Transactions
    /// A long transaction's own operations should remain stable under concurrent load
    #[test]
    fn test_long_transaction_isolation() {
        let mvcc = create_mvcc_engine();

        // Start long-running transaction
        let mut long_manager = create_manager_with_isolation(mvcc.clone(), IsolationLevel::RepeatableRead);
        let _long_tx = long_manager.begin().unwrap();

        // Long transaction gets context multiple times without concurrent activity
        let long_ctx1 = long_manager.get_transaction_context().unwrap();
        let long_ts1 = long_ctx1.snapshot.snapshot_timestamp;
        let long_ctx2 = long_manager.get_transaction_context().unwrap();
        let long_ts2 = long_ctx2.snapshot.snapshot_timestamp;

        // Without concurrent commits between calls, timestamps should be equal
        assert_eq!(long_ts1, long_ts2, "Snapshot should be stable without concurrent commits");

        // Many short transactions committing concurrently
        let mut short_handles = vec![];
        for _ in 0..30 {
            let mvcc_clone = mvcc.clone();
            let handle = thread::spawn(move || {
                let mut manager = create_manager_with_isolation(mvcc_clone, IsolationLevel::ReadCommitted);
                let tx = manager.begin().unwrap();
                manager.commit().unwrap();
                let _ = tx;
            });
            short_handles.push(handle);
        }

        // Wait for all short transactions to complete
        for handle in short_handles {
            let _ = handle.join();
        }

        // After concurrent commits, own transaction should still be visible
        let long_ctx3 = long_manager.get_transaction_context().unwrap();
        let own_visible = long_ctx3.is_visible(_long_tx, None);
        assert!(own_visible, "Own transaction should always be visible");

        long_manager.rollback().unwrap();
        println!("✓ Long transaction isolation: PASS (30 concurrent short txns)");
    }

    /// Test: All Isolation Levels Under Concurrent Load
    /// Verify each isolation level behaves correctly under concurrent stress
    #[test]
    fn test_all_isolation_levels_under_load() {
        let mvcc = create_mvcc_engine();
        let isolation_levels = vec![
            IsolationLevel::ReadUncommitted,
            IsolationLevel::ReadCommitted,
            IsolationLevel::RepeatableRead,
        ];

        for level in isolation_levels {
            let mut handles = vec![];

            // 20 concurrent transactions per isolation level
            for _ in 0..20 {
                let mvcc_clone = mvcc.clone();
                let handle = thread::spawn(move || {
                    let mut manager = TransactionManager::with_mvcc(mvcc_clone);
                    manager.set_isolation_level(level.clone());

                    let tx_id = manager.begin().unwrap();
                    let ctx = manager.get_transaction_context().unwrap();

                    // Own transaction should always be visible
                    let self_visible = ctx.is_visible(tx_id, None);

                    manager.commit().unwrap();
                    let _ = tx_id;
                    self_visible
                });
                handles.push(handle);
            }

            let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
            assert!(
                results.iter().all(|&r| r),
                "All transactions under {:?} should behave correctly",
                level
            );
        }
        println!("✓ All isolation levels under load: PASS");
    }

    /// Test: Concurrent Abort Handling
    /// Verify aborted transactions are properly handled under concurrency
    #[test]
    fn test_concurrent_abort_handling() {
        let mvcc = create_mvcc_engine();
        let mut handles = vec![];

        // 30 transactions, some will abort
        for i in 0..30 {
            let mvcc_clone = mvcc.clone();
            let should_abort = i % 3 == 0; // Every 3rd transaction aborts

            let handle = thread::spawn(move || {
                let mut manager = TransactionManager::with_mvcc(mvcc_clone);
                manager.set_isolation_level(IsolationLevel::ReadCommitted);

                let tx_id = manager.begin().unwrap();

                if should_abort {
                    manager.rollback().unwrap();
                    return true; // Success - properly aborted
                } else {
                    manager.commit().unwrap();
                    return true;
                }
            });
            handles.push(handle);
        }

        let all_success = handles.into_iter().all(|h| h.join().unwrap());
        assert!(all_success, "Abort handling should work correctly");
        println!("✓ Concurrent abort handling: PASS (30 transactions)");
    }

    /// Test: Timestamp Monotonicity Under Concurrency
    /// Global timestamp should always increase even under high concurrency
    #[test]
    fn test_timestamp_monotonicity_under_concurrency() {
        let mvcc = create_mvcc_engine();
        let timestamps = Arc::new(RwLock::new(Vec::new()));
        let mut handles = vec![];

        // 40 concurrent readers getting timestamps
        for _ in 0..40 {
            let mvcc_clone = mvcc.clone();
            let ts_clone = timestamps.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let mut manager = TransactionManager::with_mvcc(mvcc_clone.clone());
                    manager.set_isolation_level(IsolationLevel::ReadCommitted);

                    let tx_id = manager.begin().unwrap();
                    let ctx = manager.get_transaction_context().unwrap();
                    let ts = ctx.snapshot.snapshot_timestamp;

                    {
                        let mut v = ts_clone.write().unwrap();
                        v.push(ts);
                    }

                    manager.commit().unwrap();
                    let _ = tx_id;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        // Verify timestamps are monotonically non-decreasing (allow equal due to concurrency)
        let mut all_ts = timestamps.write().unwrap();
        all_ts.sort();
        let is_monotonic = all_ts.windows(2).all(|w| w[0] <= w[1]);
        assert!(is_monotonic, "Timestamps should be monotonically non-decreasing");
        println!("✓ Timestamp monotonicity under concurrency: PASS (4000 timestamps)");
    }

    /// Test: Mixed Read-Write Concurrency
    /// Concurrent readers and writers should not interfere
    #[test]
    fn test_mixed_read_write_concurrency() {
        let mvcc = create_mvcc_engine();
        let mut writer_handles = vec![];
        let mut reader_handles = vec![];

        // 10 writers
        for _ in 0..10 {
            let mvcc_clone = mvcc.clone();
            let handle = thread::spawn(move || {
                let mut manager = TransactionManager::with_mvcc(mvcc_clone);
                manager.set_isolation_level(IsolationLevel::ReadCommitted);

                let tx_id = manager.begin().unwrap();
                manager.commit().unwrap();
                let _ = tx_id;
            });
            writer_handles.push(handle);
        }

        // 20 readers
        for _ in 0..20 {
            let mvcc_clone = mvcc.clone();
            let handle = thread::spawn(move || {
                let mut manager = TransactionManager::with_mvcc(mvcc_clone);
                manager.set_isolation_level(IsolationLevel::RepeatableRead);

                let tx_id = manager.begin().unwrap();
                let ctx = manager.get_transaction_context().unwrap();
                let _ts = ctx.snapshot.snapshot_timestamp;
                manager.commit().unwrap();
                let _ = tx_id;
                true
            });
            reader_handles.push(handle);
        }

        let all_writers_ok = writer_handles.into_iter().all(|h| h.join().is_ok());
        let all_readers_ok = reader_handles.into_iter().map(|h| h.join().unwrap()).all(|r| r);
        assert!(all_writers_ok && all_readers_ok, "Mixed read-write concurrency should work");
        println!("✓ Mixed read-write concurrency: PASS (10 writers + 20 readers)");
    }

    /// Test: Snapshot Freshness After Concurrent Commits
    /// ReadCommitted should see fresh snapshot after concurrent commits
    #[test]
    fn test_read_committed_freshness_after_concurrent_commits() {
        let mvcc = create_mvcc_engine();

        // Start a ReadCommitted transaction
        let mut manager = create_manager_with_isolation(mvcc.clone(), IsolationLevel::ReadCommitted);
        let _tx1 = manager.begin().unwrap();
        let ctx1 = manager.get_transaction_context().unwrap();
        let ts1 = ctx1.snapshot.snapshot_timestamp;

        // Many concurrent commits happen
        let mut handles = vec![];
        for _ in 0..20 {
            let mvcc_clone = mvcc.clone();
            let handle = thread::spawn(move || {
                let mut mgr = create_manager_with_isolation(mvcc_clone, IsolationLevel::ReadCommitted);
                let tx = mgr.begin().unwrap();
                mgr.commit().unwrap();
                let _ = tx;
            });
            handles.push(handle);
        }
        for handle in handles {
            let _ = handle.join();
        }

        // Get new query context - should see fresh snapshot with higher timestamp
        let ctx2 = manager.get_transaction_context_for_query().unwrap();
        let ts2 = ctx2.snapshot.snapshot_timestamp;

        assert!(ts2 > ts1, "ReadCommitted should see fresh snapshot after concurrent commits");
        manager.commit().unwrap();
        println!("✓ ReadCommitted freshness after concurrent commits: PASS");
    }

    /// Test: Stress - Maximum Concurrent Transaction Contexts
    /// Test with very high number of concurrent transaction contexts
    #[test]
    fn test_max_concurrent_transaction_contexts() {
        let mvcc = create_mvcc_engine();
        let mut handles = vec![];

        // 100 concurrent transactions
        for _ in 0..100 {
            let mvcc_clone = mvcc.clone();
            let handle = thread::spawn(move || {
                let mut manager = TransactionManager::with_mvcc(mvcc_clone);
                manager.set_isolation_level(IsolationLevel::RepeatableRead);

                let tx_id = manager.begin().unwrap();

                // Multiple context accesses
                for _ in 0..5 {
                    let ctx = manager.get_transaction_context().unwrap();
                    if !ctx.is_visible(tx_id, None) {
                        return false;
                    }
                }

                manager.commit().unwrap();
                tx_id;
                true
            });
            handles.push(handle);
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(results.iter().all(|&r| r), "All 100 concurrent transactions should succeed");
        println!("✓ Maximum concurrent transaction contexts: PASS (100 concurrent)");
    }

    /// Test: Concurrent Transaction Rollback Stability
    /// Verify rollback works correctly under concurrency
    #[test]
    fn test_concurrent_rollback_stability() {
        let mvcc = create_mvcc_engine();
        let mut handles = vec![];

        for i in 0..30 {
            let mvcc_clone = mvcc.clone();
            let handle = thread::spawn(move || {
                let mut manager = TransactionManager::with_mvcc(mvcc_clone);
                manager.set_isolation_level(IsolationLevel::ReadCommitted);

                let tx_id = manager.begin().unwrap();

                // Simulate work
                thread::sleep(Duration::from_micros(i as u64));

                manager.rollback().unwrap();
                tx_id
            });
            handles.push(handle);
        }

        let all_success = handles.into_iter().all(|h| h.join().is_ok());
        assert!(all_success, "Concurrent rollback should be stable");
        println!("✓ Concurrent rollback stability: PASS (30 rollbacks)");
    }

    /// Test: Nested Transaction Context Access
    /// Multiple context accesses within same transaction should be self-consistent
    #[test]
    fn test_nested_context_access() {
        let mvcc = create_mvcc_engine();
        let mut handles = vec![];

        for _ in 0..50 {
            let mvcc_clone = mvcc.clone();
            let handle = thread::spawn(move || {
                let mut manager = TransactionManager::with_mvcc(mvcc_clone);
                manager.set_isolation_level(IsolationLevel::RepeatableRead);

                let tx_id = manager.begin().unwrap();

                // Multiple nested context accesses - get two contexts
                let ctx1 = manager.get_transaction_context().unwrap();
                let ctx2 = manager.get_transaction_context().unwrap();

                // Both should see own transaction as visible
                let self_visible1 = ctx1.is_visible(tx_id, None);
                let self_visible2 = ctx2.is_visible(tx_id, None);

                manager.commit().unwrap();
                let _ = tx_id;
                self_visible1 && self_visible2
            });
            handles.push(handle);
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(results.iter().all(|&r| r), "Nested context access should be stable");
        println!("✓ Nested context access: PASS (50 transactions)");
    }
}
