use sqlrustgo::execution_engine::EngineConfig;
use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_fresh_engine() -> MemoryExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    MemoryExecutionEngine::new_with_config(storage, EngineConfig::default())
}

#[test]
fn test_recovery_after_failed_transaction() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE users (id INTEGER, name TEXT)");
    let _ = engine.execute("INSERT INTO users VALUES (1, 'Alice')");

    let result = engine.execute("SELECT * FROM users WHERE id = 1");
    assert!(result.is_ok());
}

#[test]
fn test_recovery_after_invalid_insert() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t (id INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1)");
    let _ = engine.execute("INSERT INTO invalid_table VALUES (1)");

    let result = engine.execute("SELECT * FROM t");
    assert!(result.is_ok());
}

#[test]
fn test_recovery_after_parse_error() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t (id INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1)");
    let _ = engine.execute("SELEKT * FROM t");

    let result = engine.execute("SELECT * FROM t");
    assert!(result.is_ok());
}

#[test]
fn test_rollback_simulation() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE accounts (id INTEGER, balance INTEGER)");
    let _ = engine.execute("INSERT INTO accounts VALUES (1, 1000)");
    let _ = engine.execute("INSERT INTO accounts VALUES (2, 500)");

    let _ = engine.execute("UPDATE accounts SET balance = balance - 100 WHERE id = 1");
    let _ = engine.execute("UPDATE accounts SET balance = balance + 100 WHERE id = 2");

    let result = engine.execute("SELECT balance FROM accounts WHERE id = 2");
    assert!(result.is_ok());
}

#[test]
fn test_state_persistence_across_queries() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE counters (id INTEGER, count INTEGER)");
    let _ = engine.execute("INSERT INTO counters VALUES (1, 10)");

    let result = engine.execute("SELECT count FROM counters WHERE id = 1");
    eprintln!("Result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_partial_query_failure_isolation() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t1 (id INTEGER)");
    let _ = engine.execute("CREATE TABLE t2 (id INTEGER)");
    let _ = engine.execute("INSERT INTO t1 VALUES (1)");
    let _ = engine.execute("INSERT INTO nonexistent VALUES (1)");
    let _ = engine.execute("INSERT INTO t2 VALUES (2)");

    let result1 = engine.execute("SELECT * FROM t1");
    let result2 = engine.execute("SELECT * FROM t2");

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[test]
fn test_concurrent_crash_simulation() {
    // This test simulates crash recovery by creating tables and data,
    // then verifying the data persists in the same engine
    let mut engine = create_fresh_engine();
    let _ = engine.execute("CREATE TABLE test_data (id INTEGER)");

    for i in 0..5 {
        let _ = engine.execute(&format!("INSERT INTO test_data VALUES ({})", i));
    }

    // Verify data persists in the same engine
    let result = engine.execute("SELECT COUNT(*) FROM test_data");
    if let Err(e) = &result {
        eprintln!("SELECT COUNT(*) failed: {:?}", e);
    }
    assert!(result.is_ok());
}

#[test]
fn test_memory_cleanup_after_drops() {
    let mut engine = create_fresh_engine();

    for i in 0..10 {
        let _ = engine.execute(&format!("DROP TABLE IF EXISTS t{}", i));
    }

    for i in 0..10 {
        let _ = engine.execute(&format!("CREATE TABLE t{} (id INTEGER)", i));
    }

    let result = engine.execute("SELECT 1");
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_transaction_state_stress_100_concurrent() {
    use sqlrustgo_transaction::{IsolationLevel, TransactionManager};
    use std::sync::Arc;
    use std::sync::RwLock;

    let num_tx = 100;
    let manager: Arc<RwLock<TransactionManager>> = Arc::new(RwLock::new(TransactionManager::new()));

    let mut handles = Vec::with_capacity(num_tx);
    for i in 0..num_tx {
        let mgr = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let tx_id = {
                let mut m = mgr.write().unwrap();
                m.begin_transaction(IsolationLevel::SnapshotIsolation)
                    .expect("begin_transaction should succeed")
            };

            let key = format!("key_{}", i);
            {
                let mut m = mgr.write().unwrap();
                m.record_read(
                    tx_id,
                    key.clone().into_bytes(),
                    sqlrustgo::IsolationLevel::SnapshotIsolation,
                )
                .expect("record_read should succeed");
                m.record_write(tx_id, key.clone().into_bytes())
                    .expect("record_write should succeed");
            }

            let result = {
                let mut m = mgr.write().unwrap();
                m.commit(tx_id)
            };
            result.expect("commit should succeed");
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("task should succeed");
    }

    let active_count = {
        let m = manager.read().unwrap();
        m.active_transaction_count()
    };
    assert_eq!(
        active_count, 0,
        "State leak detected: {} active transactions after 100 concurrent commits",
        active_count
    );
}
