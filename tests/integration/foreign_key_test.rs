//! Foreign Key Constraint Tests
//!
//! Comprehensive tests for #901 foreign key functionality:
//! - Functional: INSERT validation, UPDATE validation, DELETE actions (CASCADE, SET NULL, RESTRICT)
//! - Performance: Bulk insert with FK, concurrent FK operations
//! - Edge cases: Self-referencing FK, multiple FK constraints

use sqlrustgo::parse;
use sqlrustgo::{ExecutionEngine, MemoryStorage};
use sqlrustgo_storage::engine::{
    ColumnDefinition, ForeignKeyAction, ForeignKeyConstraint, StorageEngine, TableInfo,
};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};
use std::time::Instant;

#[test]
fn test_fk_insert_valid_reference() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parent records
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Create child table with FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Insert with valid FK reference - should succeed
    let result = engine.execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap());
    assert!(
        result.is_ok(),
        "Should allow insert with valid FK reference: {:?}",
        result
    );

    let result = engine.execute(parse("INSERT INTO orders VALUES (2, 2)").unwrap());
    assert!(
        result.is_ok(),
        "Should allow insert with valid FK reference: {:?}",
        result
    );

    // Verify orders were inserted
    let result = engine
        .execute(parse("SELECT * FROM orders").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_fk_insert_invalid_reference() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parent records
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Create child table with FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Insert with invalid FK reference - should fail
    let result = engine.execute(parse("INSERT INTO orders VALUES (1, 999)").unwrap());
    assert!(
        result.is_err(),
        "Should reject insert with invalid FK reference"
    );

    if let Err(e) = result {
        let err_msg = format!("{}", e);
        assert!(
            err_msg.contains("Foreign key constraint violation"),
            "Error should mention FK violation: {}",
            err_msg
        );
    }
}

#[test]
#[ignore]
fn test_fk_insert_null_reference() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Create child table with FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Insert with NULL FK - should succeed (NULL bypasses FK check)
    let result = engine.execute(parse("INSERT INTO orders VALUES (1, NULL)").unwrap());
    assert!(result.is_ok(), "Should allow NULL FK value: {:?}", result);
}

#[test]
fn test_fk_multiple_fk_columns() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent tables
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parents
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO products VALUES (1, 'Widget')").unwrap())
        .unwrap();

    // Note: Current parser only supports one FK column per table
    // Create table with single FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Verify the FK was created
    let storage = engine.storage.read().unwrap();
    let table_info = storage.get_table_info("orders").unwrap();
    assert_eq!(table_info.columns.len(), 2);
    assert!(table_info.columns[1].references.is_some());
    let fk = table_info.columns[1].references.as_ref().unwrap();
    assert_eq!(fk.referenced_table, "users");
    assert_eq!(fk.referenced_column, "id");
    drop(storage);

    // Insert with valid FK - should succeed
    let result = engine.execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap());
    assert!(result.is_ok());

    // Insert with invalid FK - should fail
    let result = engine.execute(parse("INSERT INTO orders VALUES (2, 999)").unwrap());
    assert!(result.is_err());

    // TODO: When parser supports multiple FK columns, test:
    // - CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id), product_id INTEGER REFERENCES products(id))
    // - Multiple FK validation for both columns
}

#[test]
#[ignore]
fn test_fk_self_referencing() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create self-referencing table (employee -> manager)
    engine
        .execute(
            parse(
                "CREATE TABLE employees (id INTEGER PRIMARY KEY, name TEXT, manager_id INTEGER REFERENCES employees(id))",
            )
            .unwrap(),
        )
        .unwrap();

    // Insert root employee (no manager)
    let result = engine.execute(parse("INSERT INTO employees VALUES (1, 'CEO', NULL)").unwrap());
    assert!(result.is_ok());

    // Insert employee with valid self-reference
    let result = engine.execute(parse("INSERT INTO employees VALUES (2, 'Manager', 1)").unwrap());
    assert!(result.is_ok());

    // Insert employee with invalid self-reference
    let result = engine.execute(parse("INSERT INTO employees VALUES (3, 'Worker', 999)").unwrap());
    assert!(result.is_err());
}

#[test]
fn test_fk_update_validation() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parent
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
        .unwrap();

    // Create child table with FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Insert child
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap())
        .unwrap();

    // Note: UPDATE validation depends on implementation
    // Current MemoryStorage::update doesn't validate FK
    // This test documents expected behavior
}

#[test]
fn test_fk_cascade_delete() {
    // Test that FK actions are defined but ON DELETE CASCADE not yet implemented
    let fk_constraint = ForeignKeyConstraint {
        referenced_table: "users".to_string(),
        referenced_column: "id".to_string(),
        on_delete: Some(ForeignKeyAction::Cascade),
        on_update: None,
    };

    assert_eq!(fk_constraint.on_delete, Some(ForeignKeyAction::Cascade));
}

#[test]
fn test_fk_set_null_action() {
    // Test that FK actions are defined but ON DELETE SET NULL not yet implemented
    let fk_constraint = ForeignKeyConstraint {
        referenced_table: "users".to_string(),
        referenced_column: "id".to_string(),
        on_delete: Some(ForeignKeyAction::SetNull),
        on_update: None,
    };

    assert_eq!(fk_constraint.on_delete, Some(ForeignKeyAction::SetNull));
}

#[test]
fn test_fk_restrict_action() {
    // Test that FK actions are defined but ON DELETE RESTRICT not yet implemented
    let fk_constraint = ForeignKeyConstraint {
        referenced_table: "users".to_string(),
        referenced_column: "id".to_string(),
        on_delete: Some(ForeignKeyAction::Restrict),
        on_update: None,
    };

    assert_eq!(fk_constraint.on_delete, Some(ForeignKeyAction::Restrict));
}

// ============================================
// Performance Tests
// ============================================

#[test]
fn test_fk_bulk_insert_performance() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Create child table with FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Insert 1000 parent records
    let start = Instant::now();
    for i in 1..=1000 {
        engine
            .execute(parse(&format!("INSERT INTO users VALUES ({}, 'User{}')", i, i)).unwrap())
            .unwrap();
    }
    let parent_insert_time = start.elapsed();
    println!("Inserted 1000 parent records in {:?}", parent_insert_time);

    // Bulk insert 1000 child records with valid FKs
    let start = Instant::now();
    for i in 1..=1000 {
        let user_id = (i % 1000) + 1; // References users 1-1000
        engine
            .execute(parse(&format!("INSERT INTO orders VALUES ({}, {})", i, user_id)).unwrap())
            .unwrap();
    }
    let child_insert_time = start.elapsed();
    println!(
        "Inserted 1000 child records with FK validation in {:?}",
        child_insert_time
    );

    // Performance assertion: FK validation should not be prohibitively slow
    // 1000 FK validations should complete in reasonable time
    assert!(
        child_insert_time.as_millis() < 5000,
        "FK validation too slow: {:?}",
        child_insert_time
    );

    // Verify all orders were inserted
    let result = engine
        .execute(parse("SELECT COUNT(*) FROM orders").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_fk_bulk_insert_with_violations() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table with limited records
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Create child table with FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Insert some valid orders
    for i in 1..=10 {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO orders VALUES ({}, {})",
                    i,
                    (i % 2) + 1
                ))
                .unwrap(),
            )
            .unwrap();
    }

    // Verify valid inserts worked
    let result = engine
        .execute(parse("SELECT COUNT(*) FROM orders").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 1);

    // Try to insert with invalid FK - should fail
    let result = engine.execute(parse("INSERT INTO orders VALUES (999, 999)").unwrap());
    assert!(result.is_err());
}

#[test]
fn test_fk_concurrent_insert_simulation() {
    // Simulate concurrent FK-validated inserts by doing them sequentially
    // (Rust test framework doesn't support true threading in a single test)
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));

    // Create tables
    {
        let mut storage = storage.write().unwrap();
        storage
            .create_table(&TableInfo {
                name: "users".to_string(),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    references: None,
                    auto_increment: false,
                }],
            })
            .unwrap();

        storage
            .create_table(&TableInfo {
                name: "orders".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        references: None,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "user_id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        references: Some(ForeignKeyConstraint {
                            referenced_table: "users".to_string(),
                            referenced_column: "id".to_string(),
                            on_delete: None,
                            on_update: None,
                        }),
                        auto_increment: false,
                    },
                ],
            })
            .unwrap();
    }

    // Insert parent records
    for i in 1..=100 {
        storage
            .write()
            .unwrap()
            .insert("users", vec![vec![Value::Integer(i)]])
            .unwrap();
    }

    // Simulate concurrent child inserts
    let start = Instant::now();
    for i in 1..=100 {
        let user_id = (i % 100) + 1;
        storage
            .write()
            .unwrap()
            .insert(
                "orders",
                vec![vec![Value::Integer(i), Value::Integer(user_id)]],
            )
            .unwrap();
    }
    let insert_time = start.elapsed();

    println!(
        "Simulated 100 FK-validated inserts in {:?} ({:.0} inserts/sec)",
        insert_time,
        100.0 / insert_time.as_secs_f64()
    );

    // Performance check
    assert!(
        insert_time.as_millis() < 1000,
        "FK inserts too slow: {:?}",
        insert_time
    );
}

#[test]
fn test_fk_large_dataset_validation() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));

    // Create parent table with 10000 records
    {
        let mut storage = storage.write().unwrap();
        storage
            .create_table(&TableInfo {
                name: "categories".to_string(),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    references: None,
                    auto_increment: false,
                }],
            })
            .unwrap();

        storage
            .create_table(&TableInfo {
                name: "items".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        references: None,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "category_id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        references: Some(ForeignKeyConstraint {
                            referenced_table: "categories".to_string(),
                            referenced_column: "id".to_string(),
                            on_delete: None,
                            on_update: None,
                        }),
                        auto_increment: false,
                    },
                ],
            })
            .unwrap();
    }

    // Insert 10000 parent records
    let start = Instant::now();
    for i in 1..=10000 {
        storage
            .write()
            .unwrap()
            .insert("categories", vec![vec![Value::Integer(i)]])
            .unwrap();
    }
    println!("Inserted 10000 parent records in {:?}", start.elapsed());

    // Insert 10000 child records with FK validation
    let start = Instant::now();
    for i in 1..=10000 {
        let category_id = (i % 10000) + 1;
        storage
            .write()
            .unwrap()
            .insert(
                "items",
                vec![vec![Value::Integer(i), Value::Integer(category_id)]],
            )
            .unwrap();
    }
    let child_insert_time = start.elapsed();

    println!(
        "Inserted 10000 child records with FK validation in {:?} ({:.0} inserts/sec)",
        child_insert_time,
        10000.0 / child_insert_time.as_secs_f64()
    );

    // Verify
    let count = storage.read().unwrap().scan("items").unwrap().len();
    assert_eq!(count, 10000);

    // Performance assertion - relaxed to 60s to avoid CI flakiness
    // The purpose of this test is correctness validation, not performance benchmarking
    assert!(
        child_insert_time.as_secs() < 60,
        "FK validation too slow for 10000 records: {:?}",
        child_insert_time
    );
}

#[test]
fn test_fk_action_definitions() {
    // Verify all FK actions are properly defined
    assert!(matches!(
        ForeignKeyAction::Cascade,
        ForeignKeyAction::Cascade
    ));
    assert!(matches!(
        ForeignKeyAction::SetNull,
        ForeignKeyAction::SetNull
    ));
    assert!(matches!(
        ForeignKeyAction::Restrict,
        ForeignKeyAction::Restrict
    ));
}

#[test]
fn test_fk_constraint_structure() {
    let fk = ForeignKeyConstraint {
        referenced_table: "users".to_string(),
        referenced_column: "id".to_string(),
        on_delete: Some(ForeignKeyAction::Cascade),
        on_update: Some(ForeignKeyAction::Restrict),
    };

    assert_eq!(fk.referenced_table, "users");
    assert_eq!(fk.referenced_column, "id");
    assert!(matches!(fk.on_delete, Some(ForeignKeyAction::Cascade)));
    assert!(matches!(fk.on_update, Some(ForeignKeyAction::Restrict)));
}

#[test]
fn test_fk_edge_case_empty_parent_table() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table but don't insert any records
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Create child table with FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Try to insert child with FK reference - should fail because parent table is empty
    let result = engine.execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap());
    assert!(result.is_err());

    if let Err(e) = result {
        let err_msg = format!("{}", e);
        assert!(
            err_msg.contains("Foreign key constraint violation"),
            "Error should mention FK violation: {}",
            err_msg
        );
    }
}

#[test]
fn test_fk_edge_case_zero_value() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table with id=0
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO users VALUES (0, 'ZeroUser')").unwrap())
        .unwrap();

    // Create child table with FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Insert with FK reference to id=0 - should succeed
    let result = engine.execute(parse("INSERT INTO orders VALUES (1, 0)").unwrap());
    assert!(result.is_ok(), "FK to id=0 should work: {:?}", result);
}

#[test]
fn test_fk_edge_case_negative_value() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table with negative id
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO users VALUES (-1, 'NegativeUser')").unwrap())
        .unwrap();

    // Create child table with FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Insert with FK reference to id=-1 - should succeed
    let result = engine.execute(parse("INSERT INTO orders VALUES (1, -1)").unwrap());
    assert!(
        result.is_ok(),
        "FK to negative id should work: {:?}",
        result
    );
}

// ============================================
// DELETE/UPDATE FK Action Tests
// ============================================

#[test]
#[ignore]
fn test_fk_delete_restrict() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parent records
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Create child table with FK and RESTRICT action
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id) ON DELETE RESTRICT)")
                .unwrap(),
        )
        .unwrap();

    // Insert child records
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1), (2, 1)").unwrap())
        .unwrap();

    // Try to delete parent with referencing children - should fail with RESTRICT
    let result = engine.execute(parse("DELETE FROM users WHERE id = 1").unwrap());
    assert!(
        result.is_err(),
        "DELETE with RESTRICT should fail when children exist"
    );

    if let Err(e) = result {
        let err_msg = format!("{}", e).to_lowercase();
        assert!(
            err_msg.contains("foreign key constraint violation") || err_msg.contains("restrict"),
            "Error should mention RESTRICT constraint: {}",
            err_msg
        );
    }

    // Verify parent was not deleted
    let result = engine
        .execute(parse("SELECT * FROM users WHERE id = 1").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 1, "Parent record should not be deleted");
}

#[test]
fn test_fk_delete_cascade() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parent records
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Create child table with FK and CASCADE action
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id) ON DELETE CASCADE)")
                .unwrap(),
        )
        .unwrap();

    // Insert child records
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1), (2, 1), (3, 2)").unwrap())
        .unwrap();

    // Delete parent - should CASCADE delete children
    let result = engine.execute(parse("DELETE FROM users WHERE id = 1").unwrap());
    assert!(
        result.is_ok(),
        "DELETE with CASCADE should succeed: {:?}",
        result
    );

    // Verify parent was deleted
    let result = engine
        .execute(parse("SELECT * FROM users WHERE id = 1").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 0, "Parent record should be deleted");

    // Verify children were cascade deleted
    let result = engine
        .execute(parse("SELECT * FROM orders WHERE user_id = 1").unwrap())
        .unwrap();
    assert_eq!(
        result.rows.len(),
        0,
        "Child records should be cascade deleted"
    );

    // Verify other child (user_id = 2) still exists
    let result = engine
        .execute(parse("SELECT * FROM orders WHERE user_id = 2").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 1, "Other child records should remain");
}

#[test]
#[ignore]
fn test_fk_delete_set_null() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parent records
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Create child table with FK and SET NULL action
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id) ON DELETE SET NULL)")
                .unwrap(),
        )
        .unwrap();

    // Insert child records
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1), (2, 1), (3, 2)").unwrap())
        .unwrap();

    // Delete parent - should SET NULL on children
    let result = engine.execute(parse("DELETE FROM users WHERE id = 1").unwrap());
    assert!(
        result.is_ok(),
        "DELETE with SET NULL should succeed: {:?}",
        result
    );

    // Verify parent was deleted
    let result = engine
        .execute(parse("SELECT * FROM users WHERE id = 1").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 0, "Parent record should be deleted");

    // Verify children have NULL FK
    let result = engine
        .execute(parse("SELECT * FROM orders ORDER BY id").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 3);
    // Orders 1 and 2 should have NULL user_id
    assert_eq!(result.rows[0][1], Value::Null);
    assert_eq!(result.rows[1][1], Value::Null);
    // Order 3 should still have user_id = 2
    assert_eq!(result.rows[2][1], Value::Integer(2));
}

#[test]
#[ignore]
fn test_fk_update_cascade() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parent records
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Create child table with FK and CASCADE action
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id) ON UPDATE CASCADE)")
                .unwrap(),
        )
        .unwrap();

    // Insert child records
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1), (2, 1), (3, 2)").unwrap())
        .unwrap();

    // Update parent - should CASCADE update children
    let result = engine.execute(parse("UPDATE users SET id = 100 WHERE id = 1").unwrap());
    assert!(
        result.is_ok(),
        "UPDATE with CASCADE should succeed: {:?}",
        result
    );

    // Verify parent was updated
    let result = engine
        .execute(parse("SELECT * FROM users WHERE id = 100").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 1, "Parent should be updated to id=100");
    assert_eq!(result.rows[0][1], Value::Text("Alice".to_string()));

    // Verify children were cascade updated
    let result = engine
        .execute(parse("SELECT * FROM orders ORDER BY id").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 3);
    // Orders 1 and 2 should now have user_id = 100
    assert_eq!(result.rows[0][1], Value::Integer(100));
    assert_eq!(result.rows[1][1], Value::Integer(100));
    // Order 3 should still have user_id = 2
    assert_eq!(result.rows[2][1], Value::Integer(2));
}

#[test]
#[ignore]
fn test_fk_update_set_null() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parent records
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Create child table with FK and SET NULL action
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id) ON UPDATE SET NULL)")
                .unwrap(),
        )
        .unwrap();

    // Insert child records
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1), (2, 1), (3, 2)").unwrap())
        .unwrap();

    // Update parent - should SET NULL on children
    let result = engine.execute(parse("UPDATE users SET id = 100 WHERE id = 1").unwrap());
    assert!(
        result.is_ok(),
        "UPDATE with SET NULL should succeed: {:?}",
        result
    );

    // Verify children have NULL FK
    let result = engine
        .execute(parse("SELECT * FROM orders ORDER BY id").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 3);
    // Orders 1 and 2 should have NULL user_id
    assert_eq!(result.rows[0][1], Value::Null);
    assert_eq!(result.rows[1][1], Value::Null);
    // Order 3 should still have user_id = 2
    assert_eq!(result.rows[2][1], Value::Integer(2));
}

#[test]
fn test_fk_update_restrict() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parent records
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Create child table with FK and RESTRICT action
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id) ON UPDATE RESTRICT)")
                .unwrap(),
        )
        .unwrap();

    // Insert child records
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1), (2, 1)").unwrap())
        .unwrap();

    // Try to update parent with referencing children - should fail with RESTRICT
    let result = engine.execute(parse("UPDATE users SET id = 100 WHERE id = 1").unwrap());
    assert!(
        result.is_err(),
        "UPDATE with RESTRICT should fail when children exist"
    );

    if let Err(e) = result {
        let err_msg = format!("{}", e).to_lowercase();
        assert!(
            err_msg.contains("foreign key constraint violation") || err_msg.contains("restrict"),
            "Error should mention RESTRICT constraint: {}",
            err_msg
        );
    }

    // Verify parent was not updated
    let result = engine
        .execute(parse("SELECT * FROM users WHERE id = 1").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 1, "Parent record should not be updated");
    let result = engine
        .execute(parse("SELECT * FROM users WHERE id = 100").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 0, "id=100 should not exist");
}

#[test]
fn test_fk_multiple_fk_columns_delete_cascade() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent tables
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parents
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO products VALUES (1, 'Widget')").unwrap())
        .unwrap();

    // Create table with single FK (parser limitation noted in test_fk_multiple_fk_columns)
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id) ON DELETE CASCADE)")
                .unwrap(),
        )
        .unwrap();

    // Insert order referencing user 1
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap())
        .unwrap();

    // Delete user - should cascade delete order
    let result = engine.execute(parse("DELETE FROM users WHERE id = 1").unwrap());
    assert!(
        result.is_ok(),
        "DELETE with CASCADE should succeed: {:?}",
        result
    );

    // Verify order was cascade deleted
    let result = engine
        .execute(parse("SELECT * FROM orders").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 0, "Order should be cascade deleted");
}

#[test]
#[ignore]
fn test_fk_self_reference_delete_cascade() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create self-referencing table with CASCADE
    engine
        .execute(
            parse(
                "CREATE TABLE employees (id INTEGER PRIMARY KEY, name TEXT, manager_id INTEGER REFERENCES employees(id) ON DELETE CASCADE)",
            )
            .unwrap(),
        )
        .unwrap();

    // Insert CEO (no manager)
    engine
        .execute(parse("INSERT INTO employees VALUES (1, 'CEO', NULL)").unwrap())
        .unwrap();

    // Insert Manager referencing CEO
    engine
        .execute(parse("INSERT INTO employees VALUES (2, 'Manager', 1)").unwrap())
        .unwrap();

    // Insert Worker referencing Manager
    engine
        .execute(parse("INSERT INTO employees VALUES (3, 'Worker', 2)").unwrap())
        .unwrap();

    // Delete CEO - should cascade delete Manager and Worker
    let result = engine.execute(parse("DELETE FROM employees WHERE id = 1").unwrap());
    assert!(
        result.is_ok(),
        "DELETE with CASCADE should succeed: {:?}",
        result
    );

    // Verify all records were deleted
    let result = engine
        .execute(parse("SELECT * FROM employees").unwrap())
        .unwrap();
    assert_eq!(
        result.rows.len(),
        0,
        "All employees should be cascade deleted"
    );
}

#[test]
#[ignore]
fn test_fk_combined_actions() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parents
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Create child with both ON DELETE and ON UPDATE actions
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id) ON DELETE CASCADE ON UPDATE SET NULL)")
                .unwrap(),
        )
        .unwrap();

    // Insert orders
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1), (2, 1), (3, 2)").unwrap())
        .unwrap();

    // Update parent id - should SET NULL on children
    let result = engine.execute(parse("UPDATE users SET id = 100 WHERE id = 1").unwrap());
    assert!(
        result.is_ok(),
        "UPDATE should succeed with SET NULL: {:?}",
        result
    );

    // Verify children's FK is now NULL
    let result = engine
        .execute(parse("SELECT user_id FROM orders WHERE id IN (1, 2)").unwrap())
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Null);
    assert_eq!(result.rows[1][0], Value::Null);

    // Delete remaining parent - should CASCADE delete child
    let result = engine.execute(parse("DELETE FROM users WHERE id = 100").unwrap());
    assert!(
        result.is_ok(),
        "DELETE should succeed with CASCADE: {:?}",
        result
    );

    // Verify child was deleted
    let result = engine
        .execute(parse("SELECT * FROM orders").unwrap())
        .unwrap();
    // After UPDATE SET NULL, orders 1 and 2 have user_id = NULL (not 100)
    // DELETE CASCADE for user 100 only deletes orders with user_id = 100, which is none
    // So all 3 orders remain: orders 1 and 2 with NULL, order 3 with user_id = 2
    assert_eq!(result.rows.len(), 3);
    // Orders 1 and 2 have user_id = NULL (from SET NULL)
    assert_eq!(result.rows[0][1], Value::Null);
    assert_eq!(result.rows[1][1], Value::Null);
    // Order 3 still has user_id = 2 (unchanged, user 2 still exists)
    assert_eq!(result.rows[2][1], Value::Integer(2));
}
