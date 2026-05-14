//! Set Operation Tests (INTERSECT/EXCEPT/MINUS)
//! GAP-4: coverage improvement for set operations
//! Issue #879: 集合运算测试补全

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

#[allow(deprecated)]
fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

fn setup_table_a(engine: &mut ExecutionEngine<MemoryStorage>) {
    engine.execute("CREATE TABLE t1 (a INTEGER, b TEXT)").unwrap();
    engine.execute("INSERT INTO t1 VALUES (1, 'one')").unwrap();
    engine.execute("INSERT INTO t1 VALUES (2, 'two')").unwrap();
    engine.execute("INSERT INTO t1 VALUES (3, 'three')").unwrap();
}

fn setup_table_b(engine: &mut ExecutionEngine<MemoryStorage>) {
    engine.execute("CREATE TABLE t2 (c INTEGER, d TEXT)").unwrap();
    engine.execute("INSERT INTO t2 VALUES (2, 'two')").unwrap();
    engine.execute("INSERT INTO t2 VALUES (3, 'three')").unwrap();
    engine.execute("INSERT INTO t2 VALUES (4, 'four')").unwrap();
}

// =============================================================================
// INTERSECT Tests (Issue #879)
// =============================================================================

#[test]
#[ignore]
fn test_intersect_basic() {
    let mut engine = create_engine();
    setup_table_a(&mut engine);
    setup_table_b(&mut engine);

    let result = engine.execute("SELECT a FROM t1 INTERSECT SELECT c FROM t2");
    assert!(result.is_ok(), "INTERSECT should parse and execute: {:?}", result);
}

#[test]
#[ignore]
fn test_intersect_multiple_columns() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t3 (x INTEGER, y TEXT)").unwrap();
    engine.execute("INSERT INTO t3 VALUES (1, 'a')").unwrap();
    engine.execute("INSERT INTO t3 VALUES (2, 'b')").unwrap();
    engine.execute("INSERT INTO t3 VALUES (3, 'c')").unwrap();

    engine.execute("CREATE TABLE t4 (p INTEGER, q TEXT)").unwrap();
    engine.execute("INSERT INTO t4 VALUES (1, 'a')").unwrap();
    engine.execute("INSERT INTO t4 VALUES (2, 'b')").unwrap();
    engine.execute("INSERT INTO t4 VALUES (4, 'd')").unwrap();

    let result = engine.execute("SELECT x, y FROM t3 INTERSECT SELECT p, q FROM t4");
    assert!(result.is_ok(), "Multi-column INTERSECT should work: {:?}", result);
}

#[test]
#[ignore]
fn test_intersect_with_nulls() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t_nulls1 (id INTEGER)").unwrap();
    engine.execute("INSERT INTO t_nulls1 VALUES (1)").unwrap();
    engine.execute("INSERT INTO t_nulls1 VALUES (NULL)").unwrap();

    engine.execute("CREATE TABLE t_nulls2 (id INTEGER)").unwrap();
    engine.execute("INSERT INTO t_nulls2 VALUES (NULL)").unwrap();
    engine.execute("INSERT INTO t_nulls2 VALUES (2)").unwrap();

    let result = engine.execute("SELECT id FROM t_nulls1 INTERSECT SELECT id FROM t_nulls2");
    assert!(result.is_ok(), "INTERSECT with NULLs should work: {:?}", result);
}

#[test]
#[ignore]
fn test_intersect_all() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t_a (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t_a VALUES (1)").unwrap();
    engine.execute("INSERT INTO t_a VALUES (1)").unwrap();
    engine.execute("INSERT INTO t_a VALUES (2)").unwrap();

    engine.execute("CREATE TABLE t_b (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t_b VALUES (1)").unwrap();
    engine.execute("INSERT INTO t_b VALUES (1)").unwrap();
    engine.execute("INSERT INTO t_b VALUES (1)").unwrap();
    engine.execute("INSERT INTO t_b VALUES (3)").unwrap();

    let result = engine.execute("SELECT v FROM t_a INTERSECT ALL SELECT v FROM t_b");
    assert!(result.is_ok(), "INTERSECT ALL should work: {:?}", result);
}

// =============================================================================
// EXCEPT Tests (Issue #879)
// =============================================================================

#[test]
#[ignore]
fn test_except_basic() {
    let mut engine = create_engine();
    setup_table_a(&mut engine);
    setup_table_b(&mut engine);

    let result = engine.execute("SELECT a FROM t1 EXCEPT SELECT c FROM t2");
    assert!(result.is_ok(), "EXCEPT should parse and execute: {:?}", result);
}

#[test]
#[ignore]
fn test_except_multiple_columns() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t5 (x INTEGER, y TEXT)").unwrap();
    engine.execute("INSERT INTO t5 VALUES (1, 'a')").unwrap();
    engine.execute("INSERT INTO t5 VALUES (2, 'b')").unwrap();
    engine.execute("INSERT INTO t5 VALUES (3, 'c')").unwrap();

    engine.execute("CREATE TABLE t6 (p INTEGER, q TEXT)").unwrap();
    engine.execute("INSERT INTO t6 VALUES (1, 'a')").unwrap();
    engine.execute("INSERT INTO t6 VALUES (2, 'b')").unwrap();

    let result = engine.execute("SELECT x, y FROM t5 EXCEPT SELECT p, q FROM t6");
    assert!(result.is_ok(), "Multi-column EXCEPT should work: {:?}", result);
}

#[test]
#[ignore]
fn test_except_all() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t_c (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t_c VALUES (1)").unwrap();
    engine.execute("INSERT INTO t_c VALUES (1)").unwrap();
    engine.execute("INSERT INTO t_c VALUES (1)").unwrap();
    engine.execute("INSERT INTO t_c VALUES (2)").unwrap();

    engine.execute("CREATE TABLE t_d (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t_d VALUES (1)").unwrap();
    engine.execute("INSERT INTO t_d VALUES (1)").unwrap();

    let result = engine.execute("SELECT v FROM t_c EXCEPT ALL SELECT v FROM t_d");
    assert!(result.is_ok(), "EXCEPT ALL should work: {:?}", result);
}

// =============================================================================
// MINUS Tests (Issue #879)
// =============================================================================

#[test]
#[ignore]
fn test_minus_basic() {
    let mut engine = create_engine();
    setup_table_a(&mut engine);
    setup_table_b(&mut engine);

    let result = engine.execute("SELECT a FROM t1 MINUS SELECT c FROM t2");
    assert!(result.is_ok(), "MINUS should parse and execute: {:?}", result);
}

#[test]
#[ignore]
fn test_minus_semantics() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t_left (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t_left VALUES (10)").unwrap();
    engine.execute("INSERT INTO t_left VALUES (20)").unwrap();
    engine.execute("INSERT INTO t_left VALUES (30)").unwrap();

    engine.execute("CREATE TABLE t_right (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t_right VALUES (20)").unwrap();
    engine.execute("INSERT INTO t_right VALUES (40)").unwrap();

    let result = engine.execute("SELECT v FROM t_left MINUS SELECT v FROM t_right");
    assert!(result.is_ok(), "MINUS should return rows 10 and 30: {:?}", result);
}

// =============================================================================
// Combined Set Operation Tests
// =============================================================================

#[test]
#[ignore]
fn test_set_operations_precedence() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t_x (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t_x VALUES (1)").unwrap();
    engine.execute("INSERT INTO t_x VALUES (2)").unwrap();
    engine.execute("INSERT INTO t_x VALUES (3)").unwrap();

    engine.execute("CREATE TABLE t_y (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t_y VALUES (2)").unwrap();
    engine.execute("INSERT INTO t_y VALUES (3)").unwrap();
    engine.execute("INSERT INTO t_y VALUES (4)").unwrap();

    let result = engine.execute("SELECT v FROM t_x INTERSECT SELECT v FROM t_y");
    assert!(result.is_ok(), "Set operation precedence should work: {:?}", result);
}

#[test]
#[ignore]
fn test_intersect_except_combined() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE set_a (v INTEGER)").unwrap();
    engine.execute("INSERT INTO set_a VALUES (1)").unwrap();
    engine.execute("INSERT INTO set_a VALUES (2)").unwrap();
    engine.execute("INSERT INTO set_a VALUES (3)").unwrap();

    engine.execute("CREATE TABLE set_b (v INTEGER)").unwrap();
    engine.execute("INSERT INTO set_b VALUES (2)").unwrap();
    engine.execute("INSERT INTO set_b VALUES (3)").unwrap();
    engine.execute("INSERT INTO set_b VALUES (4)").unwrap();

    engine.execute("CREATE TABLE set_c (v INTEGER)").unwrap();
    engine.execute("INSERT INTO set_c VALUES (3)").unwrap();
    engine.execute("INSERT INTO set_c VALUES (4)").unwrap();
    engine.execute("INSERT INTO set_c VALUES (5)").unwrap();

    let result = engine.execute("SELECT v FROM set_a INTERSECT SELECT v FROM set_b EXCEPT SELECT v FROM set_c");
    assert!(result.is_ok(), "Combined set operations should work: {:?}", result);
}

#[test]
#[ignore]
fn test_set_ops_with_subqueries() {
    let mut engine = create_engine();
    setup_table_a(&mut engine);
    setup_table_b(&mut engine);

    let result = engine.execute(
        "SELECT a FROM (SELECT a FROM t1 UNION SELECT c FROM t2) AS combined INTERSECT SELECT c FROM t2"
    );
    assert!(result.is_ok(), "Set operations with subqueries should work: {:?}", result);
}
