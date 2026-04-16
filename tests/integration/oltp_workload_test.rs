// OLTP Workload Integration Tests
// Issue #1424: Sysbench OLTP workloads integration

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::sync::{Arc, RwLock};

const SBTEST_CREATE: &str = "CREATE TABLE sbtest (
    id INTEGER PRIMARY KEY,
    k INTEGER,
    c TEXT,
    pad TEXT
)";

fn setup_engine() -> ExecutionEngine {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    engine.execute(parse(SBTEST_CREATE).unwrap()).unwrap();

    let mut storage = engine.storage.write().unwrap();
    for i in 1..=100 {
        storage
            .insert(
                "sbtest",
                vec![vec![
                    sqlrustgo_types::Value::Integer(i as i64),
                    sqlrustgo_types::Value::Integer((i * 10) as i64),
                    sqlrustgo_types::Value::Text(format!("c_{}", i)),
                    sqlrustgo_types::Value::Text(format!("pad_{}", i)),
                ]],
            )
            .unwrap();
    }
    drop(storage);

    engine
}

#[test]
fn test_oltp_index_scan() {
    let mut engine = setup_engine();
    let mut rng = SmallRng::seed_from_u64(42);

    for _ in 0..10 {
        let start = rng.gen_range(1..50u64);
        let end = start.saturating_add(100);
        let sql = format!(
            "SELECT id, k FROM sbtest WHERE id BETWEEN {} AND {}",
            start, end
        );

        let result = engine.execute(parse(&sql).unwrap()).unwrap();
        assert!(result.rows.is_empty() || result.rows.len() <= 100);
    }
    println!("✓ OLTP index scan workload");
}

#[test]
fn test_oltp_range_scan() {
    let mut engine = setup_engine();
    let mut rng = SmallRng::seed_from_u64(42);

    for _ in 0..10 {
        let id1 = rng.gen_range(1..100u64);
        let id2 = rng.gen_range(1..100u64);
        let start = id1.min(id2);
        let end = id1.max(id2);
        let sql = format!(
            "SELECT * FROM sbtest WHERE id BETWEEN {} AND {} ORDER BY id",
            start, end
        );

        let result = engine.execute(parse(&sql).unwrap()).unwrap();
        assert!(result.rows.len() <= 100);
    }
    println!("✓ OLTP range scan workload");
}

#[test]
fn test_oltp_point_select() {
    let mut engine = setup_engine();
    let mut rng = SmallRng::seed_from_u64(42);

    for _ in 0..10 {
        let id = rng.gen_range(1..100u64);
        let sql = format!("SELECT id, k, c FROM sbtest WHERE id = {}", id);

        let result = engine.execute(parse(&sql).unwrap()).unwrap();
        assert!(result.rows.len() <= 1);
    }
    println!("✓ OLTP point select workload");
}

#[test]
fn test_oltp_insert() {
    let mut engine = setup_engine();
    let mut rng = SmallRng::seed_from_u64(42);

    for i in 0..10 {
        let id = 1000 + i as i64;
        let k = rng.gen_range(1..10000i64);
        let sql = format!(
            "INSERT INTO sbtest (id, k, c, pad) VALUES ({}, {}, 'c_{}', 'pad_{}')",
            id, k, id, id
        );

        let result = engine.execute(parse(&sql).unwrap()).unwrap();
        assert_eq!(result.affected_rows, 1);
    }

    let count_result = engine
        .execute(parse("SELECT COUNT(*) FROM sbtest").unwrap())
        .unwrap();
    let count: i64 = match &count_result.rows[0][0] {
        sqlrustgo_types::Value::Integer(n) => *n,
        _ => panic!("Expected integer"),
    };
    assert_eq!(count, 110);
    println!("✓ OLTP insert workload");
}

#[test]
fn test_oltp_update_index() {
    let mut engine = setup_engine();
    let mut rng = SmallRng::seed_from_u64(42);

    for _ in 0..10 {
        let id = rng.gen_range(1..100u64);
        let k = rng.gen_range(1..10000i64);
        let sql = format!("UPDATE sbtest SET k = {} WHERE id = {}", k, id);

        let result = engine.execute(parse(&sql).unwrap()).unwrap();
        assert_eq!(result.affected_rows, 1);
    }
    println!("✓ OLTP update index workload");
}

#[test]
fn test_oltp_update_non_index() {
    let mut engine = setup_engine();
    let mut rng = SmallRng::seed_from_u64(42);

    for _ in 0..10 {
        let id = rng.gen_range(1..100u64);
        let sql = format!(
            "UPDATE sbtest SET c = 'updated_{}', pad = 'pad_updated_{}' WHERE id = {}",
            id, id, id
        );

        let result = engine.execute(parse(&sql).unwrap()).unwrap();
        assert_eq!(result.affected_rows, 1);
    }
    println!("✓ OLTP update non-index workload");
}

#[test]
fn test_oltp_delete() {
    let mut engine = setup_engine();
    let mut rng = SmallRng::seed_from_u64(42);

    let ids_to_delete: Vec<i64> = (0..10).map(|_| rng.gen_range(1..101i64)).collect();

    for id in ids_to_delete {
        let sql = format!("DELETE FROM sbtest WHERE id = {}", id);
        let result = engine.execute(parse(&sql).unwrap()).unwrap();
        assert_eq!(result.affected_rows, 1);
    }
    println!("✓ OLTP delete workload");
}

#[test]
fn test_oltp_mixed() {
    let mut engine = setup_engine();
    let mut rng = SmallRng::seed_from_u64(42);

    for _ in 0..5 {
        let id = rng.gen_range(1..100u64);

        let select_sql = format!("SELECT * FROM sbtest WHERE id = {}", id);
        let _ = engine.execute(parse(&select_sql).unwrap()).unwrap();

        let update_sql = format!("UPDATE sbtest SET k = k + 1 WHERE id = {}", id);
        let _ = engine.execute(parse(&update_sql).unwrap()).unwrap();

        let insert_id = 2000 + rng.gen_range(1..100i64);
        let insert_sql = format!(
            "INSERT INTO sbtest (id, k, c, pad) VALUES ({}, 1, 'mixed', 'mixed')",
            insert_id
        );
        let _ = engine.execute(parse(&insert_sql).unwrap()).unwrap();
    }
    println!("✓ OLTP mixed workload");
}

#[test]
fn test_oltp_write_only() {
    let mut engine = setup_engine();
    let mut rng = SmallRng::seed_from_u64(42);

    for i in 0..10 {
        let insert_id = 3000 + i as i64;
        let sql = format!(
            "INSERT INTO sbtest (id, k, c, pad) VALUES ({}, {}, 'write', 'only')",
            insert_id,
            rng.gen_range(1..10000i64)
        );

        let result = engine.execute(parse(&sql).unwrap()).unwrap();
        assert_eq!(result.affected_rows, 1);
    }
    println!("✓ OLTP write-only workload");
}

#[test]
fn test_oltp_read_only() {
    let mut engine = setup_engine();

    for _ in 0..10 {
        let sql = "SELECT COUNT(*) FROM sbtest".to_string();
        let result = engine.execute(parse(&sql).unwrap()).unwrap();
        assert_eq!(result.rows.len(), 1);
    }

    for _ in 0..10 {
        let sql = "SELECT SUM(k) FROM sbtest WHERE id < 50".to_string();
        let result = engine.execute(parse(&sql).unwrap()).unwrap();
        assert_eq!(result.rows.len(), 1);
    }
    println!("✓ OLTP read-only workload");
}

#[test]
fn test_oltp_read_write() {
    let mut engine = setup_engine();

    for i in 0..10 {
        let select_sql = "SELECT * FROM sbtest WHERE id < 10".to_string();
        let _ = engine.execute(parse(&select_sql).unwrap()).unwrap();

        let update_sql = format!("UPDATE sbtest SET k = {} WHERE id = {}", i * 100, i + 1);
        let _ = engine.execute(parse(&update_sql).unwrap()).unwrap();
    }
    println!("✓ OLTP read-write workload");
}

#[test]
fn test_oltp_all_workloads_execute() {
    let mut engine = setup_engine();
    let mut rng = SmallRng::seed_from_u64(123);

    for _ in 0..20 {
        let start = rng.gen_range(1..50u64);
        let sql = format!(
            "SELECT id FROM sbtest WHERE id BETWEEN {} AND {}",
            start,
            start + 50
        );
        engine.execute(parse(&sql).unwrap()).unwrap();

        let id = rng.gen_range(1..100u64);
        let sql = format!("SELECT * FROM sbtest WHERE id = {}", id);
        engine.execute(parse(&sql).unwrap()).unwrap();

        let insert_id = 5000 + rng.gen_range(1..1000i64);
        let sql = format!(
            "INSERT INTO sbtest (id, k, c, pad) VALUES ({}, 1, 'x', 'x')",
            insert_id
        );
        engine.execute(parse(&sql).unwrap()).unwrap();

        let id = rng.gen_range(1..100u64);
        let sql = format!("UPDATE sbtest SET k = k + 1 WHERE id = {}", id);
        engine.execute(parse(&sql).unwrap()).unwrap();

        let id = 6000 + rng.gen_range(1..1000i64);
        let sql = format!("DELETE FROM sbtest WHERE id = {}", id);
        engine.execute(parse(&sql).unwrap()).unwrap();
    }

    println!("✓ All OLTP workloads execute successfully");
}
