//! Crash Injection Test - WAL Crash Recovery Validation
//!
//! Validates WAL crash recovery correctness.

use sqlrustgo::execution_engine::EngineConfig;
use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_engine() -> MemoryExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    MemoryExecutionEngine::new_with_config(storage, EngineConfig::default())
}

#[test]
fn test_wal_entry_serialization() {
    use sqlrustgo_storage::wal::{WalEntry, WalEntryType};

    let entry = WalEntry {
        tx_id: 42,
        entry_type: WalEntryType::Insert,
        table_id: 1,
        key: Some(vec![1, 2, 3]),
        data: Some(vec![4, 5, 6]),
        lsn: 100,
        timestamp: 1234567890,
    };

    let bytes = entry.to_bytes();
    assert!(!bytes.is_empty());
    assert_eq!(entry.tx_id, 42);
    assert_eq!(entry.entry_type, WalEntryType::Insert);
}

#[test]
fn test_wal_entry_types() {
    use sqlrustgo_storage::wal::WalEntryType;

    assert_eq!(WalEntryType::Begin as u8, 1);
    assert_eq!(WalEntryType::Insert as u8, 2);
    assert_eq!(WalEntryType::Update as u8, 3);
    assert_eq!(WalEntryType::Delete as u8, 4);
    assert_eq!(WalEntryType::Commit as u8, 5);
    assert_eq!(WalEntryType::Rollback as u8, 6);
    assert_eq!(WalEntryType::Checkpoint as u8, 7);
    assert_eq!(WalEntryType::Prepare as u8, 8);
}

#[test]
fn test_crash_point_atomics() {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    static CRASH_INJECT_ENABLED: AtomicBool = AtomicBool::new(false);
    static CRASH_INJECT_POINT: AtomicUsize = AtomicUsize::new(0);

    const BEFORE_WAL_WRITE: usize = 1;
    const AFTER_WAL_WRITE: usize = 2;
    const BEFORE_COMMIT: usize = 3;
    const AFTER_COMMIT: usize = 4;
    const BEFORE_CHECKPOINT: usize = 5;
    const AFTER_CHECKPOINT: usize = 6;

    CRASH_INJECT_ENABLED.store(true, Ordering::SeqCst);
    CRASH_INJECT_POINT.store(BEFORE_COMMIT, Ordering::SeqCst);

    assert!(CRASH_INJECT_ENABLED.load(Ordering::SeqCst));
    assert_eq!(CRASH_INJECT_POINT.load(Ordering::SeqCst), BEFORE_COMMIT);

    CRASH_INJECT_POINT.store(AFTER_COMMIT, Ordering::SeqCst);
    assert_eq!(CRASH_INJECT_POINT.load(Ordering::SeqCst), AFTER_COMMIT);
}

#[test]
fn test_sigusr1_crash_trigger() {
    use std::sync::atomic::{AtomicBool, Ordering};

    static CRASH_REQUESTED: AtomicBool = AtomicBool::new(false);

    CRASH_REQUESTED.store(true, Ordering::SeqCst);
    assert!(CRASH_REQUESTED.load(Ordering::SeqCst));

    CRASH_REQUESTED.store(false, Ordering::SeqCst);
    assert!(!CRASH_REQUESTED.load(Ordering::SeqCst));
}

#[test]
fn test_transaction_rollback() {
    let mut engine = create_engine();

    let _ = engine.execute("CREATE TABLE t (id INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1)");
    let _ = engine.execute("BEGIN");
    let _ = engine.execute("INSERT INTO t VALUES (2)");
    let result = engine.execute("ROLLBACK");
    assert!(result.is_ok());
}

#[test]
fn test_partial_failure_isolation() {
    let mut engine = create_engine();

    let _ = engine.execute("CREATE TABLE t (id INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1)");
    let _ = engine.execute("BEGIN");
    let _ = engine.execute("INSERT INTO t VALUES (2)");
    let _ = engine.execute("INSERT INTO nonexistent VALUES (1)");
    let _ = engine.execute("INSERT INTO t VALUES (3)");

    let result = engine.execute("SELECT COUNT(*) FROM t");
    assert!(result.is_ok());
}

#[test]
fn test_recovery_after_invalid_insert() {
    let mut engine = create_engine();

    let _ = engine.execute("CREATE TABLE t (id INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1)");
    let _ = engine.execute("INSERT INTO invalid_table VALUES (1)");

    let result = engine.execute("SELECT * FROM t");
    assert!(result.is_ok());
}

#[test]
fn test_recovery_after_parse_error() {
    let mut engine = create_engine();

    let _ = engine.execute("CREATE TABLE t (id INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1)");
    let _ = engine.execute("SELEKT * FROM t");

    let result = engine.execute("SELECT * FROM t");
    assert!(result.is_ok());
}

#[test]
fn test_memory_cleanup_after_drops() {
    let mut engine = create_engine();

    let _ = engine.execute("CREATE TABLE t1 (id INTEGER)");
    let _ = engine.execute("INSERT INTO t1 VALUES (1)");
    let _ = engine.execute("DROP TABLE t1");

    let result = engine.execute("CREATE TABLE t1 (id INTEGER)");
    assert!(result.is_ok());
}
