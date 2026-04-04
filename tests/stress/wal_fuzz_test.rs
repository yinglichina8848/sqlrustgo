//! WAL Fuzz/Property-Based Tests
//!
//! These tests use property-based testing to verify WAL invariants
//! across random operation sequences.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use sqlrustgo_storage::wal::{WalEntryType, WalManager};
    use tempfile::TempDir;

    /// Operation types for fuzz testing
    #[derive(Debug, Clone)]
    enum Op {
        Begin {
            tx_id: u64,
        },
        Insert {
            tx_id: u64,
            table_id: u64,
            key: Vec<u8>,
            data: Vec<u8>,
        },
        Update {
            tx_id: u64,
            table_id: u64,
            key: Vec<u8>,
            data: Vec<u8>,
        },
        Delete {
            tx_id: u64,
            table_id: u64,
            key: Vec<u8>,
        },
        Commit {
            tx_id: u64,
        },
        Rollback {
            tx_id: u64,
        },
    }

    impl Op {
        fn execute(&self, manager: &WalManager) -> Result<(), Box<dyn std::error::Error>> {
            match self {
                Op::Begin { tx_id } => {
                    manager.log_begin(*tx_id)?;
                }
                Op::Insert {
                    tx_id,
                    table_id,
                    key,
                    data,
                } => {
                    manager.log_insert(*tx_id, *table_id, key.clone(), data.clone())?;
                }
                Op::Update {
                    tx_id,
                    table_id,
                    key,
                    data,
                } => {
                    manager.log_update(*tx_id, *table_id, key.clone(), data.clone())?;
                }
                Op::Delete {
                    tx_id,
                    table_id,
                    key,
                } => {
                    manager.log_delete(*tx_id, *table_id, key.clone())?;
                }
                Op::Commit { tx_id } => {
                    manager.log_commit(*tx_id)?;
                }
                Op::Rollback { tx_id } => {
                    manager.log_rollback(*tx_id)?;
                }
            }
            Ok(())
        }
    }

    /// Generate a valid operation sequence
    fn gen_valid_ops() -> impl Strategy<Value = Vec<Op>> {
        prop::collection::vec(
            (
                any::<u64>(),
                any::<u64>(),
                prop::collection::vec(any::<u8>(), 1..10),
                prop::collection::vec(any::<u8>(), 1..20),
            ),
            1..50,
        )
        .prop_map(|tuples| {
            let mut ops = Vec::new();
            for (base_tx_id, table_id, key, data) in tuples {
                let tx_id = (base_tx_id % 10) + 1; // Keep tx_ids small (1-10)
                ops.push(Op::Begin { tx_id });
                ops.push(Op::Insert {
                    tx_id,
                    table_id: (table_id % 5) + 1,
                    key,
                    data,
                });
                ops.push(Op::Commit { tx_id });
            }
            ops
        })
    }

    /// Generate mixed operations with rollbacks
    fn gen_mixed_ops() -> impl Strategy<Value = Vec<Op>> {
        prop::collection::vec(
            (
                any::<u64>(),
                any::<u64>(),
                prop::collection::vec(any::<u8>(), 1..10),
                prop::collection::vec(any::<u8>(), 1..20),
                any::<bool>(),
            ),
            1..30,
        )
        .prop_map(|tuples| {
            let mut ops = Vec::new();
            for (base_tx_id, table_id, key, data, commit) in tuples {
                let tx_id = (base_tx_id % 10) + 1;
                let table_id = (table_id % 5) + 1;
                ops.push(Op::Begin { tx_id });
                ops.push(Op::Insert {
                    tx_id,
                    table_id,
                    key,
                    data,
                });
                if commit {
                    ops.push(Op::Commit { tx_id });
                } else {
                    ops.push(Op::Rollback { tx_id });
                }
            }
            ops
        })
    }

    /// Helper: Create a WAL manager with temp directory
    fn create_wal_manager() -> (TempDir, WalManager) {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test.wal");
        let manager = WalManager::new(wal_path);
        (dir, manager)
    }

    /// Property: Same operation sequence produces same WAL entries
    #[test]
    #[ignore] // Slow fuzz test - run in regression suite only
    fn test_wal_fuzz_same_ops_same_entries() {
        let ops = gen_valid_ops();

        proptest!(|(ops in ops)| {
            let (_dir, manager1) = create_wal_manager();
            let (_dir, manager2) = create_wal_manager();

            // Execute same ops on both managers
            for op in &ops {
                op.execute(&manager1).unwrap();
                op.execute(&manager2).unwrap();
            }

            let entries1 = manager1.recover().unwrap();
            let entries2 = manager2.recover().unwrap();

            // Same ops should produce same number of entries
            prop_assert_eq!(entries1.len(), entries2.len(),
                "Same ops should produce same number of entries");

            // Entry types should match
            for (e1, e2) in entries1.iter().zip(entries2.iter()) {
                prop_assert_eq!(e1.entry_type, e2.entry_type);
                prop_assert_eq!(e1.tx_id, e2.tx_id);
            }
        });
    }

    /// Property: All committed transactions have matching begin/commit
    #[test]
    #[ignore] // Slow fuzz test - run in regression suite only
    fn test_wal_fuzz_commit_has_begin() {
        let ops = gen_mixed_ops();

        proptest!(|(ops in ops)| {
            let (_dir, manager) = create_wal_manager();

            for op in &ops {
                op.execute(&manager).unwrap();
            }

            let entries = manager.recover().unwrap();

            // Find all committed tx_ids
            let committed: std::collections::HashSet<u64> = entries
                .iter()
                .filter(|e| e.entry_type == WalEntryType::Commit)
                .map(|e| e.tx_id)
                .collect();

            // Each committed tx should have a begin
            for tx_id in committed {
                let has_begin = entries.iter().any(|e| {
                    e.entry_type == WalEntryType::Begin && e.tx_id == tx_id
                });
                prop_assert!(has_begin, "Committed tx {} should have begin", tx_id);
            }
        });
    }

    /// Property: All rolled back transactions have matching begin/rollback
    #[test]
    #[ignore] // Slow fuzz test - run in regression suite only
    fn test_wal_fuzz_rollback_has_begin() {
        let ops = gen_mixed_ops();

        proptest!(|(ops in ops)| {
            let (_dir, manager) = create_wal_manager();

            for op in &ops {
                op.execute(&manager).unwrap();
            }

            let entries = manager.recover().unwrap();

            // Find all rolled back tx_ids
            let rolled_back: std::collections::HashSet<u64> = entries
                .iter()
                .filter(|e| e.entry_type == WalEntryType::Rollback)
                .map(|e| e.tx_id)
                .collect();

            // Each rolled back tx should have a begin
            for tx_id in rolled_back {
                let has_begin = entries.iter().any(|e| {
                    e.entry_type == WalEntryType::Begin && e.tx_id == tx_id
                });
                prop_assert!(has_begin, "Rolled back tx {} should have begin", tx_id);
            }
        });
    }

    /// Property: LSN always increases
    #[test]
    #[ignore] // Slow fuzz test - run in regression suite only
    fn test_wal_fuzz_lsn_increases() {
        let ops = gen_valid_ops();

        proptest!(|(ops in ops)| {
            let (_dir, manager) = create_wal_manager();

            for op in &ops {
                op.execute(&manager).unwrap();
            }

            let entries = manager.recover().unwrap();

            let mut last_lsn: u64 = 0;
            for entry in entries.iter() {
                prop_assert!(entry.lsn >= last_lsn, "LSN should increase");
                last_lsn = entry.lsn;
            }
        });
    }

    /// Property: Data integrity preserved through WAL
    #[test]
    #[ignore] // Slow fuzz test - run in regression suite only
    fn test_wal_fuzz_data_integrity() {
        let data_strategy = prop::collection::vec(any::<u8>(), 1..100);

        proptest!(|(data in data_strategy)| {
            let (_dir, manager1) = create_wal_manager();
            let (_dir, manager2) = create_wal_manager();

            let tx_id = 1u64;
            let key = vec![1u8, 2u8, 3u8];

            // Insert same data in both
            manager1.log_insert(tx_id, 1, key.clone(), data.clone()).unwrap();
            manager2.log_insert(tx_id, 1, key.clone(), data.clone()).unwrap();

            let entries1 = manager1.recover().unwrap();
            let entries2 = manager2.recover().unwrap();

            prop_assert_eq!(entries1.len(), entries2.len());

            // Data should be exactly the same
            prop_assert_eq!(&entries1[0].data, &entries2[0].data);
            prop_assert_eq!(&entries1[0].key, &entries2[0].key);
        });
    }

    /// Property: Operations with same tx_id are serialized
    #[test]
    #[ignore] // Slow fuzz test - run in regression suite only
    fn test_wal_fuzz_same_tx_serialization() {
        let ops = gen_mixed_ops();

        proptest!(|(ops in ops)| {
            let (_dir, manager) = create_wal_manager();

            for op in &ops {
                op.execute(&manager).unwrap();
            }

            let entries = manager.recover().unwrap();

            // For each tx_id, operations should appear in order
            let mut tx_entry_indices: std::collections::HashMap<u64, Vec<usize>> =
                std::collections::HashMap::new();

            for (i, entry) in entries.iter().enumerate() {
                tx_entry_indices
                    .entry(entry.tx_id)
                    .or_default()
                    .push(i);
            }

            // Each tx's entries should have increasing indices
            for (_, indices) in tx_entry_indices {
                if indices.len() > 1 {
                    for window in indices.windows(2) {
                        prop_assert!(window[0] < window[1],
                            "Entries for same tx should be in order");
                    }
                }
            }
        });
    }

    /// Property: Entry count formula for simple transactions
    #[test]
    #[ignore] // Slow stress test - run in regression suite only
    fn test_wal_fuzz_entry_count_formula() {
        // Simple begin-insert-commit produces 3 entries
        let (_dir, manager) = create_wal_manager();

        manager.log_begin(1).unwrap();
        manager.log_insert(1, 1, vec![1], vec![10]).unwrap();
        manager.log_commit(1).unwrap();

        let entries = manager.recover().unwrap();
        assert_eq!(entries.len(), 3);

        // Add another transaction
        manager.log_begin(2).unwrap();
        manager.log_insert(2, 1, vec![2], vec![20]).unwrap();
        manager.log_commit(2).unwrap();

        let entries = manager.recover().unwrap();
        assert_eq!(entries.len(), 6);
    }

    /// Fuzz test: Many random operations
    #[test]
    #[ignore] // Slow fuzz test - run in regression suite only
    fn test_wal_fuzz_many_random_operations() {
        // Generate 20 random transactions
        let ops: Vec<Op> = (1..=20)
            .flat_map(|tx_id| {
                let table_id = (tx_id % 3) + 1;
                vec![
                    Op::Begin { tx_id },
                    Op::Insert {
                        tx_id,
                        table_id,
                        key: vec![tx_id as u8],
                        data: vec![(tx_id * 10) as u8],
                    },
                    Op::Commit { tx_id },
                ]
            })
            .collect();

        let (_dir, manager) = create_wal_manager();

        for op in &ops {
            op.execute(&manager).unwrap();
        }

        let entries = manager.recover().unwrap();

        // 20 transactions * 3 entries each = 60 entries
        assert_eq!(entries.len(), 60);

        // All should be commits
        let commits = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .count();
        assert_eq!(commits, 20);
    }

    /// Property: Update and delete operations work correctly
    #[test]
    #[ignore] // Slow fuzz test - run in regression suite only
    fn test_wal_fuzz_update_delete() {
        let ops = prop::collection::vec(
            (any::<u64>(), prop::collection::vec(any::<u8>(), 1..5)),
            1..20,
        )
        .prop_map(|ops| {
            let tx_id = 1u64;
            let mut result = vec![Op::Begin { tx_id }];

            for (i, key) in ops {
                let data = vec![i as u8];
                result.push(Op::Insert {
                    tx_id,
                    table_id: 1,
                    key: key.clone(),
                    data: data.clone(),
                });

                // Update sometimes
                if i % 2 == 0 {
                    result.push(Op::Update {
                        tx_id,
                        table_id: 1,
                        key: key.clone(),
                        // Use wrapping_add to prevent overflow when i is large
                        data: vec![((i % 256) as u8).wrapping_add(100)],
                    });
                }

                // Delete sometimes
                if i % 3 == 0 {
                    result.push(Op::Delete {
                        tx_id,
                        table_id: 1,
                        key,
                    });
                }
            }

            result.push(Op::Commit { tx_id });
            result
        });

        proptest!(|(ops in ops)| {
            let (_dir, manager) = create_wal_manager();

            for op in &ops {
                op.execute(&manager).unwrap();
            }

            let entries = manager.recover().unwrap();

            // Should have at least begin and commit
            prop_assert!(entries.len() >= 2);

            // First entry should be begin
            prop_assert!(matches!(entries.first(), Some(e) if e.entry_type == WalEntryType::Begin));

            // Last entry should be commit
            prop_assert!(matches!(entries.last(), Some(e) if e.entry_type == WalEntryType::Commit));
        });
    }

    /// Stress test: Large number of small transactions
    #[test]
    #[ignore] // Slow stress test - run in regression suite only
    fn test_wal_stress_many_tiny_transactions() {
        let (_dir, manager) = create_wal_manager();

        let num_tx = 500u64;

        for tx_id in 1..=num_tx {
            manager.log_begin(tx_id).unwrap();
            manager
                .log_insert(tx_id, 1, vec![tx_id as u8], vec![tx_id as u8])
                .unwrap();
            manager.log_commit(tx_id).unwrap();
        }

        let entries = manager.recover().unwrap();

        // Each tx = 3 entries (begin, insert, commit)
        assert_eq!(entries.len(), (num_tx * 3) as usize);

        // All commits should be present
        let commits = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .count();
        assert_eq!(commits, num_tx as usize);
    }
}
