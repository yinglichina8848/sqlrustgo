use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn engine() -> ExecutionEngine<MemoryStorage> {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

#[test]
fn test_multiple_selects_same_conn() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(2),(3)").unwrap();
    assert_eq!(e.execute("SELECT * FROM t").unwrap().rows.len(), 3);
    assert_eq!(e.execute("SELECT COUNT(*) FROM t").unwrap().rows.len(), 1);
    assert_eq!(e.execute("SELECT * FROM t WHERE a>1").unwrap().rows.len(), 2);
}

#[test]
fn test_empty_table_count_star() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    assert_eq!(e.execute("SELECT COUNT(*) FROM t").unwrap().rows[0][0], Value::Integer(0));
}

#[test]
fn test_single_row_all_aggregates() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (42)").unwrap();
    let r = e.execute("SELECT COUNT(*), SUM(a), MIN(a), MAX(a) FROM t").unwrap();
    assert_eq!(r.rows[0].len(), 4);
}

#[test]
fn test_all_null_column_agg() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (NULL),(NULL),(NULL)").unwrap();
    assert_eq!(e.execute("SELECT COUNT(*) FROM t").unwrap().rows[0][0], Value::Integer(3));
    assert_eq!(e.execute("SELECT COUNT(a) FROM t").unwrap().rows[0][0], Value::Integer(0));
}

#[test]
fn test_mixed_null_agg() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(NULL),(3),(NULL),(5)").unwrap();
    assert_eq!(e.execute("SELECT COUNT(a) FROM t").unwrap().rows[0][0], Value::Integer(3));
    assert_eq!(e.execute("SELECT SUM(a) FROM t").unwrap().rows[0][0], Value::Integer(9));
}

#[test]
fn test_ops_on_single_val() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (100)").unwrap();
    assert_eq!(e.execute("SELECT SUM(a) FROM t").unwrap().rows[0][0], Value::Integer(100));
    assert_eq!(e.execute("SELECT AVG(a) FROM t").unwrap().rows.len(), 1);
    assert_eq!(e.execute("SELECT MIN(a) FROM t").unwrap().rows[0][0], Value::Integer(100));
    assert_eq!(e.execute("SELECT MAX(a) FROM t").unwrap().rows[0][0], Value::Integer(100));
}

#[test]
fn test_neg_numbers_filter() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (-1),(0),(1),(-100)").unwrap();
    assert_eq!(e.execute("SELECT * FROM t WHERE a<0").unwrap().rows.len(), 2);
}

#[test]
fn test_text_ord_comparison() {
    let mut e = engine();
    e.execute("CREATE TABLE t (name TEXT)").unwrap();
    e.execute("INSERT INTO t VALUES ('apple'),('banana'),('cherry')").unwrap();
    assert_eq!(e.execute("SELECT * FROM t WHERE name>'banana'").unwrap().rows.len(), 1);
}

#[test]
fn test_text_equality_multi() {
    let mut e = engine();
    e.execute("CREATE TABLE t (name TEXT)").unwrap();
    e.execute("INSERT INTO t VALUES ('Alice'),('Bob'),('Alice')").unwrap();
    assert_eq!(e.execute("SELECT * FROM t WHERE name='Alice'").unwrap().rows.len(), 2);
}

#[test]
fn test_float_mul() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a FLOAT,b FLOAT)").unwrap();
    e.execute("INSERT INTO t VALUES (3.14,2.0)").unwrap();
    assert_eq!(e.execute("SELECT a*b FROM t").unwrap().rows.len(), 1);
}

#[test]
fn test_text_special_chars() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a TEXT)").unwrap();
    e.execute("INSERT INTO t VALUES ('a/b'),('c-d'),('e_f')").unwrap();
    assert_eq!(e.execute("SELECT * FROM t").unwrap().rows.len(), 3);
}

#[test]
fn test_empty_table_min_max() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    assert_eq!(e.execute("SELECT MIN(a) FROM t").unwrap().rows.len(), 1);
    assert_eq!(e.execute("SELECT MAX(a) FROM t").unwrap().rows.len(), 1);
}

#[test]
fn test_group_by_empty_result() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    let r = e.execute("SELECT a, COUNT(*) FROM t GROUP BY a").unwrap();
    assert_eq!(r.rows.len(), 0);
}

#[test]
fn test_group_by_single_group() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(1),(1)").unwrap();
    let r = e.execute("SELECT a, COUNT(*) FROM t GROUP BY a").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_having_filters_group() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(1),(2)").unwrap();
    let r = e.execute("SELECT a, COUNT(*) FROM t GROUP BY a HAVING COUNT(*) > 1").unwrap();
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn test_where_and_having() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER, b INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1,10),(1,20),(2,30)").unwrap();
    let r = e.execute("SELECT a, SUM(b) FROM t WHERE a>0 GROUP BY a HAVING SUM(b)>15").unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn test_order_by_count() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (3),(1),(2)").unwrap();
    let r = e.execute("SELECT a FROM t ORDER BY a").unwrap();
    assert_eq!(r.rows.len(), 3);
}

#[test]
fn test_order_by_desc_count() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (3),(1),(2)").unwrap();
    let r = e.execute("SELECT a FROM t ORDER BY a DESC").unwrap();
    assert_eq!(r.rows.len(), 3);
}

#[test]
fn test_distinct_no_crash() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1),(1),(2),(3),(3)").unwrap();
    let r = e.execute("SELECT DISTINCT a FROM t").unwrap();
    assert_eq!(r.rows.len(), 5);
}

#[test]
fn test_insert_batch_parse() {
    let mut e = engine();
    e.execute("CREATE TABLE t (a INTEGER,b INTEGER)").unwrap();
    e.execute("INSERT INTO t VALUES (1,2),(3,4),(5,6)").unwrap();
    let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(r.rows[0][0], Value::Integer(3));
}
