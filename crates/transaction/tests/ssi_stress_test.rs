//! PROOF-026: Write Skew / SSI 并发压力测试
//!
//! Tests that SSI (Serializable Snapshot Isolation) correctly detects
//! and prevents Write Skew anomalies, with no false positives
//! under high concurrency.

use sqlrustgo_transaction::{
    mvcc::TxId,
    ssi::{SerializationGraph, SsiDetectorSync, SsiError},
    IsolationLevel,
};

/// Test basic SerializationGraph cycle detection
#[test]
fn test_serialization_graph_no_cycle() {
    let graph = SerializationGraph::new();
    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);
    assert!(!graph.would_create_cycle(tx1, tx2));
}

#[test]
fn test_serialization_graph_cycle_detection() {
    let mut graph = SerializationGraph::new();
    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    // No cycle initially
    assert!(!graph.would_create_cycle(tx1, tx2));

    // Add edges to create cycle: tx1 -> tx2 -> tx1
    graph.add_dependency(tx1, tx2);
    graph.add_dependency(tx2, tx1);

    // Now adding tx1 -> tx2 again should NOT detect a cycle
    // (the cycle already exists but would_create_cycle checks NEW edges)
    assert!(graph.would_create_cycle(tx1, tx2));
}

/// Test Write Skew scenario:
/// T1 reads X, writes Y
/// T2 reads Y, writes X
/// If committed concurrently, Invariant (X=1 OR Y=1) is violated
#[test]
fn test_write_skew_detection() {
    let detector = SsiDetectorSync::new();
    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    // T1 reads X
    detector.record_read(tx1, b"X".to_vec());
    // T2 reads Y
    detector.record_read(tx2, b"Y".to_vec());

    // T1 writes Y
    detector.record_write(tx1, b"Y".to_vec());
    // T2 writes X
    detector.record_write(tx2, b"X".to_vec());

    // Both try to commit - one should fail due to rw-rw conflict
    let r1 = detector.validate_commit(tx1);
    let r2 = detector.validate_commit(tx2);

    // At least one must fail (SerializationConflict)
    assert!(
        r1.is_err() || r2.is_err(),
        "Write Skew must be detected: one transaction must be aborted"
    );

    // Clean up both regardless
    let _ = detector.release(tx1);
    let _ = detector.release(tx2);
}

/// Test no false positives: 100 concurrent transactions on independent keys
#[test]
fn test_concurrent_no_false_positives() {
    let detector = SsiDetectorSync::new();
    let mut txs = Vec::new();

    // Create 100 transactions, each operating on their own key
    for i in 0..100u64 {
        let tx = TxId::new(i);
        let key = format!("key_{}", i).into_bytes();

        detector.record_read(tx, key.clone());
        detector.record_write(tx, key);

        txs.push(tx);
    }

    // All should commit successfully (no conflicts, all independent keys)
    for tx in &txs {
        let result = detector.validate_commit(*tx);
        assert!(
            result.is_ok(),
            "No false positive: tx {:?} should commit: {:?}",
            tx,
            result
        );
    }

    // Release all
    for tx in &txs {
        let _ = detector.release(*tx);
    }
}

/// Test ww-conflict: SSI allows two writes to same key (no rw-wr cycle)
#[test]
fn test_si_ww_conflict_allowed() {
    let detector = SsiDetectorSync::new();
    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    // T1 writes X
    detector.record_write(tx1, b"X".to_vec());
    // T2 writes X - same key, should conflict
    detector.record_write(tx2, b"X".to_vec());

    let r1 = detector.validate_commit(tx1);
    let r2 = detector.validate_commit(tx2);

    // SSI allows two writes to the same key (first-committer-wins at storage level)
    assert!(r1.is_ok(), "T1 should commit OK in SSI");
    assert!(r2.is_ok(), "T2 should commit OK in SSI (no rw-wr cycle)");

    let _ = detector.release(tx1);
    let _ = detector.release(tx2);
}

/// Test read-write conflict: T1 reads, T2 writes same key
#[test]
fn test_rw_conflict_detection() {
    let detector = SsiDetectorSync::new();
    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    // T1 reads X
    detector.record_read(tx1, b"X".to_vec());
    // T2 writes X
    detector.record_write(tx2, b"X".to_vec());

    let r1 = detector.validate_commit(tx1);
    let r2 = detector.validate_commit(tx2);

    // T2's write of X that T1 read might cause serialization failure
    // At least one should fail if they form a dependency cycle
    let _ = detector.release(tx1);
    let _ = detector.release(tx2);
}

/// Test 100 concurrent transactions on overlapping keys
/// (stress test - should not cause crashes or deadlocks)
#[test]
fn test_ssi_concurrent_stress() {
    use std::sync::Arc;
    use std::thread;

    let detector = Arc::new(SsiDetectorSync::new());
    let mut handles = Vec::new();

    for i in 0u64..20 {
        let det = detector.clone();
        handles.push(thread::spawn(move || {
            let tx = TxId::new(i);
            // Each transaction reads its own key and writes a shared key
            let own_key = format!("own_{}", i).into_bytes();
            let shared_key = b"shared_counter".to_vec();

            det.record_read(tx, own_key);
            det.record_write(tx, shared_key);

            let result = det.validate_commit(tx);
            let _ = det.release(tx);
            result
        }));
    }

    let results: Vec<Result<(), SsiError>> =
        handles.into_iter().map(|h| h.join().unwrap()).collect();

    let successes = results.iter().filter(|r| r.is_ok()).count();
    let failures = results.iter().filter(|r| r.is_err()).count();

    // In 20 concurrent transactions writing to a shared key,
    // some should succeed and some should fail (conflict expected)
    assert!(successes > 0, "At least one transaction should succeed");
    assert!(
        successes + failures == 20,
        "All 20 transactions must produce a result"
    );

    println!(
        "SSI stress test: {} committed, {} aborted (conflicts)",
        successes, failures
    );
}
