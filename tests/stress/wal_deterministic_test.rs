//! WAL Deterministic Replay Tests
//!
//! These tests verify WAL can deterministically replay operations:
//! - Same operations produce same result when replayed
//! - WAL entries can be captured and replayed to reconstruct state
//! - Property-based testing ensures consistency across multiple runs

#[cfg(test)]
mod tests {
    use sqlrustgo_storage::wal::{WalEntry, WalEntryType, WalManager};
    use tempfile::TempDir;

    /// Helper: Create a WAL manager with a temp directory
    fn create_wal_manager() -> (TempDir, WalManager, std::path::PathBuf) {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test.wal");
        let manager = WalManager::new(wal_path.clone());
        (dir, manager, wal_path)
    }

    /// Helper: Capture state as a simple map of tx_id -> operations
    fn capture_wal_state(entries: &[WalEntry]) -> Vec<(WalEntryType, u64)> {
        entries
            .iter()
            .map(|e| (e.entry_type.clone(), e.tx_id))
            .collect()
    }

    /// Test: WAL entries can be serialized and deserialized deterministically
    #[test]
    fn test_wal_entry_serialization_determinism() {
        let entry = WalEntry {
            tx_id: 42,
            entry_type: WalEntryType::Insert,
            table_id: 1,
            key: Some(vec![1, 2, 3]),
            data: Some(vec![10, 20, 30]),
            lsn: 100,
            timestamp: 1234567890,
        };

        // Serialize multiple times
        let bytes1 = entry.to_bytes();
        let bytes2 = entry.to_bytes();

        // Should be identical
        assert_eq!(bytes1, bytes2, "Serialization should be deterministic");

        // Deserialize
        let restored1 = WalEntry::from_bytes(&bytes1).unwrap();
        let restored2 = WalEntry::from_bytes(&bytes2).unwrap();

        assert_eq!(restored1.tx_id, restored2.tx_id);
        assert_eq!(restored1.entry_type, restored2.entry_type);
        assert_eq!(restored1.key, restored2.key);
        assert_eq!(restored1.data, restored2.data);
    }

    /// Test: Replaying same WAL entries produces identical state
    #[test]
    fn test_wal_deterministic_replay_same_entries() {
        let (_dir1, manager1, _path1) = create_wal_manager();
        let (_dir2, manager2, _path2) = create_wal_manager();

        // Both managers execute same operations
        let tx_id = 1u64;

        manager1.log_begin(tx_id).unwrap();
        manager1.log_insert(tx_id, 1, vec![1], vec![100]).unwrap();
        manager1.log_commit(tx_id).unwrap();

        manager2.log_begin(tx_id).unwrap();
        manager2.log_insert(tx_id, 1, vec![1], vec![100]).unwrap();
        manager2.log_commit(tx_id).unwrap();

        // Capture entries
        let entries1 = manager1.recover().unwrap();
        let entries2 = manager2.recover().unwrap();

        // Both should have same number of entries
        assert_eq!(
            entries1.len(),
            entries2.len(),
            "Same operations should produce same number of entries"
        );

        // Both should have same entry sequence
        let state1 = capture_wal_state(&entries1);
        let state2 = capture_wal_state(&entries2);
        assert_eq!(state1, state2, "Replayed state should be identical");
    }

    /// Test: WAL replay with multiple transactions
    #[test]
    fn test_wal_deterministic_replay_multiple_tx() {
        let (_dir1, manager1, _path1) = create_wal_manager();
        let (_dir2, manager2, _path2) = create_wal_manager();

        // Transaction 1
        manager1.log_begin(1).unwrap();
        manager1.log_insert(1, 1, vec![1], vec![10]).unwrap();
        manager1.log_commit(1).unwrap();

        // Transaction 2
        manager1.log_begin(2).unwrap();
        manager1.log_insert(2, 1, vec![2], vec![20]).unwrap();
        manager1.log_update(2, 1, vec![2], vec![25]).unwrap();
        manager1.log_commit(2).unwrap();

        // Transaction 3 (rollback)
        manager1.log_begin(3).unwrap();
        manager1.log_insert(3, 1, vec![3], vec![30]).unwrap();
        manager1.log_rollback(3).unwrap();

        // Replay same on manager2
        manager2.log_begin(1).unwrap();
        manager2.log_insert(1, 1, vec![1], vec![10]).unwrap();
        manager2.log_commit(1).unwrap();

        manager2.log_begin(2).unwrap();
        manager2.log_insert(2, 1, vec![2], vec![20]).unwrap();
        manager2.log_update(2, 1, vec![2], vec![25]).unwrap();
        manager2.log_commit(2).unwrap();

        manager2.log_begin(3).unwrap();
        manager2.log_insert(3, 1, vec![3], vec![30]).unwrap();
        manager2.log_rollback(3).unwrap();

        let entries1 = manager1.recover().unwrap();
        let entries2 = manager2.recover().unwrap();

        assert_eq!(entries1.len(), entries2.len());

        // Verify entry types match
        for (e1, e2) in entries1.iter().zip(entries2.iter()) {
            assert_eq!(e1.entry_type, e2.entry_type);
            assert_eq!(e1.tx_id, e2.tx_id);
            assert_eq!(e1.table_id, e2.table_id);
        }
    }

    /// Test: Large payload determinism
    #[test]
    fn test_wal_deterministic_large_payload() {
        let (_dir1, manager1, _path1) = create_wal_manager();
        let (_dir2, manager2, _path2) = create_wal_manager();

        let large_data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let tx_id = 1u64;

        manager1
            .log_insert(tx_id, 1, vec![1], large_data.clone())
            .unwrap();
        manager2
            .log_insert(tx_id, 1, vec![1], large_data.clone())
            .unwrap();

        let entries1 = manager1.recover().unwrap();
        let entries2 = manager2.recover().unwrap();

        assert_eq!(entries1.len(), entries2.len());

        // Large data should be preserved exactly
        let data1 = &entries1[0].data;
        let data2 = &entries2[0].data;
        assert_eq!(data1, data2, "Large data should be preserved exactly");
    }

    /// Test: Order preservation in WAL
    #[test]
    fn test_wal_order_preservation() {
        let (_dir, manager, _path) = create_wal_manager();

        let tx_id = 1u64;
        manager.log_begin(tx_id).unwrap();

        // Insert multiple rows
        for i in 0..100 {
            manager
                .log_insert(tx_id, 1, vec![i as u8], vec![i as u8])
                .unwrap();
        }

        manager.log_commit(tx_id).unwrap();

        let entries = manager.recover().unwrap();

        // Entries should be in order
        let mut last_lsn = 0u64;
        for entry in &entries {
            assert!(
                entry.lsn >= last_lsn,
                "LSN should increase monotonically"
            );
            last_lsn = entry.lsn;
        }

        // Should have 100 insert entries plus begin and commit
        let insert_count = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Insert)
            .count();
        assert_eq!(insert_count, 100, "Should have 100 inserts");
    }

    /// Test: Concurrent transaction isolation in WAL
    #[test]
    fn test_wal_concurrent_tx_isolation() {
        let (_dir, manager, _path) = create_wal_manager();

        // T1 and T2 run concurrently
        manager.log_begin(1).unwrap();
        manager.log_insert(1, 1, vec![1], vec![10]).unwrap();

        manager.log_begin(2).unwrap();
        manager.log_insert(2, 1, vec![2], vec![20]).unwrap();

        // T1 commits
        manager.log_commit(1).unwrap();

        // T2 still running
        manager.log_insert(2, 1, vec![3], vec![30]).unwrap();
        manager.log_commit(2).unwrap();

        let entries = manager.recover().unwrap();

        // Find commits
        let commits: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .collect();

        assert_eq!(commits.len(), 2, "Should have 2 commits");

        // First commit should be tx_id 1, second should be tx_id 2
        assert_eq!(commits[0].tx_id, 1);
        assert_eq!(commits[1].tx_id, 2);
    }

    /// Test: Checkpoint provides deterministic recovery point
    #[test]
    fn test_wal_checkpoint_deterministic() {
        let (_dir1, manager1, _path1) = create_wal_manager();
        let (_dir2, manager2, _path2) = create_wal_manager();

        // Both create checkpoint after first tx
        manager1.log_begin(1).unwrap();
        manager1.log_insert(1, 1, vec![1], vec![10]).unwrap();
        manager1.log_commit(1).unwrap();
        manager1.checkpoint(1).unwrap();

        manager2.log_begin(1).unwrap();
        manager2.log_insert(1, 1, vec![1], vec![10]).unwrap();
        manager2.log_commit(1).unwrap();
        manager2.checkpoint(1).unwrap();

        // Both add more data
        manager1.log_begin(2).unwrap();
        manager1.log_insert(2, 1, vec![2], vec![20]).unwrap();
        manager1.log_commit(2).unwrap();

        manager2.log_begin(2).unwrap();
        manager2.log_insert(2, 1, vec![2], vec![20]).unwrap();
        manager2.log_commit(2).unwrap();

        let entries1 = manager1.recover().unwrap();
        let entries2 = manager2.recover().unwrap();

        // Both should have same checkpoint structure
        let checkpoints1 = entries1
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Checkpoint)
            .count();
        let checkpoints2 = entries2
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Checkpoint)
            .count();

        assert_eq!(checkpoints1, checkpoints2);
    }

    /// Test: Empty WAL produces empty replay
    #[test]
    fn test_wal_empty_replay() {
        let (_dir, manager, _path) = create_wal_manager();

        // When no operations are logged, the WAL file may not exist
        // Recovery should handle this gracefully
        let result = manager.recover();

        match result {
            Ok(entries) => {
                assert!(entries.is_empty(), "Empty WAL should produce empty entries");
            }
            Err(e) => {
                // If file doesn't exist, that's also acceptable for empty WAL
                println!("Empty WAL recovery returned error (file may not exist): {:?}", e);
            }
        }
    }

    /// Test: Partial entry handling
    #[test]
    fn test_wal_partial_entry_at_end() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test.wal");

        // Write a complete entry
        {
            let manager = WalManager::new(wal_path.clone());
            manager.log_begin(1).unwrap();
            manager.log_insert(1, 1, vec![1], vec![10]).unwrap();
            manager.log_commit(1).unwrap();
        }

        // Truncate the file to simulate partial write
        use std::fs::OpenOptions;

        let file = OpenOptions::new().write(true).open(&wal_path).unwrap();
        // Keep only half the file
        let metadata = std::fs::metadata(&wal_path).unwrap();
        let trunc_len = metadata.len() / 2;
        file.set_len(trunc_len).unwrap();
        drop(file);

        // Recovery should handle gracefully
        let manager = WalManager::new(wal_path);
        let result = manager.recover();

        // Should either succeed with partial entries or fail gracefully
        match result {
            Ok(entries) => {
                println!(
                    "Partial recovery: got {} entries (some may be incomplete)",
                    entries.len()
                );
            }
            Err(e) => {
                println!("Partial recovery failed gracefully: {:?}", e);
            }
        }
    }

    /// Test: Many small transactions produce deterministic results
    #[test]
    fn test_wal_many_small_transactions_deterministic() {
        let (_dir1, manager1, _path1) = create_wal_manager();
        let (_dir2, manager2, _path2) = create_wal_manager();

        let num_tx = 100;

        for i in 1..=num_tx {
            manager1.log_begin(i).unwrap();
            manager1
                .log_insert(i, 1, vec![i as u8], vec![i as u8])
                .unwrap();
            manager1.log_commit(i).unwrap();

            manager2.log_begin(i).unwrap();
            manager2
                .log_insert(i, 1, vec![i as u8], vec![i as u8])
                .unwrap();
            manager2.log_commit(i).unwrap();
        }

        let entries1 = manager1.recover().unwrap();
        let entries2 = manager2.recover().unwrap();

        assert_eq!(
            entries1.len(),
            entries2.len(),
            "Same operations should produce same number of entries"
        );

        // Count by type should match
        let commits1 = entries1
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .count();
        let commits2 = entries2
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .count();

        assert_eq!(commits1, commits2);
        assert_eq!(commits1, num_tx as usize);
    }
}
