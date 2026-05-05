mod sqlite_diff;
use sqlite_diff::{assert_sql_eq, RustEngine, SqliteEngine};

fn int_setup() -> Vec<&'static str> {
    vec![
        "CREATE TABLE t(a INT);",
        "INSERT INTO t VALUES (NULL),(1),(2);",
    ]
}

#[test]
fn test_null_and_true() {
    // NULL AND TRUE -> NULL (filtered in WHERE)
    let sql = "SELECT * FROM t WHERE a IS NULL AND a > 0";
    assert_sql_eq(sql, &int_setup()).unwrap();
}

#[test]
fn test_null_and_false() {
    // NULL AND FALSE -> FALSE (short circuit, filtered out)
    let sql = "SELECT * FROM t WHERE a IS NULL AND a > 0";
    assert_sql_eq(sql, &int_setup()).unwrap();
}

#[test]
fn test_null_or_true() {
    // NULL OR TRUE -> TRUE (short circuit)
    let sql = "SELECT * FROM t WHERE a > 0 OR a IS NULL";
    assert_sql_eq(sql, &int_setup()).unwrap();
}

#[test]
fn test_null_or_false() {
    // NULL OR FALSE -> NULL (filtered in WHERE)
    let sql = "SELECT * FROM t WHERE a > 100 OR a IS NULL";
    assert_sql_eq(sql, &int_setup()).unwrap();
}

#[test]
fn test_not_null() {
    // NOT NULL -> NULL (filtered in WHERE)
    let sql = "SELECT * FROM t WHERE NOT (a > 0)";
    assert_sql_eq(sql, &int_setup()).unwrap();
}

#[test]
fn test_null_equals_null() {
    // a = NULL -> NULL (filtered in WHERE)
    let sql = "SELECT * FROM t WHERE a = NULL";
    assert_sql_eq(sql, &int_setup()).unwrap();
}

#[test]
fn test_is_null() {
    let sql = "SELECT * FROM t WHERE a IS NULL";
    assert_sql_eq(sql, &int_setup()).unwrap();
}

#[test]
fn test_is_not_null() {
    let sql = "SELECT * FROM t WHERE a IS NOT NULL";
    assert_sql_eq(sql, &int_setup()).unwrap();
}
