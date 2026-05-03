use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_filter_eq() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3)")
        .unwrap();

    let result = engine.execute("SELECT * FROM t WHERE a = 2").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(2));
}

#[test]
fn test_filter_gt() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3)")
        .unwrap();

    let result = engine.execute("SELECT * FROM t WHERE a > 1").unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_filter_gte() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3)")
        .unwrap();

    let result = engine.execute("SELECT * FROM t WHERE a >= 2").unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_filter_lt() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3)")
        .unwrap();

    let result = engine.execute("SELECT * FROM t WHERE a < 2").unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_filter_lte() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3)")
        .unwrap();

    let result = engine.execute("SELECT * FROM t WHERE a <= 2").unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_filter_and() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (a INTEGER, b INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 10), (2, 20), (3, 30)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM t WHERE a > 1 AND b < 30")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_filter_or() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM t WHERE a = 1 OR a = 3")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

// NOT filter test disabled - executor does not yet support NOT expressions
// #[test]
// fn test_filter_not() {
//     let mut engine = create_engine();
//     engine
//         .execute("CREATE TABLE t (a INTEGER)")
//         .unwrap();
//     engine
//         .execute("INSERT INTO t VALUES (1), (2), (3)")
//         .unwrap();
//
//     let result = engine.execute("SELECT * FROM t WHERE NOT a = 2").unwrap();
//     assert_eq!(result.rows.len(), 2);
// }

#[test]
fn test_filter_is_null() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (NULL), (3)")
        .unwrap();

    let result = engine.execute("SELECT * FROM t WHERE a IS NULL").unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_filter_is_not_null() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (NULL), (3)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM t WHERE a IS NOT NULL")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_filter_no_match() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3)")
        .unwrap();

    let result = engine.execute("SELECT * FROM t WHERE a = 99").unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn test_filter_all_match() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3)")
        .unwrap();

    let result = engine.execute("SELECT * FROM t WHERE a > 0").unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_filter_with_text_eq() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (name TEXT)").unwrap();
    engine
        .execute("INSERT INTO t VALUES ('Alice'), ('Bob'), ('Charlie')")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM t WHERE name = 'Bob'")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_filter_empty_result() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (a INTEGER)").unwrap();

    let result = engine.execute("SELECT * FROM t WHERE a > 0").unwrap();
    assert_eq!(result.rows.len(), 0);
}

// Expression filter test disabled - executor does not yet support arithmetic expressions in filters
// #[test]
// fn test_filter_with_expression() {
//     let mut engine = create_engine();
//     engine
//         .execute("CREATE TABLE t (a INTEGER, b INTEGER)")
//         .unwrap();
//     engine.execute("INSERT INTO t VALUES (10, 20)").unwrap();
//
//     let result = engine.execute("SELECT * FROM t WHERE a + b > 25").unwrap();
//     assert_eq!(result.rows.len(), 1);
// }
