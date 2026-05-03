mod sqlite_diff;

use sqlite_diff::{assert_sql_eq, RustEngine, SqliteEngine};

fn basic_setup() -> Vec<&'static str> {
    vec![
        "CREATE TABLE t(a INT, b INT);",
        "INSERT INTO t VALUES (1,10),(2,20),(3,30),(NULL,40);",
    ]
}

#[test]
fn test_basic_select() {
    let sql = "SELECT * FROM t";
    assert_sql_eq(sql, &basic_setup()).unwrap();
}

#[test]
fn test_null_and_true() {
    let sql = "SELECT * FROM t WHERE NULL AND TRUE";
    let sqlite = SqliteEngine::new();
    let mut rust = RustEngine::new();
    for stmt in &basic_setup() {
        sqlite.execute(stmt).unwrap();
        rust.execute(stmt).unwrap();
    }
    let sqlite_result = sqlite.query(sql).unwrap();
    let rust_result = rust.query(sql).unwrap();
    assert_eq!(
        sqlite_result, rust_result,
        "NULL AND TRUE should return NULL (filtered)"
    );
}
