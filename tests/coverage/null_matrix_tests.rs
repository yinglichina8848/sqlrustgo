mod sqlite_diff;
use sqlite_diff::{SqliteEngine, RustEngine, assert_sql_eq};

#[test]
fn test_null_and_true() {
    let sql = "SELECT * FROM t WHERE NULL AND TRUE";
    assert_sql_eq(sql, &[]).unwrap();
}

#[test]
fn test_null_and_false() {
    let sql = "SELECT * FROM t WHERE NULL AND FALSE";
    assert_sql_eq(sql, &[]).unwrap();
}

#[test]
fn test_null_or_true() {
    let sql = "SELECT * FROM t WHERE NULL OR TRUE";
    assert_sql_eq(sql, &[]).unwrap();
}

#[test]
fn test_null_or_false() {
    let sql = "SELECT * FROM t WHERE NULL OR FALSE";
    assert_sql_eq(sql, &[]).unwrap();
}

#[test]
fn test_not_null() {
    let sql = "SELECT * FROM t WHERE NOT NULL";
    assert_sql_eq(sql, &[]).unwrap();
}

#[test]
fn test_null_equals_null() {
    let sql = "SELECT * FROM t WHERE a = NULL";
    assert_sql_eq(sql, &[]).unwrap();
}

#[test]
fn test_is_null() {
    let sql = "SELECT * FROM t WHERE a IS NULL";
    let setup = vec!["CREATE TABLE t(a INT);", "INSERT INTO t VALUES (NULL),(1);"];
    assert_sql_eq(sql, &setup).unwrap();
}

#[test]
fn test_is_not_null() {
    let sql = "SELECT * FROM t WHERE a IS NOT NULL";
    let setup = vec!["CREATE TABLE t(a INT);", "INSERT INTO t VALUES (NULL),(1);"];
    assert_sql_eq(sql, &setup).unwrap();
}