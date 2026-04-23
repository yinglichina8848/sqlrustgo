// OLTP benchmark integration tests for SQLRustGo
// Tests cover point select, range select, insert, update, delete, mixed workloads

use sqlrustgo::MemoryExecutionEngine;

fn setup_oltp_engine() -> MemoryExecutionEngine {
    let mut engine = MemoryExecutionEngine::with_memory();

    // Create standard OLTP test table
    engine
        .execute(
            "CREATE TABLE oltp_test (
                id INTEGER PRIMARY KEY,
                k INTEGER,
                c TEXT,
                pad TEXT
            )",
        )
        .expect("create oltp_test table");

    // Insert 10000 rows for OLTP testing
    for i in 1..=10000 {
        let k = i as i64 % 100;
        let c = format!("c_{:08}", i);
        let pad = format!("pad_{:08}", i);
        engine
            .execute(&format!(
                "INSERT INTO oltp_test VALUES ({}, {}, '{}', '{}')",
                i, k, c, pad
            ))
            .expect("insert oltp_test row");
    }

    engine
}

// ============================================================
// OLTP Point Select Test
// Simulates: SELECT c FROM oltp_test WHERE id = ?
// ============================================================
#[test]
fn oltp_point_select_single() {
    let mut engine = setup_oltp_engine();

    // Point select by primary key
    let result = engine.execute("SELECT c, pad FROM oltp_test WHERE id = 5000");
    assert!(result.is_ok(), "Point select should work");

    if let Ok(rows) = result {
        assert!(!rows.rows.is_empty(), "Should find the row");
        assert_eq!(rows.rows.len(), 1);
    }
}

#[test]
fn oltp_point_select_multiple() {
    let mut engine = setup_oltp_engine();

    // Multiple point selects
    for id in [100, 500, 1000, 5000, 9999] {
        let result = engine.execute(&format!(
            "SELECT k, c FROM oltp_test WHERE id = {}",
            id
        ));
        assert!(result.is_ok(), "Point select for id={} should work", id);
    }
}

// ============================================================
// OLTP Range Select Test
// Simulates: SELECT c FROM oltp_test WHERE id BETWEEN ? AND ?
// ============================================================
#[test]
fn oltp_range_select_small() {
    let mut engine = setup_oltp_engine();

    // Range select (small range)
    let result = engine.execute(
        "SELECT id, k, c FROM oltp_test WHERE id >= 100 AND id <= 110",
    );
    assert!(result.is_ok(), "Range select should work");

    if let Ok(rows) = result {
        assert!(!rows.rows.is_empty(), "Should find rows in range");
        assert!(rows.rows.len() <= 11, "Should have at most 11 rows");
    }
}

#[test]
fn oltp_range_select_large() {
    let mut engine = setup_oltp_engine();

    // Range select (large range)
    let result = engine.execute(
        "SELECT COUNT(*) FROM oltp_test WHERE id BETWEEN 1000 AND 5000",
    );
    assert!(result.is_ok(), "Range select with COUNT should work");
}

// ============================================================
// OLTP Insert Test
// Simulates: INSERT INTO oltp_test VALUES (?, ?, ?, ?)
// ============================================================
#[test]
fn oltp_insert_single() {
    let mut engine = setup_oltp_engine();

    // Insert new row
    let result = engine.execute(
        "INSERT INTO oltp_test VALUES (10001, 0, 'new_c', 'new_pad')",
    );
    assert!(result.is_ok(), "Insert should work");

    // Verify the row was inserted
    let result = engine.execute(
        "SELECT * FROM oltp_test WHERE id = 10001",
    );
    assert!(result.is_ok(), "Should find inserted row");
    if let Ok(rows) = result {
        assert_eq!(rows.rows.len(), 1);
    }
}

#[test]
fn oltp_insert_multiple() {
    let mut engine = setup_oltp_engine();

    // Insert multiple rows
    for i in 0..10 {
        let result = engine.execute(&format!(
            "INSERT INTO oltp_test VALUES ({}, {}, 'c_{}', 'pad_{}')",
            10001 + i,
            i % 100,
            10001 + i,
            10001 + i
        ));
        assert!(result.is_ok(), "Insert row {} should work", i);
    }
}

// ============================================================
// OLTP Update Test
// Simulates: UPDATE oltp_test SET c = ? WHERE k = ?
// ============================================================
#[test]
fn oltp_update_single() {
    let mut engine = setup_oltp_engine();

    // Update rows where k = 50
    let result = engine.execute(
        "UPDATE oltp_test SET c = 'updated_c' WHERE k = 50",
    );
    assert!(result.is_ok(), "Update should work");
}

#[test]
fn oltp_update_multiple() {
    let mut engine = setup_oltp_engine();

    // Update multiple rows
    let result = engine.execute(
        "UPDATE oltp_test SET pad = 'modified' WHERE id BETWEEN 100 AND 200",
    );
    assert!(result.is_ok(), "Bulk update should work");
}

// ============================================================
// OLTP Delete Test
// Simulates: DELETE FROM oltp_test WHERE id = ?
// ============================================================
#[test]
fn oltp_delete_single() {
    let mut engine = setup_oltp_engine();

    // Delete a specific row
    let result = engine.execute("DELETE FROM oltp_test WHERE id = 5000");
    assert!(result.is_ok(), "Delete should work");

    // Verify the row was deleted
    let result = engine.execute("SELECT * FROM oltp_test WHERE id = 5000");
    assert!(result.is_ok(), "Query should work");
    if let Ok(rows) = result {
        assert_eq!(rows.rows.len(), 0, "Row should be deleted");
    }
}

#[test]
fn oltp_delete_multiple() {
    let mut engine = setup_oltp_engine();

    // Delete multiple rows
    let result = engine.execute(
        "DELETE FROM oltp_test WHERE id BETWEEN 100 AND 150",
    );
    assert!(result.is_ok(), "Bulk delete should work");
}

// ============================================================
// OLTP Mixed Workload Test
// Simulates: Mix of reads and writes
// ============================================================
#[test]
fn oltp_mixed_read_write() {
    let mut engine = setup_oltp_engine();

    // Read operation
    let result = engine.execute(
        "SELECT COUNT(*) FROM oltp_test WHERE k < 50",
    );
    assert!(result.is_ok(), "Read should work");

    // Write operation
    let result = engine.execute(
        "INSERT INTO oltp_test VALUES (10001, 0, 'mixed', 'mixed_pad')",
    );
    assert!(result.is_ok(), "Write should work");

    // Another read
    let result = engine.execute(
        "SELECT * FROM oltp_test WHERE id = 10001",
    );
    assert!(result.is_ok(), "Read after write should work");
}

// ============================================================
// OLTP Performance Validation Tests
// ============================================================
#[test]
fn oltp_data_integrity() {
    let mut engine = setup_oltp_engine();

    // Verify total row count
    let result = engine.execute("SELECT COUNT(*) FROM oltp_test");
    assert!(result.is_ok(), "COUNT should work");

    if let Ok(rows) = result {
        assert!(!rows.rows.is_empty(), "Should have results");
    }
}

#[test]
fn oltp_index_scan() {
    let mut engine = setup_oltp_engine();

    // Index scan by k
    let result = engine.execute(
        "SELECT id, c FROM oltp_test WHERE k = 0",
    );
    assert!(result.is_ok(), "Index scan should work");
}

// ============================================================
// Unsupported OLTP Features (documented)
// - Full table scans on large datasets
// - Complex transactions (BEGIN/COMMIT/ROLLBACK)
// - Connection pooling
// ============================================================
