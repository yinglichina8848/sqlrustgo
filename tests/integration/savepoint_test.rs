// SAVEPOINT Integration Tests (Issue #892)
// 
// Note: These tests verify parsing and basic transaction flow.

use sqlrustgo::{parse, ExecutionEngine, MemoryStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

#[test]
fn test_savepoint_parsing() {
    // Test SAVEPOINT parsing
    let result = parse("SAVEPOINT sp1");
    assert!(result.is_ok(), "SAVEPOINT should parse: {:?}", result.err());
    
    let result = parse("ROLLBACK TO SAVEPOINT sp1");
    assert!(result.is_ok(), "ROLLBACK TO SAVEPOINT should parse: {:?}", result.err());
    
    let result = parse("RELEASE SAVEPOINT sp1");
    assert!(result.is_ok(), "RELEASE SAVEPOINT should parse: {:?}", result.err());
    
    println!("✓ SAVEPOINT parsing works");
}

#[test]
fn test_transaction_basic() {
    // Basic transaction test
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine.execute(parse("CREATE TABLE tx_test (id INTEGER)").unwrap()).unwrap();
    
    engine.execute(parse("BEGIN").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO tx_test VALUES (1)").unwrap()).unwrap();
    
    // Commit
    engine.execute(parse("COMMIT").unwrap()).unwrap();
    
    // Verify row was inserted
    let result = engine.execute(parse("SELECT COUNT(*) FROM tx_test").unwrap()).unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(1));
    
    println!("✓ Basic transaction works");
}

#[test]
fn test_multiple_savepoints_parsing() {
    // Test multiple savepoints can be parsed
    let result = parse("SAVEPOINT sp1");
    assert!(result.is_ok());
    
    let result = parse("SAVEPOINT sp2");
    assert!(result.is_ok());
    
    let result = parse("ROLLBACK TO SAVEPOINT sp1");
    assert!(result.is_ok());
    
    let result = parse("RELEASE SAVEPOINT sp2");
    assert!(result.is_ok());
    
    println!("✓ Multiple SAVEPOINTs parsing works");
}

#[test]
fn test_nested_savepoint_parsing() {
    // Test nested savepoint parsing
    let result = parse("SAVEPOINT outer");
    assert!(result.is_ok());
    
    let result = parse("SAVEPOINT inner");
    assert!(result.is_ok());
    
    let result = parse("ROLLBACK TO SAVEPOINT outer");
    assert!(result.is_ok());
    
    println!("✓ Nested SAVEPOINTs parsing works");
}
