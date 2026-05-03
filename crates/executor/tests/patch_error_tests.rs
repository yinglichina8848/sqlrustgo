use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn engine() -> ExecutionEngine<MemoryStorage> {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

#[test]
fn test_select_from_nonexistent_returns_error() {
    let mut e = engine();
    let r = e.execute("SELECT * FROM nonexistent");
    assert!(r.is_err());
}

#[test]
fn test_insert_into_nonexistent_returns_error() {
    let mut e = engine();
    let r = e.execute("INSERT INTO t VALUES (1)");
    assert!(r.is_err());
}

#[test]
fn test_select_from_empty_works() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    let r = e.execute("SELECT * FROM t").unwrap();
    assert_eq!(r.rows.len(), 0);
}

#[test]
#[should_panic(expected = "ParseError")]
fn test_unterminated_where() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("SELECT * FROM t WHERE a =").unwrap();
}

#[test]
#[should_panic(expected = "ParseError")]
fn test_missing_parens() {
    let mut e = engine();
    e.execute("SELECT (1+2").unwrap();
}

#[test]
fn test_update_nonexistent_silent() {
    let mut e = engine();
    let r = e.execute("UPDATE nonexistent SET a = 1");
    assert!(r.is_ok());
}

#[test]
fn test_divide_by_zero_no_panic() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (10), (20)").unwrap();
    let r = e.execute("SELECT a / 0 FROM t").unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn test_empty_string_insert() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a TEXT)").unwrap();
    e.execute("INSERT INTO t VALUES ('')").unwrap();
    let r = e.execute("SELECT * FROM t").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_batch_insert_large() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    let values: Vec<String> = (1..=100).map(|i| format!("({})", i)).collect();
    let sql = format!("INSERT INTO t VALUES {}", values.join(", "));
    e.execute(&sql).unwrap();
    let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(r.rows[0][0], Value::Integer(100));
}

#[test]
fn test_drop_recreate_table() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1)").unwrap();
    e.execute("DROP TABLE t").unwrap();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (2)").unwrap();
    let r = e.execute("SELECT * FROM t").unwrap();
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.rows[0][0], Value::Integer(2));
}

#[test]
fn test_update_set_multiple() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER, b INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1, 10), (2, 20)").unwrap();
    e.execute("UPDATE t SET a = a + 1, b = b + 1").unwrap();
    let r = e.execute("SELECT a, b FROM t WHERE a = 2").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_delete_where() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1), (2), (3)").unwrap();
    e.execute("DELETE FROM t WHERE a = 2").unwrap();
    let r = e.execute("SELECT * FROM t").unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn test_delete_all_rows() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1), (2)").unwrap();
    e.execute("DELETE FROM t").unwrap();
    let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(r.rows[0][0], Value::Integer(0));
}

#[test]
fn test_drop_table_no_data() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("DROP TABLE t").unwrap();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(r.rows[0][0], Value::Integer(0));
}

#[test]
fn test_insert_large_values() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (10), (20), (30), (40), (50)").unwrap();
    let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(r.rows[0][0], Value::Integer(5));
}
