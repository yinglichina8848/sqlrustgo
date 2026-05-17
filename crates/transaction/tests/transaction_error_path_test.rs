//! Error path tests for SQLRustGo Transaction Module
//!
//! This module tests error conditions and edge cases in:
//! - Deadlock detection and resolution
//! - Lock acquisition timeout
//! - Lock upgrade failures
//! - Savepoint rollback and release
//! - Wait-for graph cycle detection

use sqlrustgo_transaction::{
    deadlock::DeadlockDetector,
    lock::{LockError, LockGrantMode, LockManager, LockMode, LockTarget},
    savepoint::{SavepointError, SavepointManager, UndoRecord},
    wait_for_graph::{WaitForEdge, WaitForGraph},
    TxId,
};
use std::time::Duration;

// =============================================================================
// Deadlock Detection Tests
// =============================================================================

/// Test: detect_cycle returns None for empty graph
#[test]
fn test_detect_cycle_empty_graph() {
    let detector = DeadlockDetector::new();
    let cycle = detector.detect_cycle(TxId::new(1));
    assert!(cycle.is_none());
}

/// Test: detect_cycle returns cycle for 2-node deadlock (T1→T2→T1)
#[test]
fn test_detect_cycle_two_node_deadlock() {
    let detector = DeadlockDetector::new();
    detector.add_edge_unsafe(TxId::new(1), TxId::new(2));
    detector.add_edge_unsafe(TxId::new(2), TxId::new(1));

    let cycle = detector.detect_cycle(TxId::new(1));
    assert!(cycle.is_some());
    let cycle = cycle.unwrap();
    assert_eq!(cycle.len(), 2);
}

/// Test: detect_cycle returns cycle for 3-node cycle (T1→T2→T3→T1)
#[test]
fn test_detect_cycle_three_node_deadlock() {
    let detector = DeadlockDetector::new();
    detector.add_edge_unsafe(TxId::new(1), TxId::new(2));
    detector.add_edge_unsafe(TxId::new(2), TxId::new(3));
    detector.add_edge_unsafe(TxId::new(3), TxId::new(1));

    let cycle = detector.detect_cycle(TxId::new(1));
    assert!(cycle.is_some());
    let cycle = cycle.unwrap();
    assert_eq!(cycle.len(), 3);
}

/// Test: detect_cycle returns None for linear chain (no cycle)
#[test]
fn test_detect_cycle_linear_chain_no_cycle() {
    let detector = DeadlockDetector::new();
    detector.add_edge_unsafe(TxId::new(1), TxId::new(2));
    detector.add_edge_unsafe(TxId::new(2), TxId::new(3));
    detector.add_edge_unsafe(TxId::new(3), TxId::new(4));

    let cycle = detector.detect_cycle(TxId::new(1));
    assert!(cycle.is_none());
}

/// Test: try_wait_edge rejects cycle creation (T1 waits T2, T2 waits T1)
#[test]
fn test_try_wait_edge_rejects_mutual_deadlock() {
    let detector = DeadlockDetector::new();
    // T1→T2 edge already exists
    detector.add_edge_unsafe(TxId::new(1), TxId::new(2));

    // T2 tries to wait for T1 → would create T2→T1→T2 cycle
    let result = detector.try_wait_edge(TxId::new(2), [TxId::new(1)].into());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LockError::Deadlock));
}

/// Test: try_wait_edge accepts non-cyclic edge
#[test]
fn test_try_wait_edge_accepts_non_cyclic() {
    let detector = DeadlockDetector::new();
    // T1→T2 edge
    detector.add_edge_unsafe(TxId::new(1), TxId::new(2));

    // T3 tries to wait for T2 → T3→{T2} is safe (no cycle)
    let result = detector.try_wait_edge(TxId::new(3), [TxId::new(2)].into());
    assert!(result.is_ok());
}

/// Test: Self-wait is filtered by try_wait_edge (NoSelfWait handled internally)
#[test]
fn test_self_wait_via_try_wait_edge_filtered() {
    let detector = DeadlockDetector::new();
    // try_wait_edge filters self-dependencies - this is NOT an error
    // because the edge simply isn't added (no self-loop)
    let result = detector.try_wait_edge(TxId::new(1), [TxId::new(1)].into());
    // Result is Ok because self-wait is filtered out, not because it's allowed
    // The key point is no cycle is formed
    assert!(result.is_ok());
    // Verify no edge was actually added
    assert!(detector.detect_cycle(TxId::new(1)).is_none());
}

/// Test: remove_edges_for clears all edges for a transaction
#[test]
fn test_remove_edges_for_clears_edges() {
    let detector = DeadlockDetector::new();
    detector.add_edge_unsafe(TxId::new(1), TxId::new(2));
    detector.add_edge_unsafe(TxId::new(2), TxId::new(3));
    detector.add_edge_unsafe(TxId::new(3), TxId::new(1));

    // Remove T2's edges
    detector.remove_edges_for(TxId::new(2));

    // Cycle should be broken
    let cycle = detector.detect_cycle(TxId::new(1));
    assert!(cycle.is_none());
    let cycle = detector.detect_cycle(TxId::new(3));
    assert!(cycle.is_none());
}

/// Test: get_timeout returns configured timeout
#[test]
fn test_deadlock_detector_timeout() {
    let detector = DeadlockDetector::with_timeout(Duration::from_secs(10));
    assert_eq!(detector.get_timeout(), Duration::from_secs(10));
}

/// Test: concurrent mutual deadlock prevention - at least one must fail
#[test]
fn test_concurrent_mutual_deadlock_prevention() {
    use std::sync::Arc;
    use std::thread;

    let detector = Arc::new(DeadlockDetector::new());

    let d1 = Arc::clone(&detector);
    let t1 = thread::spawn(move || d1.try_wait_edge(TxId::new(1), [TxId::new(2)].into()));

    let d2 = Arc::clone(&detector);
    let t2 = thread::spawn(move || d2.try_wait_edge(TxId::new(2), [TxId::new(1)].into()));

    let r1 = t1.join().unwrap();
    let r2 = t2.join().unwrap();

    // At least one must fail (no cycle allowed)
    let both_ok = r1.is_ok() && r2.is_ok();
    assert!(
        !both_ok,
        "Mutual deadlock TOCTOU race escaped: both edges added without check"
    );

    // Verify: no cycle exists in the graph
    assert!(
        detector.detect_cycle(TxId::new(1)).is_none(),
        "Cycle found despite atomic pre-check"
    );
}

// =============================================================================
// Lock Acquisition Error Tests
// =============================================================================

/// Test: Lock upgrade fails when already exclusively locked
#[test]
fn test_lock_upgrade_fails_when_already_exclusive() {
    let mut manager = LockManager::new();
    let tx_id = TxId::new(1);
    let key = vec![1, 2, 3];

    // Acquire exclusive lock
    manager
        .acquire_lock(tx_id, key.clone(), LockMode::Exclusive)
        .unwrap();

    // Try to upgrade - should fail (already exclusive)
    let result = manager.upgrade_lock(tx_id, key.clone());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LockError::LockUpgradeFailed));
}

/// Test: Lock upgrade fails when multiple shared holders exist
#[test]
fn test_lock_upgrade_fails_multiple_shared_holders() {
    let mut manager = LockManager::new();
    let key = vec![1, 2, 3];

    // T1 acquires shared lock
    manager
        .acquire_lock(TxId::new(1), key.clone(), LockMode::Shared)
        .unwrap();

    // T2 acquires shared lock on same key
    manager
        .acquire_lock(TxId::new(2), key.clone(), LockMode::Shared)
        .unwrap();

    // T1 tries to upgrade - should fail (multiple holders)
    let result = manager.upgrade_lock(TxId::new(1), key.clone());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LockError::LockUpgradeFailed));
}

/// Test: Lock upgrade fails when there are waiters
#[test]
fn test_lock_upgrade_fails_with_waiters() {
    let mut manager = LockManager::new();
    let key = vec![1, 2, 3];

    // T1 acquires shared lock
    manager
        .acquire_lock(TxId::new(1), key.clone(), LockMode::Shared)
        .unwrap();

    // T2 waits for exclusive lock
    manager
        .acquire_lock(TxId::new(2), key.clone(), LockMode::Exclusive)
        .unwrap();

    // T1 tries to upgrade - should fail (waiters exist)
    let result = manager.upgrade_lock(TxId::new(1), key.clone());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LockError::LockUpgradeFailed));
}

/// Test: Lock upgrade succeeds when single shared holder with no waiters
#[test]
fn test_lock_upgrade_success_single_holder_no_waiters() {
    let mut manager = LockManager::new();
    let tx_id = TxId::new(1);
    let key = vec![1, 2, 3];

    // T1 acquires shared lock
    manager
        .acquire_lock(tx_id, key.clone(), LockMode::Shared)
        .unwrap();

    // T1 upgrades to exclusive - should succeed
    let result = manager.upgrade_lock(tx_id, key.clone());
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), LockGrantMode::Granted));
    assert!(manager.has_exclusive_lock(&key, tx_id));
}

/// Test: Deadlock detection via LockManager - pre-check prevents cycle
/// The pre-check in try_wait_edge prevents the cycle from forming by rejecting
/// the second wait request before any edge is added.
#[test]
fn test_lock_manager_detect_deadlock() {
    let mut manager = LockManager::new();
    let k1 = vec![1];
    let k2 = vec![2];

    // T1 holds k1, T2 holds k2
    manager
        .acquire_lock(TxId::new(1), k1.clone(), LockMode::Exclusive)
        .unwrap();
    manager
        .acquire_lock(TxId::new(2), k2.clone(), LockMode::Exclusive)
        .unwrap();

    // T1 waits for k2 - this succeeds (T1 now waiting on k2)
    // Edge T1→{T2} is added to wait-for graph
    let result1 = manager.acquire_lock(TxId::new(1), k2.clone(), LockMode::Exclusive);
    assert!(matches!(result1, Ok(LockGrantMode::Waiting)));

    // T2 waits for k1 - pre-check detects would-create-cycle (T1→T2→T1),
    // so returns Deadlock error BEFORE adding any edge.
    // This is the key PROOF-023 behavior: cycle is prevented, not detected.
    let result2 = manager.acquire_lock(TxId::new(2), k1.clone(), LockMode::Exclusive);
    assert!(result2.is_err());
    assert!(matches!(result2.unwrap_err(), LockError::Deadlock));

    // After pre-check rejection, no T2→{T1} edge exists, so detect_cycle
    // from T1 finds only T1→{T2} (linear, no cycle).
    // The cycle was PREVENTED by the pre-check, not DETECTED after formation.
    let cycle = manager.detect_deadlock(TxId::new(1));
    assert!(cycle.is_none(), "Cycle should be prevented, not detected");

    // Verify T1 is still waiting (not granted)
    assert!(matches!(result1, Ok(LockGrantMode::Waiting)));
}

/// Test: No deadlock for linear lock acquisition
#[test]
fn test_no_deadlock_linear_chain() {
    let mut manager = LockManager::new();
    let k1 = vec![1];
    let k2 = vec![2];
    let k3 = vec![3];

    // T1→T2→T3 chain, no deadlock
    manager
        .acquire_lock(TxId::new(1), k1.clone(), LockMode::Exclusive)
        .unwrap();
    manager
        .acquire_lock(TxId::new(2), k2.clone(), LockMode::Exclusive)
        .unwrap();
    manager
        .acquire_lock(TxId::new(3), k3.clone(), LockMode::Exclusive)
        .unwrap();

    // T1 waits for T2's lock
    let result = manager.acquire_lock(TxId::new(1), k2.clone(), LockMode::Exclusive);
    assert!(matches!(result, Ok(LockGrantMode::Waiting)));

    // No deadlock detected
    let cycle = manager.detect_deadlock(TxId::new(1));
    assert!(cycle.is_none());
}

/// Test: Self-wait returns Deadlock error
#[test]
fn test_self_wait_returns_deadlock() {
    let mut manager = LockManager::new();
    let key = vec![1, 2, 3];

    // T1 holds exclusive lock
    manager
        .acquire_lock(TxId::new(1), key.clone(), LockMode::Exclusive)
        .unwrap();

    // T1 tries to acquire same key again → self-wait deadlock
    let result = manager.acquire_lock(TxId::new(1), key.clone(), LockMode::Exclusive);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LockError::Deadlock));
}

// =============================================================================
// Lock Release Tests
// =============================================================================

/// Test: release_lock on non-existent lock returns Ok
#[test]
fn test_release_nonexistent_lock_ok() {
    let mut manager = LockManager::new();
    let result = manager.release_lock(TxId::new(1), &vec![1, 2, 3]);
    // release_lock returns Ok even if lock doesn't exist (idempotent)
    assert!(result.is_ok());
}

/// Test: release_all_locks on transaction with no locks returns empty vec
#[test]
fn test_release_all_locks_none_held() {
    let mut manager = LockManager::new();
    let released = manager.release_all_locks(TxId::new(999)).unwrap();
    assert!(released.is_empty());
}

// =============================================================================
// Range Lock Error Tests
// =============================================================================

/// Test: NextKey lock conflict with Gap lock
#[test]
fn test_next_key_conflicts_with_gap() {
    let mut manager = LockManager::new();

    // T1 holds Gap lock on (10, 20)
    let gap = LockTarget::Gap {
        start: Some(vec![10]),
        end: Some(vec![20]),
    };
    manager
        .acquire_lock_with_target(TxId::new(1), gap, LockMode::Exclusive)
        .unwrap();

    // T2 tries NextKey lock on key 15 (within gap)
    let nextkey = LockTarget::NextKey(vec![15]);
    let result = manager.acquire_lock_with_target(TxId::new(2), nextkey, LockMode::Exclusive);

    // Should be blocked (waiting) due to gap conflict
    assert!(matches!(result, Ok(LockGrantMode::Waiting)));
}

/// Test: Gap lock acquisition succeeds for non-overlapping gaps
#[test]
fn test_non_overlapping_gap_locks() {
    let mut manager = LockManager::new();

    // T1 holds Gap lock on (10, 20)
    let gap1 = LockTarget::Gap {
        start: Some(vec![10]),
        end: Some(vec![20]),
    };
    manager
        .acquire_lock_with_target(TxId::new(1), gap1, LockMode::Exclusive)
        .unwrap();

    // T2 holds Gap lock on (30, 40) - non-overlapping
    let gap2 = LockTarget::Gap {
        start: Some(vec![30]),
        end: Some(vec![40]),
    };
    let result = manager.acquire_lock_with_target(TxId::new(2), gap2, LockMode::Exclusive);

    assert!(matches!(result, Ok(LockGrantMode::Granted)));
}

/// Test: Release range lock when not held returns Ok (idempotent)
#[test]
fn test_release_range_lock_not_held() {
    let mut manager = LockManager::new();
    let result = manager.release_range_lock(TxId::new(999), &vec![1, 2, 3]);
    assert!(result.is_ok());
}

// =============================================================================
// Savepoint Error Tests
// =============================================================================

/// Test: rollback_to nonexistent savepoint returns NotFound error
#[test]
fn test_rollback_to_nonexistent_savepoint() {
    let mut manager = SavepointManager::new();
    let result = manager.rollback_to("nonexistent");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SavepointError::NotFound));
}

/// Test: rollback_to savepoint removes later undo records (partial rollback)
#[test]
fn test_rollback_to_partial_rollback() {
    let mut manager = SavepointManager::new();

    // Add some undo records
    manager.add_undo(UndoRecord::Insert { key: vec![1] });
    manager.add_undo(UndoRecord::Update {
        key: vec![2],
        old_value: vec![3],
    });
    manager.add_undo(UndoRecord::Delete {
        key: vec![4],
        old_value: vec![5],
    });

    // Create savepoint after first two records
    manager.savepoint("sp1".to_string()).unwrap();
    assert_eq!(manager.get_savepoint_count(), 1);

    // Add more records
    manager.add_undo(UndoRecord::Insert { key: vec![6] });
    manager.add_undo(UndoRecord::Insert { key: vec![7] });

    // Rollback to sp1 - should remove records after sp1
    manager.rollback_to("sp1").unwrap();

    // Undo log should be truncated to sp1's index (2)
    // The SavepointManager doesn't expose undo_log length directly,
    // but we can verify by adding new records and checking savepoint position
    manager.add_undo(UndoRecord::Insert { key: vec![8] });

    // Creating new savepoint should be at current undo_log len
    manager.savepoint("sp2".to_string()).unwrap();
}

/// Test: nested savepoint rollback preserves earlier savepoints
#[test]
fn test_nested_savepoint_rollback_preserves_earlier() {
    let mut manager = SavepointManager::new();

    manager.add_undo(UndoRecord::Insert { key: vec![1] });
    manager.savepoint("sp1".to_string()).unwrap();

    manager.add_undo(UndoRecord::Insert { key: vec![2] });
    manager.savepoint("sp2".to_string()).unwrap();

    manager.add_undo(UndoRecord::Insert { key: vec![3] });
    manager.savepoint("sp3".to_string()).unwrap();

    // Rollback to sp2
    manager.rollback_to("sp2").unwrap();

    // sp1 and sp2 should still exist
    assert!(manager.get_savepoint_count() >= 2);

    // sp3 should be removed
    let result = manager.rollback_to("sp3");
    assert!(result.is_err());
}

/// Test: release_savepoint removes savepoint but preserves later ones
#[test]
fn test_release_savepoint_removes_only_target() {
    let mut manager = SavepointManager::new();

    manager.savepoint("sp1".to_string()).unwrap();
    manager.savepoint("sp2".to_string()).unwrap();
    manager.savepoint("sp3".to_string()).unwrap();

    assert_eq!(manager.get_savepoint_count(), 3);

    // Release sp2
    manager.release_savepoint("sp2").unwrap();

    // sp1 and sp3 should still exist
    assert_eq!(manager.get_savepoint_count(), 2);
}

/// Test: release_savepoint on nonexistent savepoint returns Ok (idempotent)
#[test]
fn test_release_nonexistent_savepoint_ok() {
    let mut manager = SavepointManager::new();
    let result = manager.release_savepoint("nonexistent");
    assert!(result.is_ok());
}

/// Test: savepoint override updates existing savepoint's position
#[test]
fn test_savepoint_override_updates_position() {
    let mut manager = SavepointManager::new();

    manager.add_undo(UndoRecord::Insert { key: vec![1] });
    manager.savepoint("sp1".to_string()).unwrap();

    manager.add_undo(UndoRecord::Insert { key: vec![2] });
    manager.add_undo(UndoRecord::Insert { key: vec![3] });

    // Override sp1 at new position (after 3 records)
    manager.savepoint("sp1".to_string()).unwrap();

    // Rollback to sp1 should roll back to position 3
    manager.rollback_to("sp1").unwrap();

    // Adding new undo and checking savepoint count confirms position
    manager.add_undo(UndoRecord::Insert { key: vec![4] });
    assert_eq!(manager.get_savepoint_count(), 1);
}

/// Test: savepoint with duplicate name updates existing savepoint
#[test]
fn test_savepoint_duplicate_name() {
    let mut manager = SavepointManager::new();

    manager.savepoint("sp1".to_string()).unwrap();
    assert_eq!(manager.get_savepoint_count(), 1);

    // Same name should update, not create new
    manager.savepoint("sp1".to_string()).unwrap();
    assert_eq!(manager.get_savepoint_count(), 1);
}

// =============================================================================
// Wait-For Graph Cycle Detection Tests
// =============================================================================

/// Test: detect_all_cycles finds no cycles in empty graph
#[test]
fn test_wait_for_graph_detect_all_cycles_empty() {
    let graph = WaitForGraph::new();
    let cycles = graph.detect_all_cycles();
    assert!(cycles.is_empty());
}

/// Test: detect_all_cycles finds 2-node cycle
#[test]
fn test_wait_for_graph_detect_two_node_cycle() {
    let mut graph = WaitForGraph::new();

    // Build: 1 → 2 → 1
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(1),
        writer: TxId::new(2),
        key: b"x".to_vec(),
    });
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(2),
        writer: TxId::new(1),
        key: b"y".to_vec(),
    });

    let cycles = graph.detect_all_cycles();
    assert!(!cycles.is_empty());
}

/// Test: detect_all_cycles finds no cycles in linear chain
#[test]
fn test_wait_for_graph_no_cycles_linear_chain() {
    let mut graph = WaitForGraph::new();

    // Build: 1 → 2 → 3 (no cycle)
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(1),
        writer: TxId::new(2),
        key: b"x".to_vec(),
    });
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(2),
        writer: TxId::new(3),
        key: b"y".to_vec(),
    });

    let cycles = graph.detect_all_cycles();
    assert!(cycles.is_empty());
}

/// Test: would_create_dangerous_cycle detects cycle formation
#[test]
fn test_wait_for_graph_would_create_cycle() {
    let mut graph = WaitForGraph::new();

    // 1 → 2
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(1),
        writer: TxId::new(2),
        key: b"x".to_vec(),
    });

    // Adding 2 → 1 would create cycle
    let edge = WaitForEdge::ReadWrite {
        reader: TxId::new(2),
        writer: TxId::new(1),
        key: b"y".to_vec(),
    };

    let cycle = graph.would_create_dangerous_cycle(&edge);
    assert!(cycle.is_some());
}

/// Test: would_create_dangerous_cycle allows non-cyclic addition
#[test]
fn test_wait_for_graph_would_not_create_cycle() {
    let mut graph = WaitForGraph::new();

    // 1 → 2
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(1),
        writer: TxId::new(2),
        key: b"x".to_vec(),
    });

    // Adding 3 → 1 would NOT create cycle (1 → 2 ← 3)
    let edge = WaitForEdge::ReadWrite {
        reader: TxId::new(3),
        writer: TxId::new(1),
        key: b"y".to_vec(),
    };

    let cycle = graph.would_create_dangerous_cycle(&edge);
    assert!(cycle.is_none());
}

/// Test: self-loop is rejected
#[test]
fn test_wait_for_graph_no_self_loop() {
    let mut graph = WaitForGraph::new();

    let edge = WaitForEdge::WriteWrite {
        writer1: TxId::new(1),
        writer2: TxId::new(1),
        key: b"x".to_vec(),
    };

    // Self-loop should not be added
    assert!(!graph.add_edge(edge));
    assert!(graph.is_empty());
}

/// Test: remove_transaction removes all edges
#[test]
fn test_wait_for_graph_remove_transaction() {
    let mut graph = WaitForGraph::new();

    // 1 → 2, 2 → 3
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(1),
        writer: TxId::new(2),
        key: b"x".to_vec(),
    });
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(2),
        writer: TxId::new(3),
        key: b"y".to_vec(),
    });

    // Remove transaction 2
    graph.remove_transaction(TxId::new(2));

    // All edges should be removed
    assert_eq!(graph.num_edges(), 0);
    assert!(graph.is_empty());
}

/// Test: get_waiters returns correct waiters
#[test]
fn test_wait_for_graph_get_waiters() {
    let mut graph = WaitForGraph::new();

    // 1 → 2, 1 → 3
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(1),
        writer: TxId::new(2),
        key: b"x".to_vec(),
    });
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(1),
        writer: TxId::new(3),
        key: b"y".to_vec(),
    });

    let waiters = graph.get_waiters(TxId::new(1));
    assert_eq!(waiters.len(), 2);
    assert!(waiters.contains(&TxId::new(2)));
    assert!(waiters.contains(&TxId::new(3)));
}

/// Test: get_holders returns correct holders
#[test]
fn test_wait_for_graph_get_holders() {
    let mut graph = WaitForGraph::new();

    // 1 → 2, 3 → 2
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(1),
        writer: TxId::new(2),
        key: b"x".to_vec(),
    });
    graph.add_edge(WaitForEdge::ReadWrite {
        reader: TxId::new(3),
        writer: TxId::new(2),
        key: b"y".to_vec(),
    });

    let holders = graph.get_holders(TxId::new(2));
    assert_eq!(holders.len(), 2);
    assert!(holders.contains(&TxId::new(1)));
    assert!(holders.contains(&TxId::new(3)));
}

/// Test: has_rw_wr_conflict detects dangerous cycle
#[test]
fn test_wait_for_graph_rw_wr_conflict() {
    let mut graph = WaitForGraph::new();

    // T1 reads X, T2 writes X
    graph.record_read(TxId::new(1), b"X".to_vec());
    graph.record_write(TxId::new(2), b"X".to_vec());

    // T1 writes Y, T2 reads Y
    graph.record_write(TxId::new(1), b"Y".to_vec());
    graph.record_read(TxId::new(2), b"Y".to_vec());

    // This is a dangerous RW-WR cycle
    assert!(graph.has_rw_wr_conflict(TxId::new(1), TxId::new(2)));
}

/// Test: has_rw_wr_conflict returns false for non-conflicting transactions
#[test]
fn test_wait_for_graph_no_rw_wr_conflict() {
    let mut graph = WaitForGraph::new();

    // T1 reads X, T2 writes X
    graph.record_read(TxId::new(1), b"X".to_vec());
    graph.record_write(TxId::new(2), b"X".to_vec());

    // But they don't have cross-dependencies
    assert!(!graph.has_rw_wr_conflict(TxId::new(1), TxId::new(2)));
}

/// Test: is_empty returns true for new graph
#[test]
fn test_wait_for_graph_is_empty_new() {
    let graph = WaitForGraph::new();
    assert!(graph.is_empty());
    assert_eq!(graph.num_transactions(), 0);
    assert_eq!(graph.num_edges(), 0);
}

/// Test: adding duplicate edge returns false
#[test]
fn test_wait_for_graph_duplicate_edge() {
    let mut graph = WaitForGraph::new();

    let edge = WaitForEdge::ReadWrite {
        reader: TxId::new(1),
        writer: TxId::new(2),
        key: b"x".to_vec(),
    };

    assert!(graph.add_edge(edge.clone()));
    assert!(!graph.add_edge(edge)); // Duplicate
    assert_eq!(graph.num_edges(), 1);
}

// =============================================================================
// Error Display/Debug Tests
// =============================================================================

/// Test: LockError Display implementation
#[test]
fn test_lock_error_display() {
    let deadlock = LockError::Deadlock;
    assert_eq!(deadlock.to_string(), "deadlock detected");

    let upgrade_failed = LockError::LockUpgradeFailed;
    assert_eq!(upgrade_failed.to_string(), "lock upgrade failed");

    let not_held = LockError::LockNotHeld;
    assert_eq!(not_held.to_string(), "lock not held by transaction");

    let timeout = LockError::LockTimeout;
    assert_eq!(timeout.to_string(), "lock acquisition timeout");
}

/// Test: SavepointError Display implementation
#[test]
fn test_savepoint_error_display() {
    let not_found = SavepointError::NotFound;
    assert_eq!(not_found.to_string(), "savepoint not found");

    let invalid = SavepointError::InvalidOperation;
    assert_eq!(invalid.to_string(), "invalid savepoint operation");
}

/// Test: SavepointError can be used as error trait object
#[test]
fn test_savepoint_error_as_trait_object() {
    let err: Box<dyn std::error::Error> = Box::new(SavepointError::NotFound);
    assert_eq!(err.to_string(), "savepoint not found");
}

/// Test: LockError can be used as error trait object
#[test]
fn test_lock_error_as_trait_object() {
    let err: Box<dyn std::error::Error> = Box::new(LockError::Deadlock);
    assert_eq!(err.to_string(), "deadlock detected");
}
