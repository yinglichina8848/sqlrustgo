//! DDL Statement Tests
//! GAP-5: coverage improvement for DDL statements
//! Issue #875: DDL 语句测试覆盖补全 (8 个缺口)

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

#[allow(deprecated)]
fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

// =============================================================================
// TRUNCATE Tests (Issue #875)
// =============================================================================

#[test]
#[ignore]
fn test_truncate_basic() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, name TEXT)")
        .unwrap();
    engine.execute("INSERT INTO t1 VALUES (1, 'one')").unwrap();
    engine.execute("INSERT INTO t1 VALUES (2, 'two')").unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (3, 'three')")
        .unwrap();

    let result = engine.execute("TRUNCATE TABLE t1");
    assert!(result.is_ok(), "TRUNCATE should work: {:?}", result);

    let count = engine.execute("SELECT COUNT(*) FROM t1").unwrap();
    assert_eq!(count.rows[0][0], sqlrustgo_types::Value::Integer(0));
}

#[test]
#[ignore]
fn test_truncate_with_restart_identity() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t2 (id INTEGER PRIMARY KEY AUTO_INCREMENT, name TEXT)")
        .unwrap();
    engine.execute("INSERT INTO t2 VALUES (1, 'one')").unwrap();
    engine.execute("INSERT INTO t2 VALUES (2, 'two')").unwrap();

    let result = engine.execute("TRUNCATE TABLE t2 RESTART IDENTITY");
    assert!(
        result.is_ok(),
        "TRUNCATE RESTART IDENTITY should work: {:?}",
        result
    );

    let count = engine.execute("SELECT COUNT(*) FROM t2").unwrap();
    assert_eq!(count.rows[0][0], sqlrustgo_types::Value::Integer(0));
}

#[test]
#[ignore]
fn test_truncate_cascade() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE parent (id INTEGER PRIMARY KEY)")
        .unwrap();
    engine.execute("INSERT INTO parent VALUES (1)").unwrap();

    let result = engine.execute("TRUNCATE TABLE parent CASCADE");
    assert!(result.is_ok(), "TRUNCATE CASCADE should work: {:?}", result);
}

#[test]
#[ignore]
fn test_truncate_restrict() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t3 (id INTEGER PRIMARY KEY)")
        .unwrap();
    engine.execute("INSERT INTO t3 VALUES (1)").unwrap();

    let result = engine.execute("TRUNCATE TABLE t3 RESTRICT");
    assert!(
        result.is_ok(),
        "TRUNCATE RESTRICT should work: {:?}",
        result
    );
}

// =============================================================================
// RENAME TABLE Tests (Issue #875)
// =============================================================================

#[test]
#[ignore]
fn test_rename_table_basic() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE old_name (id INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO old_name VALUES (1)").unwrap();

    let result = engine.execute("RENAME TABLE old_name TO new_name");
    assert!(result.is_ok(), "RENAME TABLE should work: {:?}", result);

    let check = engine.execute("SELECT * FROM new_name");
    assert!(check.is_ok(), "Table should be renamed: {:?}", check);
}

#[test]
#[ignore]
fn test_rename_table_multiple() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
    engine.execute("CREATE TABLE t2 (id INTEGER)").unwrap();

    let result = engine.execute("RENAME TABLE t1 TO t1_new, t2 TO t2_new");
    assert!(
        result.is_ok(),
        "RENAME multiple tables should work: {:?}",
        result
    );
}

#[test]
#[ignore]
fn test_rename_table_to_existing() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
    engine.execute("CREATE TABLE t2 (id INTEGER)").unwrap();

    let result = engine.execute("RENAME TABLE t1 TO t2");
    assert!(result.is_err(), "RENAME to existing table should fail");
}

// =============================================================================
// CREATE/DROP VIEW Tests (Issue #875)
// =============================================================================

#[test]
#[ignore]
fn test_create_view_basic() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1, 'Alice')")
        .unwrap();
    engine.execute("INSERT INTO t1 VALUES (2, 'Bob')").unwrap();

    let result = engine.execute("CREATE VIEW v1 AS SELECT id, name FROM t1 WHERE id = 1");
    assert!(result.is_ok(), "CREATE VIEW should work: {:?}", result);

    let view_data = engine.execute("SELECT * FROM v1");
    assert!(
        view_data.is_ok(),
        "View should be queryable: {:?}",
        view_data
    );
}

#[test]
#[ignore]
fn test_drop_view_basic() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
    engine
        .execute("CREATE VIEW v1 AS SELECT * FROM t1")
        .unwrap();

    let result = engine.execute("DROP VIEW v1");
    assert!(result.is_ok(), "DROP VIEW should work: {:?}", result);
}

#[test]
#[ignore]
fn test_drop_view_if_exists() {
    let mut engine = create_engine();
    let result = engine.execute("DROP VIEW IF EXISTS nonexistent_view");
    assert!(
        result.is_ok(),
        "DROP VIEW IF EXISTS should not error: {:?}",
        result
    );
}

#[test]
#[ignore]
fn test_create_view_with_aggregation() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE sales (region TEXT, amount INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES ('North', 100)")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES ('North', 200)")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES ('South', 150)")
        .unwrap();

    let result = engine.execute(
        "CREATE VIEW regional_sales AS SELECT region, SUM(amount) as total FROM sales GROUP BY region"
    );
    assert!(
        result.is_ok(),
        "CREATE VIEW with aggregation should work: {:?}",
        result
    );
}

// =============================================================================
// CREATE/DROP DATABASE Tests (Issue #875)
// =============================================================================

#[test]
#[ignore]
fn test_create_database_basic() {
    let mut engine = create_engine();
    let result = engine.execute("CREATE DATABASE mydb");
    assert!(result.is_ok(), "CREATE DATABASE should work: {:?}", result);
}

#[test]
#[ignore]
fn test_create_database_if_not_exists() {
    let mut engine = create_engine();
    let result1 = engine.execute("CREATE DATABASE IF NOT EXISTS existing_db");
    assert!(
        result1.is_ok(),
        "CREATE DATABASE IF NOT EXISTS should work: {:?}",
        result1
    );

    let result2 = engine.execute("CREATE DATABASE IF NOT EXISTS existing_db");
    assert!(
        result2.is_ok(),
        "Second CREATE should also work: {:?}",
        result2
    );
}

#[test]
#[ignore]
fn test_drop_database_basic() {
    let mut engine = create_engine();
    let result = engine.execute("DROP DATABASE mydb");
    assert!(result.is_ok(), "DROP DATABASE should work: {:?}", result);
}

#[test]
#[ignore]
fn test_drop_database_if_exists() {
    let mut engine = create_engine();
    let result = engine.execute("DROP DATABASE IF EXISTS nonexistent_db");
    assert!(
        result.is_ok(),
        "DROP DATABASE IF EXISTS should not error: {:?}",
        result
    );
}

// =============================================================================
// ALTER TABLE DROP/MODIFY Tests (Issue #875)
// =============================================================================

#[test]
fn test_alter_table_drop_column() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, name TEXT, age INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1, 'Alice', 30)")
        .unwrap();

    let result = engine.execute("ALTER TABLE t1 DROP COLUMN age");
    assert!(
        result.is_ok(),
        "ALTER TABLE DROP COLUMN should work: {:?}",
        result
    );

    let desc = engine.execute("DESCRIBE t1").unwrap();
    assert!(desc
        .rows
        .iter()
        .all(|r| r[0] != sqlrustgo_types::Value::Text("age".to_string())));
}

#[test]
fn test_alter_table_drop_multiple_columns() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t2 (a INTEGER, b TEXT, c INTEGER, d TEXT)")
        .unwrap();

    let result = engine.execute("ALTER TABLE t2 DROP COLUMN b, DROP COLUMN c");
    assert!(
        result.is_ok(),
        "DROP multiple columns should work: {:?}",
        result
    );
}

#[test]
#[ignore]
fn test_alter_table_modify_column_type() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t3 (id INTEGER, value TEXT)")
        .unwrap();

    let result = engine.execute("ALTER TABLE t3 MODIFY COLUMN value VARCHAR(100)");
    assert!(result.is_ok(), "MODIFY COLUMN should work: {:?}", result);
}

#[test]
#[ignore]
fn test_alter_table_modify_column_not_null() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t4 (id INTEGER, name TEXT)")
        .unwrap();

    let result = engine.execute("ALTER TABLE t4 MODIFY COLUMN name TEXT NOT NULL");
    assert!(
        result.is_ok(),
        "MODIFY to NOT NULL should work: {:?}",
        result
    );
}

#[test]
#[ignore]
fn test_alter_table_add_constraint() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t5 (id INTEGER, email TEXT)")
        .unwrap();

    let result = engine.execute("ALTER TABLE t5 MODIFY COLUMN email TEXT UNIQUE");
    assert!(
        result.is_ok(),
        "MODIFY with UNIQUE constraint should work: {:?}",
        result
    );
}
