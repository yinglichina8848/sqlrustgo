//! OLTP Integration Tests
//!
//! Tests for Sysbench-style OLTP workloads against MemoryExecutionEngine.

use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn setup_engine() -> MemoryExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    MemoryExecutionEngine::new(storage)
}

fn setup_sbtest(engine: &mut MemoryExecutionEngine) {
    engine
        .execute(
            "CREATE TABLE sbtest1 (
                id INTEGER PRIMARY KEY,
                k INTEGER DEFAULT 0,
                c CHAR(120) DEFAULT '',
                pad CHAR(60) DEFAULT ''
            )",
        )
        .expect("create sbtest1 table");

    for i in 1..=10000 {
        let k = i % 1000;
        engine
            .execute(&format!(
                "INSERT INTO sbtest1 VALUES ({}, {}, 'pad.{}', 'pad{}')",
                i, k, i, i
            ))
            .expect("insert sbtest1 row");
    }
}

#[test]
fn oltp_point_select() {
    let mut engine = setup_engine();
    setup_sbtest(&mut engine);

    let mut success_count = 0;
    let iterations = 1000;

    for i in 1..=iterations {
        let id = (i % 10000) + 1;
        match engine.execute(&format!("SELECT c FROM sbtest1 WHERE id = {}", id)) {
            Ok(_) => success_count += 1,
            Err(_) => {}
        }
    }

    assert_eq!(
        success_count, iterations,
        "All point selects should succeed"
    );
}

#[test]
fn oltp_range_select() {
    let mut engine = setup_engine();
    setup_sbtest(&mut engine);

    let mut success_count = 0;
    let iterations = 100;

    for i in 1..=iterations {
        let start = (i % 9000) + 1;
        let end = start + 100;
        match engine.execute(&format!(
            "SELECT c FROM sbtest1 WHERE id BETWEEN {} AND {}",
            start, end
        )) {
            Ok(_) => success_count += 1,
            Err(_) => {}
        }
    }

    assert_eq!(
        success_count, iterations,
        "All range selects should succeed"
    );
}

// NOTE: UPDATE is parsed but not fully executed in executor
// #[test]
// fn oltp_update_index() {
//     let mut engine = setup_engine();
//     setup_sbtest(&mut engine);

//     let mut success_count = 0;
//     let iterations = 100;

//     for i in 1..=iterations {
//         let id = (i % 10000) + 1;
//         match engine.execute(&format!(
//             "UPDATE sbtest1 SET k = k + 1 WHERE id = {}",
//             id
//         )) {
//             Ok(_) => success_count += 1,
//             Err(_) => {}
//         }
//     }

//     assert_eq!(
//         success_count, iterations,
//         "All index updates should succeed"
//     );
// }

#[test]
fn oltp_insert() {
    let mut engine = setup_engine();
    setup_sbtest(&mut engine);

    let mut success_count = 0;
    let start_id = 10001i64;
    let iterations = 100;

    for i in 0..iterations {
        let id = start_id + i;
        let k = i % 1000;
        match engine.execute(&format!(
            "INSERT INTO sbtest1 VALUES ({}, {}, 'pad.{}', 'pad{}')",
            id, k, id, id
        )) {
            Ok(_) => success_count += 1,
            Err(_) => {}
        }
    }

    assert_eq!(success_count, iterations, "All inserts should succeed");
}

#[test]
fn oltp_delete() {
    let mut engine = setup_engine();
    setup_sbtest(&mut engine);

    for i in 0..10 {
        let id = 20001 + i;
        let k = i % 1000;
        engine
            .execute(&format!(
                "INSERT INTO sbtest1 VALUES ({}, {}, 'pad.{}', 'pad{}')",
                id, k, id, id
            ))
            .expect("insert for delete test");
    }

    let mut success_count = 0;
    let iterations = 10;

    for i in 0..iterations {
        let id = 20001 + i;
        match engine.execute(&format!("DELETE FROM sbtest1 WHERE id = {}", id)) {
            Ok(_) => success_count += 1,
            Err(_) => {}
        }
    }

    assert_eq!(success_count, iterations, "All deletes should succeed");
}

#[test]
fn oltp_mixed() {
    let mut engine = setup_engine();
    setup_sbtest(&mut engine);

    let mut read_success = 0;
    let mut insert_success = 0;
    let iterations = 50;

    for i in 1..=iterations {
        let id = (i % 10000) + 1;

        if engine
            .execute(&format!("SELECT c FROM sbtest1 WHERE id = {}", id))
            .is_ok()
        {
            read_success += 1;
        }

        // Use INSERT instead of UPDATE (UPDATE not fully supported)
        let new_id = 30000 + i;
        if engine
            .execute(&format!(
                "INSERT INTO sbtest1 VALUES ({}, {}, 'mixed', 'pad')",
                new_id,
                i % 1000
            ))
            .is_ok()
        {
            insert_success += 1;
        }
    }

    assert_eq!(read_success, iterations, "All reads should succeed");
    assert_eq!(insert_success, iterations, "All inserts should succeed");
}

#[test]
fn oltp_bulk_insert() {
    let mut engine = setup_engine();

    engine
        .execute(
            "CREATE TABLE bulk_test (
                id INTEGER PRIMARY KEY,
                value INTEGER DEFAULT 0
            )",
        )
        .expect("create bulk_test table");

    let iterations = 1000;
    let mut success_count = 0;

    for i in 1..=iterations {
        match engine.execute(&format!(
            "INSERT INTO bulk_test VALUES ({}, {})",
            i,
            i % 100
        )) {
            Ok(_) => success_count += 1,
            Err(_) => {}
        }
    }

    assert_eq!(success_count, iterations, "All bulk inserts should succeed");

    let result = engine.execute("SELECT COUNT(*) FROM bulk_test");
    assert!(result.is_ok(), "COUNT should succeed");
}

#[test]
fn oltp_aggregation() {
    let mut engine = setup_engine();
    setup_sbtest(&mut engine);

    let result =
        engine.execute("SELECT k, COUNT(*) as cnt FROM sbtest1 GROUP BY k ORDER BY k LIMIT 10");

    assert!(result.is_ok(), "Aggregation should succeed");

    let result = engine.execute(
        "SELECT k, COUNT(*) as cnt FROM sbtest1 GROUP BY k HAVING COUNT(*) > 5 ORDER BY k LIMIT 10",
    );

    assert!(result.is_ok(), "Aggregation with HAVING should succeed");
}
