mod sqlite_diff;

use sqlite_diff::assert_sql_eq;

#[test]
fn test_cross_join_basic() {
    let setup = vec![
        "CREATE TABLE t1(a INT);",
        "CREATE TABLE t2(b INT);",
        "INSERT INTO t1 VALUES (1),(2);",
        "INSERT INTO t2 VALUES (10),(20);",
    ];
    let sql = "SELECT * FROM t1, t2";
    assert_sql_eq(sql, &setup).unwrap();
    // 期望: 2x2=4 行
}

#[test]
fn test_cross_join_3_tables() {
    let setup = vec![
        "CREATE TABLE t1(a INT);",
        "CREATE TABLE t2(b INT);",
        "CREATE TABLE t3(c INT);",
        "INSERT INTO t1 VALUES (1);",
        "INSERT INTO t2 VALUES (10),(20);",
        "INSERT INTO t3 VALUES (100),(200),(300);",
    ];
    let sql = "SELECT * FROM t1, t2, t3";
    assert_sql_eq(sql, &setup).unwrap();
    // 期望: 1x2x3=6 行
}
