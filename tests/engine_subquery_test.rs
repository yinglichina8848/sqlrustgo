mod sqlite_diff;
use sqlite_diff::{assert_sql_eq, RustEngine, SqliteEngine};

#[test]
fn test_in_basic() {
    let setup = vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (1),(2),(3);",
    ];
    let sql = "SELECT * FROM t WHERE a IN (SELECT a FROM t WHERE a > 1)";
    assert_sql_eq(sql, &setup).unwrap();
    // 期望: 2, 3
}

#[test]
fn test_in_with_null() {
    let setup = vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (1),(2),(NULL);",
    ];
    // 1 IN (1,2,NULL) = TRUE
    // 3 IN (1,2,NULL) = NULL
    let sql = "SELECT a FROM t WHERE a IN (SELECT a FROM t WHERE a < 3)";
    assert_sql_eq(sql, &setup).unwrap();
}

#[test]
fn test_not_in_with_null() {
    let setup = vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (1),(2),(NULL);",
    ];
    // Test: NULL NOT IN (1, 2, NULL) = NULL (not FALSE!)
    // 100 NOT IN (1, 2, NULL) = NULL (not FALSE!) because of NULL
    // WHERE filters out NULL, so result is empty
    let sql = "SELECT a FROM t WHERE a NOT IN (SELECT a FROM t WHERE a < 3)";
    assert_sql_eq(sql, &setup).unwrap();
}

#[test]
fn test_exists() {
    let setup = vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (1),(2),(3);",
    ];
    let sql = "SELECT * FROM t WHERE EXISTS (SELECT 1 FROM t WHERE a > 1)";
    assert_sql_eq(sql, &setup).unwrap();
    // EXISTS 返回 TRUE，因为子查询有结果
}

#[test]
fn test_not_exists() {
    let setup = vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (1),(2),(3);",
    ];
    let sql = "SELECT * FROM t WHERE NOT EXISTS (SELECT 1 FROM t WHERE a > 100)";
    assert_sql_eq(sql, &setup).unwrap();
    // NOT EXISTS 返回 TRUE，因为子查询无结果
}

#[test]
fn test_in_value_list() {
    let setup = vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (1),(2),(3);",
    ];
    let sql = "SELECT * FROM t WHERE a IN (1, 2)";
    assert_sql_eq(sql, &setup).unwrap();
}

#[test]
fn test_not_in_value_list() {
    let setup = vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (1),(2),(3);",
    ];
    let sql = "SELECT * FROM t WHERE a NOT IN (1, 2)";
    assert_sql_eq(sql, &setup).unwrap();
}

#[test]
fn test_in_value_list_with_null() {
    let setup = vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (1),(2),(3);",
    ];
    // 3 NOT IN (1,2,NULL) = NULL (不是 FALSE!)
    let sql = "SELECT a FROM t WHERE a NOT IN (1, 2, NULL)";
    assert_sql_eq(sql, &setup).unwrap();
}

#[test]
fn test_in_empty_result() {
    let setup = vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (1),(2),(3);",
    ];
    // 子查询返回空，所以 1 IN (empty) = FALSE
    let sql = "SELECT * FROM t WHERE a IN (SELECT a FROM t WHERE a > 100)";
    assert_sql_eq(sql, &setup).unwrap();
}

#[test]
fn test_exists_no_rows() {
    let setup = vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (1),(2),(3);",
    ];
    // EXISTS 子查询无结果返回空
    let sql = "SELECT * FROM t WHERE EXISTS (SELECT 1 FROM t WHERE a > 100)";
    assert_sql_eq(sql, &setup).unwrap();
}

#[test]
fn test_not_exists_no_rows() {
    let setup = vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (1),(2),(3);",
    ];
    // NOT EXISTS 子查询无结果，返回所有行
    let sql = "SELECT * FROM t WHERE NOT EXISTS (SELECT 1 FROM t WHERE a > 100)";
    assert_sql_eq(sql, &setup).unwrap();
}
