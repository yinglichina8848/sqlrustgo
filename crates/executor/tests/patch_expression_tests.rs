use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn engine() -> ExecutionEngine<MemoryStorage> {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

#[test]
fn test_expr_column_plus_column() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER, b INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (7, 5)").unwrap();
    let r = e.execute("SELECT a + b FROM t").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_expr_column_sub_column() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER, b INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (10, 3)").unwrap();
    let r = e.execute("SELECT a - b FROM t").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_expr_column_mul_column() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER, b INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (6, 7)").unwrap();
    let r = e.execute("SELECT a * b FROM t").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_expr_column_div_column() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER, b INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (20, 4)").unwrap();
    let r = e.execute("SELECT a / b FROM t").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_expr_column_plus_literal() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (2)").unwrap();
    let r = e.execute("SELECT a + 3 FROM t WHERE a > 0").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_expr_with_null_column() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (NULL)").unwrap();
    let r = e.execute("SELECT a + 1 FROM t").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_coalesce_on_table() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (NULL), (1)").unwrap();
    let r = e.execute("SELECT COALESCE(a, 99) FROM t").unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn test_nullif_on_table() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (5)").unwrap();
    let r = e.execute("SELECT NULLIF(a, 5) FROM t").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_case_when_on_table() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(2),(3)").unwrap();
    let r = e
        .execute("SELECT CASE WHEN a=1 THEN 'one' WHEN a=2 THEN 'two' ELSE 'other' END FROM t")
        .unwrap();
    assert_eq!(r.rows.len(), 3);
}

#[test]
fn test_not_null_filter() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (NULL), (1)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE a IS NOT NULL").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_is_null_filter() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (NULL), (1)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE a IS NULL").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_where_eq_null_no_rows() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (NULL)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE a = NULL").unwrap();
    assert_eq!(r.rows.len(), 0);
}

#[test]
fn test_where_ne_null_no_rows() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (NULL)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE a != NULL").unwrap();
    assert_eq!(r.rows.len(), 0);
}

#[test]
fn test_where_gt_null_no_rows() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (NULL)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE a > NULL").unwrap();
    assert_eq!(r.rows.len(), 0);
}

#[test]
fn test_where_lt_null_no_rows() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (NULL)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE a < NULL").unwrap();
    assert_eq!(r.rows.len(), 0);
}

#[test]
fn test_column_alias() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (42)").unwrap();
    let r = e.execute("SELECT a AS my_col FROM t").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_where_false_no_rows() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE 1=0").unwrap();
    assert_eq!(r.rows.len(), 0);
}

#[test]
fn test_where_true_all_rows() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE 1=1").unwrap();
    assert_eq!(r.rows.len(), 1);
}

// ===== IN operator tests =====

#[test]
fn test_in_integer_list() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(2),(3),(4),(5)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE a IN (1, 3, 5)").unwrap();
    assert_eq!(r.rows.len(), 3);
}

#[test]
fn test_in_text_list() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a TEXT)").unwrap();
    e.execute("INSERT INTO t VALUES ('apple'),('banana'),('cherry')").unwrap();
    let r = e.execute("SELECT * FROM t WHERE a IN ('apple', 'cherry')").unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn test_in_no_match() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(2),(3)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE a IN (10, 20, 30)").unwrap();
    assert_eq!(r.rows.len(), 0);
}

#[test]
fn test_not_in() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(2),(3),(4),(5)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE a NOT IN (2, 4)").unwrap();
    assert_eq!(r.rows.len(), 3);
}

// ===== DISTINCT tests =====

#[test]
fn test_distinct_basic() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(1),(2),(2),(3)").unwrap();
    let r = e.execute("SELECT DISTINCT a FROM t").unwrap();
    assert_eq!(r.rows.len(), 3);
}

#[test]
fn test_distinct_multiple_columns() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER, b TEXT)").unwrap();
    e.execute("INSERT INTO t VALUES (1,'x'),(1,'x'),(1,'y'),(2,'x')").unwrap();
    let r = e.execute("SELECT DISTINCT a, b FROM t").unwrap();
    assert_eq!(r.rows.len(), 3);
}

#[test]
fn test_distinct_with_null() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(NULL),(NULL),(2)").unwrap();
    let r = e.execute("SELECT DISTINCT a FROM t").unwrap();
    assert_eq!(r.rows.len(), 3); // 1, NULL, 2
}
