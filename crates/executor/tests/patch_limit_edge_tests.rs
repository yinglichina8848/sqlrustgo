use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn engine() -> ExecutionEngine<MemoryStorage> {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

#[test]
fn test_limit_zero_rows() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1), (2), (3)").unwrap();
    let r = e.execute("SELECT * FROM t LIMIT 0").unwrap();
    assert_eq!(r.rows.len(), 0);
}

#[test]
fn test_limit_one_row() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (10), (20), (30)").unwrap();
    let r = e.execute("SELECT * FROM t LIMIT 1").unwrap();
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.rows[0][0], Value::Integer(10));
}

#[test]
fn test_limit_exceeds_all() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1), (2)").unwrap();
    let r = e.execute("SELECT * FROM t LIMIT 100").unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn test_limit_exact_all() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1), (2)").unwrap();
    let r = e.execute("SELECT * FROM t LIMIT 2").unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn test_limit_with_offset_zero() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1), (2), (3), (4), (5)").unwrap();
    let r = e.execute("SELECT * FROM t LIMIT 3 OFFSET 0").unwrap();
    assert_eq!(r.rows.len(), 3);
}

#[test]
fn test_limit_with_offset_mid() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1), (2), (3), (4), (5)").unwrap();
    let r = e.execute("SELECT * FROM t LIMIT 2 OFFSET 2").unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn test_limit_after_filter() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1), (2), (3), (4), (5)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE a > 2 LIMIT 2").unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn test_limit_after_aggregate() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1), (2), (3)").unwrap();
    let r = e.execute("SELECT COUNT(*) FROM t LIMIT 1").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_limit_with_order_asc() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (3), (1), (2)").unwrap();
    let r = e.execute("SELECT * FROM t ORDER BY a LIMIT 2").unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn test_limit_with_order_desc() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (3), (1), (2)").unwrap();
    let r = e.execute("SELECT * FROM t ORDER BY a DESC LIMIT 2").unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn test_pagination_three_pages() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(2),(3),(4),(5),(6),(7),(8),(9),(10)").unwrap();
    for offset in &[0usize, 3, 6] {
        let sql = format!("SELECT * FROM t ORDER BY a LIMIT 3 OFFSET {}", offset);
        let r = e.execute(&sql).unwrap();
        assert_eq!(r.rows.len(), 3, "page offset={}", offset);
    }
}

#[test]
fn test_limit_empty_table() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    let r = e.execute("SELECT * FROM t LIMIT 5").unwrap();
    assert_eq!(r.rows.len(), 0);
}
