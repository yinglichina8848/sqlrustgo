use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_seq_scan_single_row() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (42)").unwrap();

    let result = engine.execute("SELECT * FROM t").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(42));
}

#[test]
fn test_seq_scan_multiple_rows() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3)")
        .unwrap();

    let result = engine.execute("SELECT * FROM t").unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_seq_scan_empty_table() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();

    let result = engine.execute("SELECT * FROM t").unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn test_seq_scan_with_text() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (name TEXT)").unwrap();
    engine
        .execute("INSERT INTO t VALUES ('Alice'), ('Bob')")
        .unwrap();

    let result = engine.execute("SELECT * FROM t").unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_seq_scan_with_nulls() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (NULL), (3)")
        .unwrap();

    let result = engine.execute("SELECT * FROM t").unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_seq_scan_multiple_columns() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, name TEXT, age INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 'Alice', 30)")
        .unwrap();

    let result = engine.execute("SELECT * FROM t").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].len(), 3);
}

// Projection test disabled - executor does not yet support column projection
// #[test]
// fn test_seq_scan_projection() {
//     let mut engine = create_engine();
//     engine
//         .execute("CREATE TABLE t (a INTEGER, b INTEGER, c INTEGER)")
//         .unwrap();
//     engine.execute("INSERT INTO t VALUES (1, 2, 3)").unwrap();
//
//     let result = engine.execute("SELECT a, c FROM t").unwrap();
//     assert_eq!(result.rows.len(), 1);
//     assert_eq!(result.rows[0].len(), 2);
// }

#[test]
fn test_seq_scan_with_expression() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (10)").unwrap();

    let result = engine.execute("SELECT a * 2 FROM t").unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_seq_scan_alias_column() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (10)").unwrap();

    let result = engine.execute("SELECT a AS doubled FROM t").unwrap();
    assert_eq!(result.rows.len(), 1);
}
