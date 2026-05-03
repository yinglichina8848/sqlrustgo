mod sqlite_diff;
use sqlite_diff::{SqliteEngine, RustEngine, assert_sql_eq};

fn setup() -> Vec<&'static str> {
    vec![
        "CREATE TABLE t(a INT, b INT);",
        "INSERT INTO t VALUES (1,10),(2,20),(NULL,30);",
        "CREATE TABLE t2(x INT, y INT);",
        "INSERT INTO t2 VALUES (1,100),(2,200);",
    ]
}

#[test]
fn test_divide_by_zero() {
    let sql = "SELECT 1/0";
    assert_sql_eq(sql, &[]).unwrap();
}

#[test]
fn test_null_in_group_by() {
    let sql = "SELECT a, COUNT(*) FROM t GROUP BY a HAVING a IS NULL";
    assert_sql_eq(sql, &setup()).unwrap();
}

#[test]
fn test_subquery_empty_result() {
    let sql = "SELECT * FROM t WHERE a IN (SELECT x FROM t2 WHERE x > 100)";
    assert_sql_eq(sql, &setup()).unwrap();
}