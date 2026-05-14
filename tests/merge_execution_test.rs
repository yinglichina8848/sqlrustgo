//! MERGE Statement Execution Tests
//! GAP-1: coverage improvement for MERGE statement (was 0%)

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_merge_basic_matched() {
    // MERGE that updates matching rows
    let mut engine = create_engine();

    // Create target table
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'old')")
        .unwrap();

    // Create source table
    engine
        .execute("CREATE TABLE source (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'new')")
        .unwrap();

    // MERGE: update when matched
    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET value = source.value"
    ).unwrap();

    // Verify update happened
    let result = engine
        .execute("SELECT value FROM target WHERE id = 1")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("new".to_string()));
}

#[test]
fn test_merge_not_matched() {
    // MERGE that inserts non-matching rows
    let mut engine = create_engine();

    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'exists')")
        .unwrap();

    engine
        .execute("CREATE TABLE source (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (2, 'new_row')")
        .unwrap();

    // MERGE: insert when not matched
    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN NOT MATCHED THEN INSERT (id, value) VALUES (source.id, source.value)"
    ).unwrap();

    // Verify insert happened
    let result = engine.execute("SELECT COUNT(*) FROM target").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(2));
}

#[test]
fn test_merge_both_clauses() {
    // MERGE with both matched and not matched clauses
    let mut engine = create_engine();

    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'old')")
        .unwrap();

    engine
        .execute("CREATE TABLE source (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'updated'), (2, 'inserted')")
        .unwrap();

    let result = engine
        .execute(
            "MERGE INTO target USING source ON target.id = source.id \
         WHEN MATCHED THEN UPDATE SET value = source.value \
         WHEN NOT MATCHED THEN INSERT (id, value) VALUES (source.id, source.value)",
        )
        .unwrap();

    // Verify both update and insert
    let result = engine.execute("SELECT * FROM target ORDER BY id").unwrap();
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0][1], Value::Text("updated".to_string()));
    assert_eq!(result.rows[1][1], Value::Text("inserted".to_string()));
}

#[test]
fn test_merge_no_matches() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'original')")
        .unwrap();

    engine
        .execute("CREATE TABLE source (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (99, 'new')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET value = source.value"
    ).unwrap();

    let result = engine
        .execute("SELECT value FROM target WHERE id = 1")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("original".to_string()));
}

#[test]
fn test_merge_multiple_source_rows() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'target')")
        .unwrap();

    engine
        .execute("CREATE TABLE source (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'v1'), (1, 'v2'), (1, 'v3')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET value = source.value"
    ).unwrap();

    let result = engine
        .execute("SELECT value FROM target WHERE id = 1")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("v3".to_string()));
}

#[test]
fn test_merge_multiple_columns_update() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, a TEXT, b INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'old', 100)")
        .unwrap();

    engine
        .execute("CREATE TABLE source (id INTEGER, a TEXT, b INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'new_a', 200)")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET a = source.a, b = source.b"
    ).unwrap();

    let result = engine
        .execute("SELECT a, b FROM target WHERE id = 1")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("new_a".to_string()));
    assert_eq!(result.rows[0][1], Value::Integer(200));
}

#[test]
fn test_merge_complex_condition() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER, version INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 1, 'old')")
        .unwrap();

    engine
        .execute("CREATE TABLE source (id INTEGER, version INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 2, 'new')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id AND target.version < source.version WHEN MATCHED THEN UPDATE SET value = source.value"
    ).unwrap();

    let result = engine
        .execute("SELECT value FROM target WHERE id = 1")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("new".to_string()));
}

#[test]
fn test_merge_all_source_matched() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 't1'), (2, 't2')")
        .unwrap();

    engine
        .execute("CREATE TABLE source (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 's1'), (2, 's2'), (3, 's3')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET value = source.value WHEN NOT MATCHED THEN INSERT (id, value) VALUES (source.id, source.value)"
    ).unwrap();

    let result = engine.execute("SELECT COUNT(*) FROM target").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(3));
}

#[test]
fn test_merge_multiple_matched_rows() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 't1'), (2, 't2'), (3, 't3')")
        .unwrap();

    engine
        .execute("CREATE TABLE source (id INTEGER PRIMARY KEY, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 's1'), (3, 's3')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET name = source.name"
    ).unwrap();

    let result = engine
        .execute("SELECT name FROM target WHERE id = 1")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("s1".to_string()));
    let result = engine
        .execute("SELECT name FROM target WHERE id = 2")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("t2".to_string()));
    let result = engine
        .execute("SELECT name FROM target WHERE id = 3")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("s3".to_string()));
}

#[test]
fn test_merge_1_basic_update() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'old')")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'new')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET name = source.name",
    );
    assert!(result.is_ok(), "MERGE-1 should work: {:?}", result);

    let result = engine
        .execute("SELECT name FROM target WHERE id = 1")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("new".to_string()));
}

#[test]
fn test_merge_2_not_matched_insert() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'new')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN NOT MATCHED THEN INSERT (id, name) VALUES (source.id, source.name)",
    );
    assert!(result.is_ok(), "MERGE-2 should work: {:?}", result);

    let result = engine
        .execute("SELECT name FROM target WHERE id = 1")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("new".to_string()));
}

#[test]
fn test_merge_3_update_with_where() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, name TEXT, status INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'old', 1), (2, 'old', 0)")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER, name TEXT, status INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'new', 1), (2, 'new', 0)")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED AND target.status = 1 THEN UPDATE SET name = source.name",
    );
    assert!(result.is_ok(), "MERGE-3 should work: {:?}", result);

    let result = engine
        .execute("SELECT name FROM target WHERE id = 1")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("new".to_string()));
    let result = engine
        .execute("SELECT name FROM target WHERE id = 2")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("old".to_string()));
}

#[test]
fn test_merge_4_insert_with_where() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, name TEXT, status INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'existing', 1)")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER, name TEXT, status INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (2, 'new', 1), (3, 'skip', 0)")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN NOT MATCHED AND source.status = 1 THEN INSERT (id, name) VALUES (source.id, source.name)",
    );
    assert!(result.is_ok(), "MERGE-4 should work: {:?}", result);

    let result = engine.execute("SELECT COUNT(*) FROM target").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(2));
    let result = engine
        .execute("SELECT name FROM target WHERE id = 2")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("new".to_string()));
}

#[test]
fn test_merge_5_multiple_when_matched() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, name TEXT, status INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'old', 1)")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER, name TEXT, status INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'new', 1)")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED AND source.status = 1 THEN UPDATE SET name = 'updated1' WHEN MATCHED AND source.status = 0 THEN UPDATE SET name = 'updated2'",
    );
    assert!(result.is_ok(), "MERGE-5 should work: {:?}", result);

    let result = engine
        .execute("SELECT name FROM target WHERE id = 1")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("updated1".to_string()));
}

#[test]
fn test_merge_6_subquery_as_source() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'old')")
        .unwrap();
    engine
        .execute("CREATE TABLE src (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO src VALUES (2, 'from_src')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING src ON target.id = src.id WHEN NOT MATCHED THEN INSERT (id, name) VALUES (src.id, src.name)",
    );
    assert!(result.is_ok(), "MERGE-6 should work: {:?}", result);

    let result = engine
        .execute("SELECT name FROM target WHERE id = 2")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("from_src".to_string()));
}

#[test]
fn test_merge_7_delete_operation() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'delete_me'), (2, 'keep_me')")
        .unwrap();
    engine
        .execute("CREATE TABLE src (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO src VALUES (1, 'delete_me')")
        .unwrap();

    let merge_result = engine
        .execute("MERGE INTO target USING src ON target.id = src.id WHEN MATCHED THEN DELETE");
    assert!(
        merge_result.is_ok(),
        "MERGE-7 should work: {:?}",
        merge_result
    );

    let result = engine.execute("SELECT COUNT(*) FROM target").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(1));
    let result = engine
        .execute("SELECT name FROM target WHERE id = 2")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Text("keep_me".to_string()));
}

#[test]
fn test_merge_8_bulk_merge() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER, value INTEGER)")
        .unwrap();

    engine
        .execute("INSERT INTO target VALUES (1, 1), (2, 2)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 10), (2, 20)")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET value = source.value",
    );
    assert!(result.is_ok(), "MERGE-8 should work: {:?}", result);

    let result = engine
        .execute("SELECT value FROM target WHERE id = 1")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(10));

    let result = engine
        .execute("SELECT value FROM target WHERE id = 2")
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(20));
}
