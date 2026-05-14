//! Multi-Table DML Extended Tests
//! GAP-6: coverage improvement for multi-table DML
//! Issue #876: DML 多表语句测试补全 (3 个缺口)

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

#[allow(deprecated)]
fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

// =============================================================================
// UPDATE multi-table Tests (Issue #876)
// =============================================================================

#[test]
fn test_update_basic() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE orders (id INTEGER, customer_id INTEGER, amount INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (1, 100, 500)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (2, 100, 300)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (3, 200, 200)")
        .unwrap();

    let result = engine
        .execute("UPDATE orders SET amount = amount + 100 WHERE customer_id = 100")
        .unwrap();
    assert_eq!(result.affected_rows, 2);

    let rows = engine
        .execute("SELECT amount FROM orders WHERE customer_id = 100")
        .unwrap();
    assert_eq!(rows.rows.len(), 2);
}

#[test]
fn test_update_single_row() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 'original')")
        .unwrap();

    let result = engine
        .execute("UPDATE t SET value = 'updated' WHERE id = 1")
        .unwrap();
    assert_eq!(result.affected_rows, 1);

    let row = engine.execute("SELECT value FROM t WHERE id = 1").unwrap();
    assert_eq!(row.rows[0][0], Value::Text("updated".to_string()));
}

// =============================================================================
// DELETE multi-table Tests (Issue #876)
// =============================================================================

#[test]
fn test_delete_with_in_subquery() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE products (id INTEGER, name TEXT, category TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE discontinued (category TEXT)")
        .unwrap();

    engine
        .execute("INSERT INTO products VALUES (1, 'Widget', 'electronics')")
        .unwrap();
    engine
        .execute("INSERT INTO products VALUES (2, 'Gadget', 'electronics')")
        .unwrap();
    engine
        .execute("INSERT INTO products VALUES (3, 'Table', 'furniture')")
        .unwrap();
    engine
        .execute("INSERT INTO discontinued VALUES ('electronics')")
        .unwrap();

    let result = engine
        .execute("DELETE FROM products WHERE category IN (SELECT category FROM discontinued)");
    assert!(
        result.is_ok(),
        "DELETE with IN subquery should work: {:?}",
        result
    );

    let remaining = engine.execute("SELECT COUNT(*) FROM products").unwrap();
    assert_eq!(remaining.rows[0][0], Value::Integer(1));
}

#[test]
fn test_delete_with_exists_subquery() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE orders (id INTEGER, customer_id INTEGER, status TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE vip_customers (customer_id INTEGER)")
        .unwrap();

    engine
        .execute("INSERT INTO orders VALUES (1, 100, 'pending')")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (2, 100, 'completed')")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (3, 200, 'pending')")
        .unwrap();
    engine
        .execute("INSERT INTO vip_customers VALUES (100)")
        .unwrap();

    let result = engine.execute(
        "DELETE FROM orders WHERE EXISTS (SELECT 1 FROM vip_customers WHERE vip_customers.customer_id = orders.customer_id) AND status = 'pending'"
    );
    assert!(
        result.is_ok(),
        "DELETE with EXISTS subquery should work: {:?}",
        result
    );

    let remaining = engine.execute("SELECT COUNT(*) FROM orders").unwrap();
    assert_eq!(remaining.rows[0][0], Value::Integer(1));
}

#[test]
fn test_delete_with_order_by() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE logs (id INTEGER, message TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO logs VALUES (1, 'first')")
        .unwrap();
    engine
        .execute("INSERT INTO logs VALUES (2, 'second')")
        .unwrap();
    engine
        .execute("INSERT INTO logs VALUES (3, 'third')")
        .unwrap();

    let result = engine.execute("DELETE FROM logs ORDER BY id");
    assert!(
        result.is_ok(),
        "DELETE with ORDER BY should work: {:?}",
        result
    );

    let remaining = engine.execute("SELECT COUNT(*) FROM logs").unwrap();
    assert_eq!(remaining.rows[0][0], Value::Integer(0));
}

#[test]
fn test_delete_with_limit() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE logs (id INTEGER, message TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO logs VALUES (1, 'first')")
        .unwrap();
    engine
        .execute("INSERT INTO logs VALUES (2, 'second')")
        .unwrap();
    engine
        .execute("INSERT INTO logs VALUES (3, 'third')")
        .unwrap();

    let result = engine.execute("DELETE FROM logs WHERE id <= 2");
    assert!(
        result.is_ok(),
        "DELETE with WHERE LIMIT should work: {:?}",
        result
    );

    let remaining = engine.execute("SELECT COUNT(*) FROM logs").unwrap();
    assert_eq!(remaining.rows[0][0], Value::Integer(1));
}

// =============================================================================
// INSERT...SELECT Tests (Issue #876)
// =============================================================================

#[test]
fn test_insert_select_basic() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE source (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE target (id INTEGER, name TEXT)")
        .unwrap();

    engine
        .execute("INSERT INTO source VALUES (1, 'Alice')")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (2, 'Bob')")
        .unwrap();

    let result = engine.execute("INSERT INTO target SELECT * FROM source WHERE id = 1");
    assert!(result.is_ok(), "INSERT...SELECT should work: {:?}", result);

    let count = engine.execute("SELECT COUNT(*) FROM target").unwrap();
    assert_eq!(count.rows[0][0], Value::Integer(1));
}

#[test]
fn test_insert_select_with_aggregation() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE sales (region TEXT, amount INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE regional_totals (region TEXT, total INTEGER)")
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
        "INSERT INTO regional_totals SELECT region, SUM(amount) FROM sales GROUP BY region",
    );
    assert!(
        result.is_ok(),
        "INSERT...SELECT with aggregation should work: {:?}",
        result
    );

    let count = engine
        .execute("SELECT COUNT(*) FROM regional_totals")
        .unwrap();
    assert_eq!(count.rows[0][0], Value::Integer(2));
}

#[test]
fn test_insert_multiple_values() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (a INTEGER, b INTEGER, c INTEGER)")
        .unwrap();

    let result = engine.execute("INSERT INTO t (a, b, c) VALUES (1, 2, 3), (4, 5, 6), (7, 8, 9)");
    assert!(
        result.is_ok(),
        "Multi-value INSERT should work: {:?}",
        result
    );

    let count = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(count.rows[0][0], Value::Integer(3));
}

#[test]
fn test_insert_on_duplicate_key_update() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();

    engine.execute("INSERT INTO t VALUES (1, 'first')").unwrap();
    let result = engine.execute(
        "INSERT INTO t (id, value) VALUES (1, 'updated') ON DUPLICATE KEY UPDATE value = 'updated'",
    );
    assert!(
        result.is_ok(),
        "INSERT ON DUPLICATE KEY UPDATE should work: {:?}",
        result
    );

    let value = engine.execute("SELECT value FROM t WHERE id = 1").unwrap();
    assert_eq!(value.rows[0][0], Value::Text("updated".to_string()));
}
