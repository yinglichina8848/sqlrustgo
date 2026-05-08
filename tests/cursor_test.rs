// Cursor Integration Tests (BP2-8)
//! Tests for cursor operations: DECLARE, OPEN, FETCH, CLOSE
//! BP2 Gate: cargo test --test cursor_test

use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_catalog::Catalog;
use std::sync::{Arc, RwLock};

fn create_engine() -> MemoryExecutionEngine {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    MemoryExecutionEngine::with_memory_and_catalog(catalog)
}

/// Test basic cursor lifecycle: DECLARE cur CURSOR FOR SELECT, OPEN, FETCH, CLOSE
#[test]
fn test_cursor_basic_lifecycle() {
    let mut engine = create_engine();

    // Create test table
    let result = engine.execute("CREATE TABLE t1 (id INTEGER, name TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Insert test data
    let result = engine.execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b'), (3, 'c')");
    assert!(result.is_ok(), "INSERT failed: {:?}", result.err());

    // Create procedure with cursor
    let sql = "CREATE PROCEDURE cursor_test() \
        BEGIN \
        DECLARE cur CURSOR FOR SELECT * FROM t1; \
        OPEN cur; \
        FETCH cur INTO v_id, v_name; \
        CLOSE cur; \
    END";
    let result = engine.execute(sql);
    assert!(
        result.is_ok(),
        "CREATE PROCEDURE failed: {:?}",
        result.err()
    );

    // Call procedure
    let result = engine.execute("CALL cursor_test()");
    assert!(result.is_ok(), "CALL failed: {:?}", result.err());
}

/// Test cursor with REPEAT loop and CONTINUE HANDLER FOR NOT FOUND
#[test]
fn test_cursor_with_loop() {
    let mut engine = create_engine();

    // Create test table
    let result = engine.execute("CREATE TABLE t2 (id INTEGER, val TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Insert test data
    let result = engine.execute("INSERT INTO t2 VALUES (10, 'x'), (20, 'y')");
    assert!(result.is_ok(), "INSERT failed: {:?}", result.err());

    // Create procedure that loops through cursor
    let sql = "CREATE PROCEDURE loop_cursor() \
        BEGIN \
        DECLARE done INT DEFAULT 0; \
        DECLARE cur CURSOR FOR SELECT * FROM t2; \
        DECLARE CONTINUE HANDLER FOR NOT FOUND SET done = 1; \
        OPEN cur; \
        REPEAT \
            FETCH cur INTO vid, vval; \
            IF NOT done THEN \
                ITERATE; \
            END IF; \
        UNTIL done END REPEAT; \
        CLOSE cur; \
    END";
    let result = engine.execute(sql);
    assert!(
        result.is_ok(),
        "CREATE PROCEDURE failed: {:?}",
        result.err()
    );

    // Call procedure
    let result = engine.execute("CALL loop_cursor()");
    assert!(result.is_ok(), "CALL failed: {:?}", result.err());
}

/// Test cursor fetch into variables
#[test]
fn test_cursor_fetch_into() {
    let mut engine = create_engine();

    // Create test table
    let result = engine.execute("CREATE TABLE t3 (a INTEGER, b INTEGER)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Insert test data
    let result = engine.execute("INSERT INTO t3 VALUES (100, 200)");
    assert!(result.is_ok(), "INSERT failed: {:?}", result.err());

    // Create procedure with cursor fetch
    let sql = "CREATE PROCEDURE fetch_test() \
        BEGIN \
        DECLARE cur CURSOR FOR SELECT a, b FROM t3; \
        OPEN cur; \
        FETCH cur INTO x, y; \
        CLOSE cur; \
    END";
    let result = engine.execute(sql);
    assert!(
        result.is_ok(),
        "CREATE PROCEDURE failed: {:?}",
        result.err()
    );

    // Call procedure
    let result = engine.execute("CALL fetch_test()");
    assert!(result.is_ok(), "CALL failed: {:?}", result.err());
}

/// Test multiple cursors in one procedure
#[test]
fn test_multiple_cursors() {
    let mut engine = create_engine();

    // Create test tables
    let result = engine.execute("CREATE TABLE t4 (id INTEGER)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    let result = engine.execute("CREATE TABLE t5 (id INTEGER)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Insert test data
    let result = engine.execute("INSERT INTO t4 VALUES (1), (2)");
    assert!(result.is_ok(), "INSERT failed: {:?}", result.err());

    let result = engine.execute("INSERT INTO t5 VALUES (10), (20)");
    assert!(result.is_ok(), "INSERT failed: {:?}", result.err());

    // Create procedure with multiple cursors
    let sql = "CREATE PROCEDURE multi_cursor() \
        BEGIN \
        DECLARE cur1 CURSOR FOR SELECT * FROM t4; \
        DECLARE cur2 CURSOR FOR SELECT * FROM t5; \
        OPEN cur1; \
        OPEN cur2; \
        CLOSE cur1; \
        CLOSE cur2; \
    END";
    let result = engine.execute(sql);
    assert!(
        result.is_ok(),
        "CREATE PROCEDURE failed: {:?}",
        result.err()
    );

    // Call procedure
    let result = engine.execute("CALL multi_cursor()");
    assert!(result.is_ok(), "CALL failed: {:?}", result.err());
}

/// Test cursor with ORDER BY clause
#[test]
fn test_cursor_with_order_by() {
    let mut engine = create_engine();

    // Create test table
    let result = engine.execute("CREATE TABLE t6 (score INTEGER)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Insert test data in random order
    let result = engine.execute("INSERT INTO t6 VALUES (30), (10), (20)");
    assert!(result.is_ok(), "INSERT failed: {:?}", result.err());

    // Create procedure with ordered cursor
    let sql = "CREATE PROCEDURE ordered_cursor() \
        BEGIN \
        DECLARE cur CURSOR FOR SELECT * FROM t6 ORDER BY score; \
        OPEN cur; \
        FETCH cur INTO s1; \
        FETCH cur INTO s2; \
        FETCH cur INTO s3; \
        CLOSE cur; \
    END";
    let result = engine.execute(sql);
    assert!(
        result.is_ok(),
        "CREATE PROCEDURE failed: {:?}",
        result.err()
    );

    // Call procedure
    let result = engine.execute("CALL ordered_cursor()");
    assert!(result.is_ok(), "CALL failed: {:?}", result.err());
}

/// Test cursor with WHERE clause
#[test]
fn test_cursor_with_where() {
    let mut engine = create_engine();

    // Create test table
    let result = engine.execute("CREATE TABLE t7 (id INTEGER, val TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Insert test data
    let result = engine.execute("INSERT INTO t7 VALUES (1, 'a'), (2, 'b'), (3, 'c')");
    assert!(result.is_ok(), "INSERT failed: {:?}", result.err());

    // Create procedure with filtered cursor
    let sql = "CREATE PROCEDURE filtered_cursor() \
        BEGIN \
        DECLARE cur CURSOR FOR SELECT * FROM t7 WHERE id > 1; \
        OPEN cur; \
        FETCH cur INTO vid, vval; \
        CLOSE cur; \
    END";
    let result = engine.execute(sql);
    assert!(
        result.is_ok(),
        "CREATE PROCEDURE failed: {:?}",
        result.err()
    );

    // Call procedure
    let result = engine.execute("CALL filtered_cursor()");
    assert!(result.is_ok(), "CALL failed: {:?}", result.err());
}
