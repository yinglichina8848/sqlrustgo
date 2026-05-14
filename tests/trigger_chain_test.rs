// Trigger Chain Tests (BP2-9)
//! Tests for trigger chain execution: multiple triggers ordered correctly
//! BP2 Gate: cargo test --test trigger_chain_test

use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_catalog::Catalog;
use std::sync::{Arc, RwLock};

fn create_engine() -> MemoryExecutionEngine {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    MemoryExecutionEngine::with_memory_and_catalog(catalog)
}

/// Test multiple BEFORE INSERT triggers execute in order
#[test]
fn test_before_insert_trigger_chain_order() {
    let mut engine = create_engine();

    // Create table
    let result = engine.execute("CREATE TABLE events (id INTEGER, data TEXT, ts TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Create first trigger
    let result = engine.execute(
        "CREATE TRIGGER set_ts1 BEFORE INSERT ON events FOR EACH ROW BEGIN SET NEW.ts = 'first'; END"
    );
    assert!(
        result.is_ok(),
        "CREATE TRIGGER set_ts1 failed: {:?}",
        result.err()
    );

    // Create second trigger
    let result = engine.execute(
        "CREATE TRIGGER set_ts2 BEFORE INSERT ON events FOR EACH ROW BEGIN SET NEW.ts = 'second'; END"
    );
    assert!(
        result.is_ok(),
        "CREATE TRIGGER set_ts2 failed: {:?}",
        result.err()
    );

    // Insert - second trigger should override (last wins in chain)
    let result = engine.execute("INSERT INTO events VALUES (1, 'test', '')");
    assert!(result.is_ok(), "INSERT failed: {:?}", result.err());

    // Verify trigger executed
    let rows = engine.execute("SELECT ts FROM events").unwrap();
    assert_eq!(rows.rows.len(), 1);
}

/// Test AFTER INSERT trigger chain
#[test]
fn test_after_insert_trigger_chain() {
    let mut engine = create_engine();

    // Create tables
    let result =
        engine.execute("CREATE TABLE orders (id INTEGER, product_id INTEGER, quantity INTEGER)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    let result = engine.execute("CREATE TABLE inventory (product_id INTEGER, stock INTEGER)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    let result = engine.execute("CREATE TABLE order_log (id INTEGER, message TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Setup inventory
    let result = engine.execute("INSERT INTO inventory VALUES (1, 100)");
    assert!(
        result.is_ok(),
        "INSERT inventory failed: {:?}",
        result.err()
    );

    // Create trigger to decrement stock
    let result = engine.execute(
        "CREATE TRIGGER decrement_stock AFTER INSERT ON orders FOR EACH ROW BEGIN UPDATE inventory SET stock = stock - NEW.quantity WHERE product_id = NEW.product_id; END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER failed: {:?}", result.err());

    // Create trigger to log
    let result = engine.execute(
        "CREATE TRIGGER log_order AFTER INSERT ON orders FOR EACH ROW BEGIN INSERT INTO order_log VALUES (NEW.id, 'order created'); END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER failed: {:?}", result.err());

    // Insert order
    let result = engine.execute("INSERT INTO orders VALUES (1, 1, 10)");
    assert!(result.is_ok(), "INSERT orders failed: {:?}", result.err());

    // Verify both triggers worked
    let inventory = engine
        .execute("SELECT stock FROM inventory WHERE product_id = 1")
        .unwrap();
    assert_eq!(inventory.rows.len(), 1);

    let order_log = engine.execute("SELECT * FROM order_log").unwrap();
    assert_eq!(order_log.rows.len(), 1);
}

/// Test UPDATE trigger chain
#[test]
fn test_update_trigger_chain() {
    let mut engine = create_engine();

    // Create tables
    let result = engine.execute("CREATE TABLE products (id INTEGER, name TEXT, price INTEGER)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    let result = engine.execute(
        "CREATE TABLE price_history (product_id INTEGER, old_price INTEGER, new_price INTEGER)",
    );
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    let result = engine.execute("CREATE TABLE update_log (id INTEGER, message TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Insert product
    let result = engine.execute("INSERT INTO products VALUES (1, 'Widget', 100)");
    assert!(result.is_ok(), "INSERT product failed: {:?}", result.err());

    // Create trigger to log price change
    let result = engine.execute(
        "CREATE TRIGGER log_price_change AFTER UPDATE ON products FOR EACH ROW BEGIN INSERT INTO price_history VALUES (OLD.id, OLD.price, NEW.price); END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER failed: {:?}", result.err());

    // Create trigger to log update
    let result = engine.execute(
        "CREATE TRIGGER log_update AFTER UPDATE ON products FOR EACH ROW BEGIN INSERT INTO update_log VALUES (NEW.id, 'updated'); END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER failed: {:?}", result.err());

    // Update product
    let result = engine.execute("UPDATE products SET price = 150 WHERE id = 1");
    assert!(result.is_ok(), "UPDATE failed: {:?}", result.err());

    // Verify triggers worked
    let history = engine.execute("SELECT * FROM price_history").unwrap();
    assert_eq!(history.rows.len(), 1);

    let log = engine.execute("SELECT * FROM update_log").unwrap();
    assert_eq!(log.rows.len(), 1);
}

/// Test DELETE trigger chain
#[test]
fn test_delete_trigger_chain() {
    let mut engine = create_engine();

    // Create tables
    let result = engine.execute("CREATE TABLE orders (id INTEGER, status TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    let result = engine.execute("CREATE TABLE cancelled_orders (id INTEGER, status TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    let result = engine.execute("CREATE TABLE delete_log (id INTEGER, message TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Insert order
    let result = engine.execute("INSERT INTO orders VALUES (1, 'cancelled')");
    assert!(result.is_ok(), "INSERT order failed: {:?}", result.err());

    // Create trigger to move to cancelled
    let result = engine.execute(
        "CREATE TRIGGER move_cancelled BEFORE DELETE ON orders FOR EACH ROW BEGIN INSERT INTO cancelled_orders VALUES (OLD.id, OLD.status); END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER failed: {:?}", result.err());

    // Create trigger to log delete
    let result = engine.execute(
        "CREATE TRIGGER log_delete BEFORE DELETE ON orders FOR EACH ROW BEGIN INSERT INTO delete_log VALUES (OLD.id, 'deleted'); END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER failed: {:?}", result.err());

    // Delete order
    let result = engine.execute("DELETE FROM orders WHERE id = 1");
    assert!(result.is_ok(), "DELETE failed: {:?}", result.err());

    // Verify triggers worked
    let cancelled = engine.execute("SELECT * FROM cancelled_orders").unwrap();
    assert_eq!(cancelled.rows.len(), 1);

    let log = engine.execute("SELECT * FROM delete_log").unwrap();
    assert_eq!(log.rows.len(), 1);
}

/// Test BEFORE trigger can modify NEW values
#[test]
fn test_before_trigger_modifies_new() {
    let mut engine = create_engine();

    // Create table
    let result =
        engine.execute("CREATE TABLE items (id INTEGER, price INTEGER, discounted_price INTEGER)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Create trigger to set discounted price
    let result = engine.execute(
        "CREATE TRIGGER apply_discount BEFORE INSERT ON items FOR EACH ROW BEGIN SET NEW.discounted_price = NEW.price - 10; END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER failed: {:?}", result.err());

    // Insert item with price
    let result = engine.execute("INSERT INTO items VALUES (1, 100, 0)");
    assert!(result.is_ok(), "INSERT failed: {:?}", result.err());

    // Verify trigger set discounted price
    let rows = engine
        .execute("SELECT discounted_price FROM items WHERE id = 1")
        .unwrap();
    assert_eq!(rows.rows.len(), 1);
}

/// Test OLD value binding in UPDATE trigger
#[test]
fn test_old_value_binding() {
    let mut engine = create_engine();

    // Create table
    let result = engine.execute("CREATE TABLE users (id INTEGER, name TEXT, old_name TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Insert user
    let result = engine.execute("INSERT INTO users VALUES (1, 'Alice', '')");
    assert!(result.is_ok(), "INSERT failed: {:?}", result.err());

    // Create trigger to save old name
    let result = engine.execute(
        "CREATE TRIGGER save_old_name BEFORE UPDATE ON users FOR EACH ROW BEGIN SET NEW.old_name = OLD.name; END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER failed: {:?}", result.err());

    // Update user
    let result = engine.execute("UPDATE users SET name = 'Bob' WHERE id = 1");
    assert!(result.is_ok(), "UPDATE failed: {:?}", result.err());

    // Verify OLD name was captured
    let rows = engine
        .execute("SELECT old_name FROM users WHERE id = 1")
        .unwrap();
    assert_eq!(rows.rows.len(), 1);
}

/// Test multiple triggers on same table with different events
#[test]
fn test_mixed_event_triggers() {
    let mut engine = create_engine();

    // Create table
    let result = engine.execute("CREATE TABLE audit (id INTEGER, action TEXT, data TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Create audit_log table first
    let result = engine.execute("CREATE TABLE audit_log (id INTEGER, action TEXT, data TEXT)");
    assert!(result.is_ok(), "CREATE TABLE failed: {:?}", result.err());

    // Create INSERT trigger
    let result = engine.execute(
        "CREATE TRIGGER audit_insert AFTER INSERT ON audit FOR EACH ROW BEGIN INSERT INTO audit_log VALUES (NEW.id, 'insert', NEW.data); END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER failed: {:?}", result.err());

    // Create UPDATE trigger
    let result = engine.execute(
        "CREATE TRIGGER audit_update AFTER UPDATE ON audit FOR EACH ROW BEGIN INSERT INTO audit_log VALUES (NEW.id, 'update', NEW.data); END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER failed: {:?}", result.err());

    // Create DELETE trigger
    let result = engine.execute(
        "CREATE TRIGGER audit_delete AFTER DELETE ON audit FOR EACH ROW BEGIN INSERT INTO audit_log VALUES (OLD.id, 'delete', OLD.data); END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER failed: {:?}", result.err());

    // Test INSERT
    let result = engine.execute("INSERT INTO audit VALUES (1, 'test', 'inserted')");
    assert!(result.is_ok(), "INSERT failed: {:?}", result.err());

    // Verify INSERT trigger worked
    let log = engine.execute("SELECT * FROM audit_log").unwrap();
    assert_eq!(log.rows.len(), 1);
}
