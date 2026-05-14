//! Window Function Execution Tests
//! GAP-2: coverage improvement for window functions

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

fn setup_sales_data(engine: &mut ExecutionEngine<MemoryStorage>) {
    engine.execute("CREATE TABLE sales (id INTEGER PRIMARY KEY, region TEXT, quarter TEXT, amount INTEGER)").unwrap();
    for sql in &[
        "INSERT INTO sales VALUES (1, 'North', 'Q1', 100)",
        "INSERT INTO sales VALUES (2, 'North', 'Q2', 150)",
        "INSERT INTO sales VALUES (3, 'North', 'Q3', 120)",
        "INSERT INTO sales VALUES (4, 'North', 'Q4', 180)",
        "INSERT INTO sales VALUES (5, 'South', 'Q1', 200)",
        "INSERT INTO sales VALUES (6, 'South', 'Q2', 160)",
        "INSERT INTO sales VALUES (7, 'South', 'Q3', 220)",
        "INSERT INTO sales VALUES (8, 'South', 'Q4', 190)",
    ] {
        engine.execute(sql).unwrap();
    }
}

#[test]
fn test_window_lead() {
    let mut engine = create_engine();
    setup_sales_data(&mut engine);
    let result = engine
        .execute("SELECT LEAD(amount) OVER (ORDER BY quarter) FROM sales WHERE region = 'North'")
        .unwrap();
    assert!(!result.rows.is_empty());
}

#[test]
fn test_window_lag() {
    let mut engine = create_engine();
    setup_sales_data(&mut engine);
    let result = engine
        .execute("SELECT LAG(amount) OVER (ORDER BY quarter) FROM sales WHERE region = 'North'")
        .unwrap();
    assert!(!result.rows.is_empty());
}

#[test]
fn test_window_first_value() {
    let mut engine = create_engine();
    setup_sales_data(&mut engine);
    let result = engine
        .execute(
            "SELECT FIRST_VALUE(amount) OVER (ORDER BY quarter) FROM sales WHERE region = 'North'",
        )
        .unwrap();
    assert!(!result.rows.is_empty());
}

#[test]
fn test_window_last_value() {
    let mut engine = create_engine();
    setup_sales_data(&mut engine);
    let result = engine
        .execute(
            "SELECT LAST_VALUE(amount) OVER (ORDER BY quarter) FROM sales WHERE region = 'North'",
        )
        .unwrap();
    assert!(!result.rows.is_empty());
}

#[test]
fn test_window_ntile() {
    let mut engine = create_engine();
    setup_sales_data(&mut engine);
    let result = engine
        .execute("SELECT NTILE(2) OVER (ORDER BY amount) FROM sales WHERE region = 'North'")
        .unwrap();
    assert!(!result.rows.is_empty());
}

#[test]
fn test_window_nth_value() {
    let mut engine = create_engine();
    setup_sales_data(&mut engine);
    let result = engine
        .execute(
            "SELECT NTH_VALUE(amount, 2) OVER (ORDER BY quarter) FROM sales WHERE region = 'North'",
        )
        .unwrap();
    assert!(!result.rows.is_empty());
}

#[test]
fn test_window_partition_by() {
    let mut engine = create_engine();
    setup_sales_data(&mut engine);
    let result = engine
        .execute("SELECT SUM(amount) OVER (PARTITION BY region) FROM sales")
        .unwrap();
    assert!(!result.rows.is_empty());
}

#[test]
fn test_window_order_by() {
    let mut engine = create_engine();
    setup_sales_data(&mut engine);
    let result = engine
        .execute("SELECT ROW_NUMBER() OVER (ORDER BY amount DESC) FROM sales")
        .unwrap();
    assert!(!result.rows.is_empty());
}

#[test]
fn test_multiple_window_functions() {
    let mut engine = create_engine();
    setup_sales_data(&mut engine);
    let result = engine.execute("SELECT ROW_NUMBER() OVER (PARTITION BY region ORDER BY amount), SUM(amount) OVER (PARTITION BY region) FROM sales").unwrap();
    assert!(!result.rows.is_empty());
}
