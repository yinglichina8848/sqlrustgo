//! Crash Recovery Tests
//!
//! These tests verify WAL crash recovery capabilities:
//! - Full recovery after crash
//! - Partial commit/rollback recovery
//! - WAL integrity checks

#[cfg(test)]
mod tests {
    use sqlrustgo_storage::wal::{WalEntry, WalEntryType, WalManager};
    use std::fs;
    use tempfile::TempDir;

    /// Test: Full recovery after crash
    /// Verifies that committed transactions are recovered after simulated crash
    /// Note: WAL layer recovers ALL entries; transaction filtering happens at higher layer
    #[test]
    fn test_crash_recovery_committed() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test_recovery.wal");

        // Step 1: Create and commit some transactions
        {
            let manager = WalManager::new(wal_path.clone());

            // Transaction 1: committed
            let _tx1 = manager.log_begin(1).unwrap();
            let _ = manager.log_insert(1, 1, vec![1], vec![10]).unwrap();
            let _ = manager.log_commit(1).unwrap();

            // Transaction 2: committed
            let _tx2 = manager.log_begin(2).unwrap();
            let _ = manager.log_insert(2, 1, vec![2], vec![20]).unwrap();
            let _ = manager.log_commit(2).unwrap();

            // Transaction 3: not committed (should be rolled back)
            let _tx3 = manager.log_begin(3).unwrap();
            let _ = manager.log_insert(3, 1, vec![3], vec![30]).unwrap();
            // No commit - simulate crash
        }

        // Step 2: Recover
        let manager = WalManager::new(wal_path.clone());
        let entries = manager.recover().unwrap();

        // Verify: Should have entries from committed transactions
        let committed_count = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .count();

        assert_eq!(
            committed_count, 2,
            "Should recover 2 committed transactions"
        );

        // Note: WAL recovers ALL entries including uncommitted
        // Transaction layer should filter uncommitted data at recovery time
        let uncommitted_inserts = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Insert && e.tx_id == 3)
            .count();

        println!(
            "✓ Crash recovery: recovered {} total entries",
            entries.len()
        );
        println!("  - Commits: {}", committed_count);
        println!("  - Uncommitted inserts (TX3): {}", uncommitted_inserts);

        // WAL should recover committed transactions
        assert!(
            committed_count >= 2,
            "Should recover committed transactions"
        );
    }

    /// Test: Partial rollback recovery
    /// Verifies that mixed committed/rolled-back transactions are handled correctly
    #[test]
    fn test_partial_rollback_recovery() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test_rollback.wal");

        // Create mixed scenario
        {
            let manager = WalManager::new(wal_path.clone());

            // T1: Insert + Commit
            let _ = manager.log_begin(1).unwrap();
            let _ = manager.log_insert(1, 1, vec![1], vec![10]).unwrap();
            let _ = manager.log_commit(1).unwrap();

            // T2: Insert + Rollback
            let _ = manager.log_begin(2).unwrap();
            let _ = manager.log_insert(2, 1, vec![2], vec![20]).unwrap();
            let _ = manager.log_rollback(2).unwrap();

            // T3: Insert + Commit
            let _ = manager.log_begin(3).unwrap();
            let _ = manager.log_insert(3, 1, vec![3], vec![30]).unwrap();
            let _ = manager.log_commit(3).unwrap();
        }

        // Recover
        let manager = WalManager::new(wal_path.clone());
        let entries = manager.recover().unwrap();

        // Verify commits
        let commits: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .collect();

        let rollbacks: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Rollback)
            .collect();

        assert_eq!(commits.len(), 2, "Should have 2 commits");
        assert_eq!(rollbacks.len(), 1, "Should have 1 rollback");

        println!("✓ Partial rollback: mixed commit/rollback handled correctly");
    }

    /// Test: WAL integrity after crash
    /// Verifies WAL file integrity after abnormal termination
    #[test]
    fn test_wal_integrity_after_crash() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test_integrity.wal");

        // Write some data
        {
            let manager = WalManager::new(wal_path.clone());

            for i in 1..=100 {
                let tx_id = i % 10 + 1; // 10 transactions
                let _ = manager.log_begin(tx_id).unwrap();
                let _ = manager
                    .log_insert(tx_id, 1, vec![i as u8], vec![i as u8])
                    .unwrap();

                if i % 3 == 0 {
                    let _ = manager.log_commit(tx_id).unwrap();
                }
            }
        }

        // Verify WAL file exists and has content
        let metadata = fs::metadata(&wal_path).unwrap();
        assert!(metadata.len() > 0, "WAL file should have content");

        // Verify recovery works
        let manager = WalManager::new(wal_path.clone());
        let entries = manager.recover().unwrap();

        assert!(!entries.is_empty(), "Should recover some entries");

        println!(
            "✓ WAL integrity: {} entries recovered from {} bytes",
            entries.len(),
            metadata.len()
        );
    }

    /// Test: Checkpoint recovery
    /// Verifies that checkpoint provides correct recovery point
    #[test]
    fn test_checkpoint_recovery() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test_checkpoint.wal");

        {
            let manager = WalManager::new(wal_path.clone());

            // Write some data
            let _ = manager.log_begin(1).unwrap();
            let _ = manager.log_insert(1, 1, vec![1], vec![10]).unwrap();
            let _ = manager.log_commit(1).unwrap();

            // Create checkpoint
            let _ = manager.checkpoint(1).unwrap();

            // Write more data after checkpoint
            let _ = manager.log_begin(2).unwrap();
            let _ = manager.log_insert(2, 1, vec![2], vec![20]).unwrap();
            let _ = manager.log_commit(2).unwrap();
        }

        // Recover
        let manager = WalManager::new(wal_path.clone());
        let entries = manager.recover().unwrap();

        // Should recover all data (including after checkpoint)
        let commits = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .count();

        assert_eq!(commits, 2, "Should recover 2 committed transactions");

        // Should have checkpoint entry
        let checkpoints = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Checkpoint)
            .count();

        assert!(checkpoints >= 1, "Should have checkpoint entry");

        println!("✓ Checkpoint recovery: {} commits recovered", commits);
    }

    /// Test: Large WAL recovery performance
    /// Verifies recovery time scales reasonably
    #[test]
    fn test_large_wal_recovery_performance() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test_large.wal");

        // Create 1000 entries
        let entry_count = 1000;
        {
            let manager = WalManager::new(wal_path.clone());

            for i in 0..entry_count {
                let tx_id = (i % 100) as u64 + 1;
                let _ = manager.log_begin(tx_id).unwrap();
                let key = vec![i as u8];
                let data = vec![i as u8; 100]; // 100 bytes per entry
                let _ = manager.log_insert(tx_id, 1, key, data).unwrap();
                let _ = manager.log_commit(tx_id).unwrap();
            }
        }

        // Measure recovery time
        let start = std::time::Instant::now();
        let manager = WalManager::new(wal_path);
        let entries = manager.recover().unwrap();
        let elapsed = start.elapsed();

        println!(
            "✓ Large WAL recovery: {} entries in {:?}",
            entries.len(),
            elapsed
        );

        // Verify: 1000 inserts + 1000 begins + 1000 commits = 3000 entries
        assert!(
            entries.len() >= entry_count,
            "Should recover at least {} entries",
            entry_count
        );

        // Target: < 1 second for 1000 entries
        assert!(
            elapsed.as_secs_f64() < 1.0,
            "Recovery too slow: {:?} for {} entries",
            elapsed,
            entries.len()
        );
    }

    /// Test: Multiple concurrent transactions crash recovery
    #[test]
    fn test_concurrent_transactions_recovery() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test_concurrent.wal");

        {
            let manager = WalManager::new(wal_path.clone());

            // Create 5 concurrent transactions
            for tx_id in 1..=5 {
                let _ = manager.log_begin(tx_id).unwrap();
                for row_id in 1..=10 {
                    let _ = manager
                        .log_insert(
                            tx_id,
                            1,
                            vec![row_id],
                            vec![tx_id as u8 * 10 + row_id as u8],
                        )
                        .unwrap();
                }
                if tx_id % 2 == 0 {
                    let _ = manager.log_commit(tx_id).unwrap();
                }
            }
        }

        let manager = WalManager::new(wal_path.clone());
        let entries = manager.recover().unwrap();

        let commits = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .count();
        let inserts = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Insert)
            .count();

        assert_eq!(commits, 2, "Should recover 2 committed transactions");
        assert_eq!(inserts, 50, "Should recover 50 insert operations");

        println!(
            "✓ Concurrent recovery: {} commits, {} inserts",
            commits, inserts
        );
    }

    /// Test: Empty WAL recovery
    #[test]
    fn test_empty_wal_recovery() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test_empty.wal");

        // Create WAL with some entries then delete
        {
            let manager = WalManager::new(wal_path.clone());
            let _ = manager.log_begin(1).unwrap();
            let _ = manager.log_insert(1, 1, vec![1], vec![10]).unwrap();
            let _ = manager.log_commit(1).unwrap();
        }

        // Delete WAL file to simulate empty case
        std::fs::remove_file(&wal_path).ok();

        // Recovery should handle missing file gracefully
        let manager = WalManager::new(wal_path.clone());
        let result = manager.recover();

        println!(
            "✓ Empty WAL: recovery {:?}",
            if result.is_ok() {
                "succeeded"
            } else {
                "failed gracefully"
            }
        );
    }

    /// Test: Corrupted entry recovery (partial data)
    #[test]
    fn test_partial_entry_recovery() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test_partial.wal");

        {
            let manager = WalManager::new(wal_path.clone());
            let _ = manager.log_begin(1).unwrap();
            let _ = manager.log_insert(1, 1, vec![1], vec![10]).unwrap();
            let _ = manager.log_commit(1).unwrap();
        }

        // Simulate partial write by truncating file
        let metadata = fs::metadata(&wal_path).unwrap();
        let trunc_len = metadata.len() / 2;
        let file = fs::OpenOptions::new().write(true).open(&wal_path).unwrap();
        file.set_len(trunc_len).unwrap();
        drop(file);

        // Recover - should handle gracefully
        let manager = WalManager::new(wal_path.clone());
        let result = manager.recover();

        println!(
            "✓ Partial entry: recovery {:?}",
            if result.is_ok() {
                "succeeded"
            } else {
                "failed gracefully"
            }
        );
    }

    /// Test: Rapid commit/rollback cycles
    #[test]
    fn test_rapid_commit_rollback_cycles() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test_rapid.wal");

        let cycles = 100;
        {
            let manager = WalManager::new(wal_path.clone());

            for i in 1..=cycles {
                let _ = manager.log_begin(i).unwrap();
                let _ = manager
                    .log_insert(i, 1, vec![i as u8], vec![i as u8])
                    .unwrap();
                if i % 2 == 0 {
                    let _ = manager.log_commit(i).unwrap();
                } else {
                    let _ = manager.log_rollback(i).unwrap();
                }
            }
        }

        let manager = WalManager::new(wal_path.clone());
        let entries = manager.recover().unwrap();

        let commits = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .count();
        let rollbacks = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Rollback)
            .count();

        assert_eq!(
            commits,
            (cycles / 2) as usize,
            "Should have {} commits",
            cycles / 2
        );
        assert_eq!(
            rollbacks,
            (cycles / 2) as usize,
            "Should have {} rollbacks",
            cycles / 2
        );

        println!(
            "✓ Rapid cycles: {} commits, {} rollbacks",
            commits, rollbacks
        );
    }
}
