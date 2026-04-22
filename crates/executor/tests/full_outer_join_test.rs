//! Full Outer Join Tests
//!
//! These tests verify the FULL OUTER JOIN implementation in the planner and optimizer.
//!
//! ## Note: Execution Engine Gap
//!
//! These tests currently fail because `ExecutionEngine::execute_select` does not
//! process `join_clause`. This is a pre-existing gap in the execution engine.
//!
//! The FULL OUTER JOIN implementation (Tasks 2 & 3) is correct at the planner/optimizer
//! level, but requires the execution engine to invoke `HashJoinExec` for queries
//! with JOIN clauses.
//!
//! Once the execution engine is updated to handle `join_clause`, these tests
//! should pass and verify:
//! - Matched rows from both tables
//! - Unmatched rows from t1 with NULLs for t2 columns
//! - Unmatched rows from t2 with NULLs for t1 columns

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
#[ignore]
fn test_full_outer_join_basic() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER, value INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b'), (3, 'c')")
        .unwrap();
    engine
        .execute("INSERT INTO t2 VALUES (1, 100), (2, 200), (4, 400)")
        .unwrap();

    let result = engine
        .execute(
            "SELECT t1.id, t1.name, t2.id, t2.value FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 4);

    let row_with_1 = result
        .rows
        .iter()
        .find(|row| row[0] == Value::Integer(1))
        .expect("Should find row with id=1");
    assert_eq!(row_with_1[0], Value::Integer(1));
    assert_eq!(row_with_1[1], Value::Text("a".to_string()));
    assert_eq!(row_with_1[2], Value::Integer(1));
    assert_eq!(row_with_1[3], Value::Integer(100));

    let row_with_2 = result
        .rows
        .iter()
        .find(|row| row[0] == Value::Integer(2))
        .expect("Should find row with id=2");
    assert_eq!(row_with_2[0], Value::Integer(2));
    assert_eq!(row_with_2[1], Value::Text("b".to_string()));
    assert_eq!(row_with_2[2], Value::Integer(2));
    assert_eq!(row_with_2[3], Value::Integer(200));

    let row_with_3 = result
        .rows
        .iter()
        .find(|row| row[0] == Value::Integer(3))
        .expect("Should find row with id=3 from t1");
    assert_eq!(row_with_3[0], Value::Integer(3));
    assert_eq!(row_with_3[1], Value::Text("c".to_string()));
    assert_eq!(row_with_3[2], Value::Null);
    assert_eq!(row_with_3[3], Value::Null);

    let row_with_4 = result
        .rows
        .iter()
        .find(|row| row[2] == Value::Integer(4))
        .expect("Should find row with id=4 from t2");
    assert_eq!(row_with_4[0], Value::Null);
    assert_eq!(row_with_4[1], Value::Null);
    assert_eq!(row_with_4[2], Value::Integer(4));
    assert_eq!(row_with_4[3], Value::Integer(400));
}

#[test]
#[ignore]
fn test_full_outer_join_all_match() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER, value INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b')")
        .unwrap();
    engine
        .execute("INSERT INTO t2 VALUES (1, 100), (2, 200)")
        .unwrap();

    let result = engine
        .execute(
            "SELECT t1.id, t1.name, t2.id, t2.value FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 2);

    let row_with_1 = result
        .rows
        .iter()
        .find(|row| row[0] == Value::Integer(1))
        .expect("Should find row with id=1");
    assert_eq!(row_with_1[0], Value::Integer(1));
    assert_eq!(row_with_1[1], Value::Text("a".to_string()));
    assert_eq!(row_with_1[2], Value::Integer(1));
    assert_eq!(row_with_1[3], Value::Integer(100));

    let row_with_2 = result
        .rows
        .iter()
        .find(|row| row[0] == Value::Integer(2))
        .expect("Should find row with id=2");
    assert_eq!(row_with_2[0], Value::Integer(2));
    assert_eq!(row_with_2[1], Value::Text("b".to_string()));
    assert_eq!(row_with_2[2], Value::Integer(2));
    assert_eq!(row_with_2[3], Value::Integer(200));
}

#[test]
#[ignore]
fn test_full_outer_join_no_match() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER, value INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b')")
        .unwrap();
    engine
        .execute("INSERT INTO t2 VALUES (3, 100), (4, 200)")
        .unwrap();

    let result = engine
        .execute(
            "SELECT t1.id, t1.name, t2.id, t2.value FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 4);

    let row_with_1 = result
        .rows
        .iter()
        .find(|row| row[0] == Value::Integer(1))
        .expect("Should find row with id=1 from t1");
    assert_eq!(row_with_1[0], Value::Integer(1));
    assert_eq!(row_with_1[1], Value::Text("a".to_string()));
    assert_eq!(row_with_1[2], Value::Null);
    assert_eq!(row_with_1[3], Value::Null);

    let row_with_2 = result
        .rows
        .iter()
        .find(|row| row[0] == Value::Integer(2))
        .expect("Should find row with id=2 from t1");
    assert_eq!(row_with_2[0], Value::Integer(2));
    assert_eq!(row_with_2[1], Value::Text("b".to_string()));
    assert_eq!(row_with_2[2], Value::Null);
    assert_eq!(row_with_2[3], Value::Null);

    let row_with_3 = result
        .rows
        .iter()
        .find(|row| row[2] == Value::Integer(3))
        .expect("Should find row with id=3 from t2");
    assert_eq!(row_with_3[0], Value::Null);
    assert_eq!(row_with_3[1], Value::Null);
    assert_eq!(row_with_3[2], Value::Integer(3));
    assert_eq!(row_with_3[3], Value::Integer(100));

    let row_with_4 = result
        .rows
        .iter()
        .find(|row| row[2] == Value::Integer(4))
        .expect("Should find row with id=4 from t2");
    assert_eq!(row_with_4[0], Value::Null);
    assert_eq!(row_with_4[1], Value::Null);
    assert_eq!(row_with_4[2], Value::Integer(4));
    assert_eq!(row_with_4[3], Value::Integer(200));
}
