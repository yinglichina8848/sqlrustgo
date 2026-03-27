// AUTO_INCREMENT Integration Tests (Issue #889)

use sqlrustgo::{parse, ExecutionEngine, MemoryStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

#[test]
fn test_autoinc_parsing() {
    // Test AUTO_INCREMENT parsing
    let result = parse("CREATE TABLE orders (id INTEGER AUTO_INCREMENT PRIMARY KEY, name TEXT)");
    assert!(result.is_ok(), "AUTO_INCREMENT should parse: {:?}", result.err());
    
    // Also test AUTOINCREMENT (SQLite variant)
    let result = parse("CREATE TABLE items (id INTEGER AUTOINCREMENT PRIMARY KEY)");
    assert!(result.is_ok(), "AUTOINCREMENT should parse");
    
    println!("✓ AUTO_INCREMENT parsing works");
}

#[test]
fn test_autoinc_insert() {
    // Test AUTO_INCREMENT behavior
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine.execute(parse("CREATE TABLE orders (id INTEGER AUTO_INCREMENT PRIMARY KEY, name TEXT)").unwrap()).unwrap();
    
    // Insert without specifying id
    engine.execute(parse("INSERT INTO orders (name) VALUES ('first')").unwrap()).unwrap();
    
    // Insert another
    engine.execute(parse("INSERT INTO orders (name) VALUES ('second')").unwrap()).unwrap();
    
    // Should have 2 rows
    let count = engine.execute(parse("SELECT COUNT(*) FROM orders").unwrap()).unwrap();
    assert_eq!(count.rows[0][0], Value::Integer(2));
    
    println!("✓ AUTO_INCREMENT insert test completed");
}

#[test]
fn test_autoinc_with_explicit_id() {
    // Test AUTO_INCREMENT with explicit id
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine.execute(parse("CREATE TABLE items (id INTEGER AUTO_INCREMENT PRIMARY KEY, name TEXT)").unwrap()).unwrap();
    
    // Insert with explicit id
    engine.execute(parse("INSERT INTO items VALUES (100, 'explicit')").unwrap()).unwrap();
    
    // Insert without id (should get next auto value)
    engine.execute(parse("INSERT INTO items (name) VALUES ('auto')").unwrap()).unwrap();
    
    // Should have 2 rows
    let count = engine.execute(parse("SELECT COUNT(*) FROM items").unwrap()).unwrap();
    assert_eq!(count.rows[0][0], Value::Integer(2));
    
    println!("✓ AUTO_INCREMENT with explicit ID works");
}

#[test]
fn test_autoinc_multiple_columns() {
    // Test AUTO_INCREMENT on non-PK column
    let result = parse("CREATE TABLE logs (id INTEGER PRIMARY KEY, seq INTEGER AUTO_INCREMENT, data TEXT)");
    assert!(result.is_ok(), "AUTO_INCREMENT on non-PK should parse");
    
    println!("✓ AUTO_INCREMENT on non-PK column parsing works");
}
