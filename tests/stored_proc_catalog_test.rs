// Stored Procedure Catalog Integration Tests
//
// These tests verify the stored procedure and trigger catalog integration
// that was added for Issue #1636 (存储过程与触发器 Catalog 集成)

use sqlrustgo::{ExecutionEngine, MemoryStorage};
use sqlrustgo_catalog::Catalog;
use std::sync::{Arc, RwLock};

#[test]
fn test_create_and_call_procedure_with_catalog() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob')")
        .unwrap();

    let create_result =
        engine.execute("CREATE PROCEDURE get_count() BEGIN SELECT COUNT(*) FROM users; END");
    assert!(
        create_result.is_ok(),
        "CREATE PROCEDURE should succeed with catalog"
    );

    let call_result = engine.execute("CALL get_count()");
    assert!(
        call_result.is_ok(),
        "CALL should succeed after procedure creation"
    );
}

#[test]
fn test_call_requires_catalog() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    let result = engine.execute("CALL my_proc(1, 2)");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("catalog"),
        "Error should mention catalog requirement"
    );
}

#[test]
fn test_create_procedure_requires_catalog() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    let result = engine.execute("CREATE PROCEDURE test_proc() BEGIN END");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("catalog"),
        "Error should mention catalog requirement"
    );
}

#[test]
fn test_procedure_not_found() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog);

    let result = engine.execute("CALL nonexistent_proc()");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not found"),
        "Error should indicate procedure not found"
    );
}

#[test]
fn test_duplicate_procedure_error() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    let create1 = engine.execute("CREATE PROCEDURE test_dup() BEGIN SELECT 1; END");
    assert!(create1.is_ok());

    let create2 = engine.execute("CREATE PROCEDURE test_dup() BEGIN SELECT 2; END");
    assert!(create2.is_err(), "Duplicate procedure should return error");
}

#[test]
fn test_create_trigger_with_catalog() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT, created_ts TEXT)")
        .unwrap();

    let create_trigger = engine.execute(
        "CREATE TRIGGER before_insert_ts BEFORE INSERT ON users FOR EACH ROW BEGIN SET NEW.created_ts = 'triggered'; END"
    );
    assert!(create_trigger.is_ok(), "CREATE TRIGGER should succeed");

    let insert_result = engine.execute("INSERT INTO users VALUES (1, 'Alice')");
    assert!(insert_result.is_ok());
}

#[test]
fn test_before_insert_trigger_sets_column() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE orders (id INTEGER, price FLOAT, quantity INTEGER, total FLOAT)")
        .unwrap();

    let create_trigger = engine.execute(
        "CREATE TRIGGER calc_total BEFORE INSERT ON orders FOR EACH ROW BEGIN SET NEW.total = 50.0; END"
    );
    assert!(create_trigger.is_ok(), "CREATE TRIGGER should succeed");

    let result = engine.execute("INSERT INTO orders VALUES (1, 10.0, 5, 0.0)");
    assert!(
        result.is_ok(),
        "INSERT should succeed even if trigger modifies value"
    );

    let rows = engine.execute("SELECT * FROM orders").unwrap();
    assert_eq!(rows.rows.len(), 1);
}

#[test]
fn test_after_insert_trigger_executes() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE source (id INTEGER, val TEXT)")
        .unwrap();

    engine
        .execute("CREATE TRIGGER log_insert AFTER INSERT ON source FOR EACH ROW BEGIN END")
        .unwrap();

    let result = engine.execute("INSERT INTO source VALUES (1, 'test')");
    assert!(
        result.is_ok(),
        "AFTER INSERT trigger should execute without error"
    );
}

#[test]
fn test_before_update_trigger_sets_column() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE products (id INTEGER, price FLOAT, discounted_price FLOAT)")
        .unwrap();

    engine
        .execute("INSERT INTO products VALUES (1, 100.0, 0.0)")
        .unwrap();

    let create_trigger = engine.execute(
        "CREATE TRIGGER apply_discount BEFORE UPDATE ON products FOR EACH ROW BEGIN SET NEW.discounted_price = 90.0; END"
    );
    assert!(create_trigger.is_ok(), "CREATE TRIGGER should succeed");

    let result = engine.execute("UPDATE products SET price = 100.0 WHERE id = 1");
    assert!(
        result.is_ok(),
        "UPDATE should succeed even if trigger modifies value"
    );
}

#[test]
fn test_after_update_trigger_executes() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE items (id INTEGER, status TEXT)")
        .unwrap();

    engine
        .execute("INSERT INTO items VALUES (1, 'pending')")
        .unwrap();

    engine
        .execute("CREATE TRIGGER log_status AFTER UPDATE ON items FOR EACH ROW BEGIN END")
        .unwrap();

    let result = engine.execute("UPDATE items SET status = 'approved' WHERE id = 1");
    assert!(
        result.is_ok(),
        "AFTER UPDATE trigger should execute without error"
    );
}

#[test]
fn test_before_delete_trigger_executes() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE items (id INTEGER, name TEXT)")
        .unwrap();

    engine
        .execute("INSERT INTO items VALUES (1, 'to_delete')")
        .unwrap();

    engine
        .execute("CREATE TRIGGER archive_delete BEFORE DELETE ON items FOR EACH ROW BEGIN END")
        .unwrap();

    let result = engine.execute("DELETE FROM items WHERE id = 1");
    assert!(
        result.is_ok(),
        "BEFORE DELETE trigger should execute without error"
    );

    let remaining = engine.execute("SELECT * FROM items").unwrap();
    assert_eq!(remaining.rows.len(), 0, "Row should be deleted");
}

#[test]
fn test_after_delete_trigger_executes() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE orders (id INTEGER, amount FLOAT)")
        .unwrap();

    engine
        .execute("INSERT INTO orders VALUES (1, 100.0)")
        .unwrap();

    engine
        .execute("CREATE TRIGGER track_deletion AFTER DELETE ON orders FOR EACH ROW BEGIN END")
        .unwrap();

    let result = engine.execute("DELETE FROM orders WHERE id = 1");
    assert!(
        result.is_ok(),
        "AFTER DELETE trigger should execute without error"
    );
}

#[test]
fn test_multiple_triggers_on_same_table() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE events (id INTEGER, data TEXT, ts TEXT)")
        .unwrap();

    engine
        .execute("CREATE TRIGGER set_ts1 BEFORE INSERT ON events FOR EACH ROW BEGIN SET NEW.ts = 'first'; END")
        .unwrap();

    engine
        .execute("CREATE TRIGGER set_ts2 BEFORE INSERT ON events FOR EACH ROW BEGIN SET NEW.ts = 'second'; END")
        .unwrap();

    let result = engine.execute("INSERT INTO events VALUES (1, 'test', '')");
    assert!(result.is_ok());

    let rows = engine.execute("SELECT ts FROM events").unwrap();
    assert_eq!(rows.rows.len(), 1);
}

#[test]
fn test_trigger_executes_insert() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE orders (id INTEGER, product_id INTEGER, quantity INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE inventory (product_id INTEGER, stock INTEGER)")
        .unwrap();

    engine
        .execute("INSERT INTO inventory VALUES (1, 100)")
        .unwrap();

    let result = engine.execute(
        "CREATE TRIGGER decrement_stock AFTER INSERT ON orders FOR EACH ROW BEGIN UPDATE inventory SET stock = stock - NEW.quantity WHERE product_id = NEW.product_id; END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER should succeed");

    let insert_result = engine.execute("INSERT INTO orders VALUES (1, 1, 10)");
    assert!(insert_result.is_ok(), "INSERT should trigger trigger");

    let inventory = engine
        .execute("SELECT stock FROM inventory WHERE product_id = 1")
        .unwrap();
    assert_eq!(inventory.rows.len(), 1);
}

#[test]
fn test_trigger_executes_update() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE products (id INTEGER, name TEXT, price INTEGER)")
        .unwrap();
    engine
        .execute(
            "CREATE TABLE price_history (product_id INTEGER, old_price INTEGER, new_price INTEGER)",
        )
        .unwrap();

    engine
        .execute("INSERT INTO products VALUES (1, 'Widget', 100)")
        .unwrap();

    let result = engine.execute(
        "CREATE TRIGGER log_price_change AFTER UPDATE ON products FOR EACH ROW BEGIN INSERT INTO price_history VALUES (OLD.id, OLD.price, NEW.price); END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER should succeed");

    let update_result = engine.execute("UPDATE products SET price = 150 WHERE id = 1");
    assert!(update_result.is_ok(), "UPDATE should trigger trigger");

    let history = engine.execute("SELECT * FROM price_history").unwrap();
    assert_eq!(history.rows.len(), 1);
}

#[test]
fn test_trigger_executes_delete() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE orders (id INTEGER, status TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE cancelled_orders (id INTEGER, status TEXT)")
        .unwrap();

    engine
        .execute("INSERT INTO orders VALUES (1, 'cancelled')")
        .unwrap();

    let result = engine.execute(
        "CREATE TRIGGER move_cancelled BEFORE DELETE ON orders FOR EACH ROW BEGIN INSERT INTO cancelled_orders VALUES (OLD.id, OLD.status); END"
    );
    assert!(result.is_ok(), "CREATE TRIGGER should succeed");

    let delete_result = engine.execute("DELETE FROM orders WHERE id = 1");
    assert!(delete_result.is_ok(), "DELETE should trigger trigger");

    let remaining = engine.execute("SELECT * FROM orders").unwrap();
    assert_eq!(remaining.rows.len(), 0);

    let cancelled = engine.execute("SELECT * FROM cancelled_orders").unwrap();
    assert_eq!(cancelled.rows.len(), 1);
}
