use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_window_row_number() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE sales (id INTEGER, product TEXT, amount INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES (1, 'A', 100)")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES (2, 'A', 200)")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES (3, 'B', 150)")
        .unwrap();

    let result =
        engine.execute("SELECT ROW_NUMBER() OVER (ORDER BY id) as rn, product, amount FROM sales");
    assert!(
        result.is_ok(),
        "ROW_NUMBER window function should work: {:?}",
        result
    );
}

#[test]
fn test_window_rank() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE scores (id INTEGER, student TEXT, score INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO scores VALUES (1, 'alice', 95)")
        .unwrap();
    engine
        .execute("INSERT INTO scores VALUES (2, 'bob', 95)")
        .unwrap();
    engine
        .execute("INSERT INTO scores VALUES (3, 'carol', 90)")
        .unwrap();

    let result = engine
        .execute("SELECT RANK() OVER (ORDER BY score DESC) as rank, student, score FROM scores");
    assert!(
        result.is_ok(),
        "RANK window function should work: {:?}",
        result
    );
}

#[test]
fn test_window_dense_rank() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE rankings (id INTEGER, name TEXT, points INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO rankings VALUES (1, 'alice', 100)")
        .unwrap();
    engine
        .execute("INSERT INTO rankings VALUES (2, 'bob', 100)")
        .unwrap();
    engine
        .execute("INSERT INTO rankings VALUES (3, 'carol', 90)")
        .unwrap();

    let result = engine.execute(
        "SELECT DENSE_RANK() OVER (ORDER BY points DESC) as dense_rank, name FROM rankings",
    );
    assert!(
        result.is_ok(),
        "DENSE_RANK window function should work: {:?}",
        result
    );
}
