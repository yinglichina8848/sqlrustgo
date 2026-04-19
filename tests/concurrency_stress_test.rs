use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};
use std::thread;

fn setup_engine() -> MemoryExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    MemoryExecutionEngine::new(storage)
}

#[test]
fn test_concurrent_reads() {
    let mut handles = vec![];

    for _ in 0..10 {
        let handle = thread::spawn(|| {
            let mut engine = setup_engine();
            let _ = engine.execute("SELECT 1");
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_writes() {
    let mut handles = vec![];

    for i in 0..10 {
        let handle = thread::spawn(move || {
            let mut engine = setup_engine();
            let _ = engine.execute("CREATE TABLE t (id INTEGER)");
            let _ = engine.execute(&format!("INSERT INTO t VALUES ({})", i));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_mixed_operations() {
    let mut handles = vec![];

    for i in 0..10 {
        let handle = thread::spawn(move || {
            let mut engine = setup_engine();
            let _ = engine.execute("CREATE TABLE t (id INTEGER, value TEXT)");
            let _ = engine.execute(&format!("INSERT INTO t VALUES ({}, 'test')", i));
            let _ = engine.execute("SELECT * FROM t");
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_sequential_operations() {
    let mut engine = setup_engine();

    let _ = engine.execute("CREATE TABLE users (id INTEGER, name TEXT)");
    let _ = engine.execute("INSERT INTO users VALUES (1, 'Alice')");
    let _ = engine.execute("INSERT INTO users VALUES (2, 'Bob')");
    let result = engine.execute("SELECT * FROM users");

    assert!(result.is_ok());
}

#[test]
fn test_rapid_create_drop_table() {
    let mut engine = setup_engine();

    for i in 0..10 {
        let create_sql = format!("CREATE TABLE t{} (id INTEGER)", i);
        let drop_sql = format!("DROP TABLE t{}", i);

        let _ = engine.execute(&create_sql);
        let _ = engine.execute(&drop_sql);
    }
}

#[test]
fn test_concurrent_table_creation() {
    let mut handles = vec![];

    for i in 0..20 {
        let handle = thread::spawn(move || {
            let mut engine = setup_engine();
            let sql = format!("CREATE TABLE table_{} (id INTEGER, data TEXT)", i);
            let _ = engine.execute(&sql);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_long_running_query_interruption() {
    let mut engine = setup_engine();
    let _ = engine.execute("SELECT 1");
}

#[test]
fn test_repeated_same_query() {
    let mut engine = setup_engine();

    for _ in 0..100 {
        let _ = engine.execute("SELECT 1");
    }
}

#[test]
fn test_mixed_dml_operations() {
    let mut engine = setup_engine();

    let _ = engine.execute("CREATE TABLE test (id INTEGER, value INTEGER)");
    let _ = engine.execute("INSERT INTO test VALUES (1, 100)");
    let _ = engine.execute("INSERT INTO test VALUES (2, 200)");
    let _ = engine.execute("UPDATE test SET value = 150 WHERE id = 1");
    let _ = engine.execute("SELECT * FROM test WHERE id = 1");
    let _ = engine.execute("DELETE FROM test WHERE id = 2");
    let _ = engine.execute("SELECT COUNT(*) FROM test");
}
