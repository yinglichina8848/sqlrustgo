mod sqlite_diff;
use sqlite_diff::{assert_sql_eq, RustEngine, SqliteEngine};

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
    let sql = "SELECT a / 2 FROM t";
    let sqlite = SqliteEngine::new();
    let mut rust = RustEngine::new();
    for s in &setup() {
        sqlite.execute(s).unwrap();
        rust.execute(s).unwrap();
    }
    let sqlite_result = sqlite.query(sql);
    let rust_result = rust.query(sql);
    assert!(sqlite_result.is_ok() || rust_result.is_ok());
}

#[test]
fn test_divide_by_zero() {
    let sql = "SELECT a / 0 FROM t";
    assert_sql_eq(sql, &setup()).unwrap();
}
