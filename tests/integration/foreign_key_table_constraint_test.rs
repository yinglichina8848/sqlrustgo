//! Foreign Key Table Constraint Integration Tests
//!
//! Tests for Issue #1379: Table-level FOREIGN KEY constraint syntax

use sqlrustgo::parse;
use sqlrustgo::{ExecutionEngine, MemoryStorage};
use std::sync::{Arc, RwLock};

#[test]
fn test_fk_table_constraint_single_column() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Table-level FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Valid FK reference
    let result = engine.execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap());
    assert!(
        result.is_ok(),
        "Should allow insert with valid FK: {:?}",
        result
    );

    // Invalid FK reference
    let result = engine.execute(parse("INSERT INTO orders VALUES (2, 999)").unwrap());
    assert!(result.is_err(), "Should reject insert with invalid FK");
}

#[test]
fn test_fk_table_constraint_multi_column() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Parent table with composite PK
    engine
        .execute(
            parse("CREATE TABLE order_products (order_id INTEGER, product_id INTEGER, PRIMARY KEY (order_id, product_id))")
                .unwrap(),
        )
        .unwrap();
    engine
        .execute(parse("INSERT INTO order_products VALUES (1, 100)").unwrap())
        .unwrap();

    // Child table with multi-column FK
    engine
        .execute(
            parse("CREATE TABLE line_items (id INTEGER, order_id INTEGER, product_id INTEGER, FOREIGN KEY (order_id, product_id) REFERENCES order_products(order_id, product_id))")
                .unwrap(),
        )
        .unwrap();

    // Valid composite FK
    let result = engine.execute(parse("INSERT INTO line_items VALUES (1, 1, 100)").unwrap());
    assert!(
        result.is_ok(),
        "Should allow insert with valid composite FK: {:?}",
        result
    );

    // Partial match - should fail
    let result = engine.execute(parse("INSERT INTO line_items VALUES (2, 1, 999)").unwrap());
    assert!(
        result.is_err(),
        "Should reject insert with partial composite FK match"
    );
}

#[test]
fn test_fk_table_constraint_on_delete_cascade() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
        .unwrap();

    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE)")
                .unwrap(),
        )
        .unwrap();
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap())
        .unwrap();

    // Delete parent - should cascade delete child
    engine
        .execute(parse("DELETE FROM users WHERE id = 1").unwrap())
        .unwrap();

    let result = engine
        .execute(parse("SELECT * FROM orders").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 0, "Child rows should be cascade deleted");
}

#[test]
fn test_fk_table_constraint_on_update_set_null() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE SET NULL)")
                .unwrap(),
        )
        .unwrap();
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap())
        .unwrap();

    // Update parent PK - should SET NULL on child
    engine
        .execute(parse("UPDATE users SET id = 100 WHERE id = 1").unwrap())
        .unwrap();

    let result = engine
        .execute(parse("SELECT user_id FROM orders").unwrap())
        .unwrap();
    assert_eq!(result.rows[0][0], sqlrustgo_types::Value::Null);
}

#[test]
fn test_primary_key_table_constraint() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(
            parse("CREATE TABLE composite (a INTEGER, b INTEGER, c TEXT, PRIMARY KEY (a, b))")
                .unwrap(),
        )
        .unwrap();

    // Insert valid record
    let result = engine.execute(parse("INSERT INTO composite VALUES (1, 2, 'test')").unwrap());
    assert!(result.is_ok());

    // Insert duplicate PK - should fail
    let result = engine.execute(parse("INSERT INTO composite VALUES (1, 2, 'dup')").unwrap());
    assert!(result.is_err(), "Should reject duplicate primary key");
}

#[test]
fn test_fk_and_column_level_fk_together() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent tables
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY)").unwrap())
        .unwrap();
    engine
        .execute(parse("CREATE TABLE products (id INTEGER PRIMARY KEY)").unwrap())
        .unwrap();

    engine
        .execute(parse("INSERT INTO users VALUES (1)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO products VALUES (100)").unwrap())
        .unwrap();

    // Table with both column-level and table-level FKs
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id), product_id INTEGER, FOREIGN KEY (product_id) REFERENCES products(id))")
                .unwrap(),
        )
        .unwrap();

    // Both FKs valid
    let result = engine.execute(parse("INSERT INTO orders VALUES (1, 1, 100)").unwrap());
    assert!(result.is_ok());

    // Invalid column-level FK
    let result = engine.execute(parse("INSERT INTO orders VALUES (2, 999, 100)").unwrap());
    assert!(result.is_err());

    // Invalid table-level FK
    let result = engine.execute(parse("INSERT INTO orders VALUES (3, 1, 999)").unwrap());
    assert!(result.is_err());
}

#[test]
fn test_unique_table_constraint() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE users (id INTEGER, email TEXT, UNIQUE (email))").unwrap())
        .unwrap();

    let result = engine.execute(parse("INSERT INTO users VALUES (1, 'a@test.com')").unwrap());
    assert!(result.is_ok());

    // Duplicate unique value
    let result = engine.execute(parse("INSERT INTO users VALUES (2, 'a@test.com')").unwrap());
    assert!(result.is_err(), "Should reject duplicate unique value");
}
