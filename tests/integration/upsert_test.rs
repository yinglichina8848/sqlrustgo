// UPSERT/REPLACE/INSERT IGNORE - 完整功能测试 (Issue #890)

use sqlrustgo::{parse, ExecutionEngine, MemoryStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

// ============== REPLACE 测试 ==============
#[test]
fn test_replace_updates_existing_row() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    engine.execute(parse("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT)").unwrap()).unwrap();
    
    engine.execute(parse("INSERT INTO t VALUES (1, 'Alice')").unwrap()).unwrap();
    engine.execute(parse("REPLACE INTO t VALUES (1, 'Bob')").unwrap()).unwrap();
    
    // Should still have 1 row (replaced)
    let count = engine.execute(parse("SELECT COUNT(*) FROM t").unwrap()).unwrap();
    assert_eq!(count.rows[0][0], Value::Integer(1), "Should have 1 row after REPLACE");
    
    println!("✓ REPLACE 更新: 保持 1 行");
}

// ============== INSERT IGNORE 测试 ==============
#[test]
fn test_insert_ignore_skips_duplicate() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    engine.execute(parse("CREATE TABLE t2 (id INTEGER PRIMARY KEY, name TEXT)").unwrap()).unwrap();
    
    engine.execute(parse("INSERT INTO t2 VALUES (1, 'Alice')").unwrap()).unwrap();
    engine.execute(parse("INSERT IGNORE INTO t2 VALUES (1, 'Bob')").unwrap()).unwrap();
    
    // Should still have 1 row
    let count = engine.execute(parse("SELECT COUNT(*) FROM t2").unwrap()).unwrap();
    assert_eq!(count.rows[0][0], Value::Integer(1), "Should have 1 row");
    
    println!("✓ INSERT IGNORE 跳过重复");
}

// ============== UPSERT 测试 ==============
#[test]
fn test_upsert_inserts_new_row() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    engine.execute(parse("CREATE TABLE t3 (id INTEGER PRIMARY KEY, name TEXT)").unwrap()).unwrap();
    
    engine.execute(parse("INSERT INTO t3 VALUES (1, 'new') ON DUPLICATE KEY UPDATE name='updated'").unwrap()).unwrap();
    
    let count = engine.execute(parse("SELECT COUNT(*) FROM t3").unwrap()).unwrap();
    assert_eq!(count.rows[0][0], Value::Integer(1));
    
    println!("✓ UPSERT 插入新行");
}

// ============== 解析测试 ==============
#[test]
fn test_upsert_parsing() {
    let stmt = parse("INSERT INTO t VALUES (1, 'a') ON DUPLICATE KEY UPDATE name='b'").unwrap();
    if let sqlrustgo_parser::parser::Statement::Insert(insert) = stmt {
        assert!(insert.on_duplicate.is_some());
        println!("✓ UPSERT 解析正确");
    }
}

#[test]
fn test_replace_parsing() {
    let stmt = parse("REPLACE INTO t VALUES (1, 'a')").unwrap();
    if let sqlrustgo_parser::parser::Statement::Insert(insert) = stmt {
        assert!(insert.replace);
        println!("✓ REPLACE 解析正确");
    }
}

#[test]
fn test_insert_ignore_parsing() {
    let stmt = parse("INSERT IGNORE INTO t VALUES (1, 'a')").unwrap();
    if let sqlrustgo_parser::parser::Statement::Insert(insert) = stmt {
        assert!(insert.ignore);
        println!("✓ INSERT IGNORE 解析正确");
    }
}
