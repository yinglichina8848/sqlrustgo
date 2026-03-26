// UPSERT (ON DUPLICATE KEY UPDATE) Integration Tests (Issue #890)

use sqlrustgo::{parse, ExecutionEngine, MemoryStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

#[test]
fn test_upsert_parsing() {
    // Test UPSERT parsing
    let result = parse("INSERT INTO users (id, name) VALUES (1, 'Alice') ON DUPLICATE KEY UPDATE name='Alice'");
    assert!(result.is_ok(), "UPSERT should parse: {:?}", result.err());
    
    println!("✓ UPSERT parsing works");
}

#[test]
fn test_upsert_basic_insert() {
    // Test basic UPSERT insert (when key doesn't exist)
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine.execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap()).unwrap();
    
    // UPSERT with non-existent key - should insert
    let result = engine.execute(
        parse("INSERT INTO users VALUES (1, 'new') ON DUPLICATE KEY UPDATE name='updated'").unwrap()
    );
    assert!(result.is_ok());
    
    // Should have 1 row
    let count = engine.execute(parse("SELECT COUNT(*) FROM users").unwrap()).unwrap();
    assert_eq!(count.rows[0][0], Value::Integer(1));
    
    println!("✓ UPSERT insert works");
}

#[test]
fn test_upsert_multiple_columns() {
    // Test UPSERT with multiple column updates
    let result = parse("INSERT INTO t (id, a, b) VALUES (1, 'x', 'y') ON DUPLICATE KEY UPDATE a='x2', b='y2'");
    assert!(result.is_ok(), "Multi-column UPSERT should parse");
    
    println!("✓ Multi-column UPSERT parsing works");
}

#[test]
fn test_upsert_with_expression() {
    // Test UPSERT with expression in UPDATE
    let result = parse("INSERT INTO t VALUES (1, 10) ON DUPLICATE KEY UPDATE count = count + 1");
    assert!(result.is_ok(), "UPSERT with expression should parse");
    
    println!("✓ UPSERT with expression parsing works");
}
