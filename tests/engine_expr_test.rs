mod sqlite_diff;

use sqlite_diff::{assert_sql_eq, RustEngine, SqliteEngine};

fn basic_setup() -> Vec<&'static str> {
    vec![
        "CREATE TABLE t(a INT, b INT);",
        "INSERT INTO t VALUES (1,10),(2,20),(3,30),(NULL,40);",
    ]
}

fn setup_with_data() -> (SqliteEngine, RustEngine) {
    let sqlite = SqliteEngine::new();
    let mut rust = RustEngine::new();
    for stmt in &basic_setup() {
        sqlite.execute(stmt).unwrap();
        rust.execute(stmt).unwrap();
    }
    (sqlite, rust)
}

#[test]
fn test_basic_select() {
    let sql = "SELECT * FROM t";
    assert_sql_eq(sql, &basic_setup()).unwrap();
}

#[test]
fn test_null_and_true() {
    let sql = "SELECT * FROM t WHERE a IS NULL AND 1=1";
    let (sqlite, mut rust) = setup_with_data();
    let sqlite_result = sqlite.query(sql).unwrap();
    let rust_result = rust.query(sql).unwrap();
    assert_eq!(sqlite_result, rust_result);
}

#[test]
fn test_null_or_false() {
    let sql = "SELECT * FROM t WHERE a IS NULL OR 1=0";
    let (sqlite, mut rust) = setup_with_data();
    let sqlite_result = sqlite.query(sql).unwrap();
    let rust_result = rust.query(sql).unwrap();
    assert_eq!(sqlite_result, rust_result);
}

#[test]
fn test_null_not() {
    let sql = "SELECT * FROM t WHERE NOT (a > 0)";
    let (sqlite, mut rust) = setup_with_data();
    let sqlite_result = sqlite.query(sql).unwrap();
    let rust_result = rust.query(sql).unwrap();
    assert_eq!(sqlite_result, rust_result);
}

#[test]
fn test_arithmetic_expr() {
    let sql = "SELECT a, b FROM t WHERE a + b > 15";
    let (sqlite, mut rust) = setup_with_data();
    let sqlite_result = sqlite.query(sql).unwrap();
    let rust_result = rust.query(sql).unwrap();
    assert_eq!(sqlite_result, rust_result);
}

#[test]
fn test_complex_where() {
    let sql = "SELECT * FROM t WHERE b < 30 OR a IS NULL";
    let (sqlite, mut rust) = setup_with_data();
    let sqlite_result = sqlite.query(sql).unwrap();
    let rust_result = rust.query(sql).unwrap();
    assert_eq!(sqlite_result, rust_result);
}

#[test]
fn test_short_circuit_and() {
    let sql = "SELECT * FROM t WHERE 1=0 AND 1/0 > 0";
    let (sqlite, mut rust) = setup_with_data();
    let sqlite_result = sqlite.query(sql).unwrap();
    let rust_result = rust.query(sql).unwrap();
    assert_eq!(sqlite_result, rust_result);
}

#[test]
fn test_short_circuit_or() {
    let sql = "SELECT * FROM t WHERE 1=1 OR 1/0 > 0";
    let (sqlite, mut rust) = setup_with_data();
    let sqlite_result = sqlite.query(sql).unwrap();
    let rust_result = rust.query(sql).unwrap();
    assert_eq!(sqlite_result, rust_result);
}

#[test]
fn test_projection_arithmetic() {
    let setup = vec![
        "CREATE TABLE t(a INT, b INT);",
        "INSERT INTO t VALUES (1,10),(2,20),(3,30);",
    ];
    let sql = "SELECT a + 1, a * 2, a + b FROM t WHERE a > 1";
    assert_sql_eq(sql, &setup).unwrap();
    // 期望: (2,4,12), (3,6,33)
}

#[test]
fn test_projection_with_null() {
    let setup = vec![
        "CREATE TABLE t(a INT, b INT);",
        "INSERT INTO t VALUES (1,10),(NULL,20),(3,NULL);",
    ];
    let sql = "SELECT a + 1, a * 2 FROM t";
    assert_sql_eq(sql, &setup).unwrap();
    // NULL 参与运算应该产生 NULL
}

#[test]
fn test_projection_complex() {
    let setup = vec![
        "CREATE TABLE t(a INT, b INT);",
        "INSERT INTO t VALUES (1,10),(2,20);",
    ];
    let sql = "SELECT a + b * 2, a * b + 2 FROM t";
    assert_sql_eq(sql, &setup).unwrap();
    // 验证运算优先级: a + b * 2 = a + (b * 2), a * b + 2 = (a * b) + 2
}
