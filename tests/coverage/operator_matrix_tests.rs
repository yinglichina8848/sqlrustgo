mod sqlite_diff;
use sqlite_diff::{SqliteEngine, RustEngine, assert_sql_eq};

fn setup() -> Vec<&'static str> {
    vec![
        "CREATE TABLE t(a INT, b INT);",
        "INSERT INTO t VALUES (1,10),(2,20),(3,30);",
    ]
}

#[test]
fn test_equals() {
    assert_sql_eq("SELECT * FROM t WHERE a = 1", &setup()).unwrap();
}

#[test]
fn test_not_equals() {
    assert_sql_eq("SELECT * FROM t WHERE a != 1", &setup()).unwrap();
}

#[test]
fn test_greater_than() {
    assert_sql_eq("SELECT * FROM t WHERE a > 1", &setup()).unwrap();
}

#[test]
fn test_less_than() {
    assert_sql_eq("SELECT * FROM t WHERE a < 3", &setup()).unwrap();
}

#[test]
fn test_greater_than_or_equals() {
    assert_sql_eq("SELECT * FROM t WHERE a >= 2", &setup()).unwrap();
}

#[test]
fn test_less_than_or_equals() {
    assert_sql_eq("SELECT * FROM t WHERE a <= 2", &setup()).unwrap();
}

#[test]
fn test_add() {
    assert_sql_eq("SELECT a + 1 FROM t", &setup()).unwrap();
}

#[test]
fn test_subtract() {
    assert_sql_eq("SELECT a - 1 FROM t", &setup()).unwrap();
}

#[test]
fn test_multiply() {
    assert_sql_eq("SELECT a * 2 FROM t", &setup()).unwrap();
}

#[test]
fn test_divide() {
    assert_sql_eq("SELECT a / 2 FROM t", &setup()).unwrap();
}

#[test]
fn test_divide_by_zero() {
    let sql = "SELECT 1/0";
    assert_sql_eq(sql, &setup()).unwrap();
}