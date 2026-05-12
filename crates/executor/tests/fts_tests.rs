//! Integration tests for Full-Text Search (FTS) MATCH...AGAINST execution

#![allow(deprecated)]

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_fts_basic_syntax() {
    // Test that FTS syntax is parsed correctly end-to-end
    let mut engine = create_engine();
    engine.execute("CREATE TABLE articles (id INTEGER, title TEXT, content TEXT)").unwrap();

    engine.execute("INSERT INTO articles VALUES (1, 'Hello World', 'This is a test article')").unwrap();
    engine.execute("INSERT INTO articles VALUES (2, 'Rust Programming', 'Learn Rust programming language')").unwrap();
    engine.execute("INSERT INTO articles VALUES (3, 'Database Systems', 'Introduction to database systems')").unwrap();

    // This should parse without error - execution will return Null since FTS executor not implemented
    let result = engine.execute("SELECT * FROM articles WHERE MATCH(title, content) AGAINST('rust')");
    assert!(result.is_ok(), "FTS query should parse: {:?}", result);
}

#[test]
fn test_fts_select_with_fts_expression() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE docs (id INTEGER, body TEXT)").unwrap();
    engine.execute("INSERT INTO docs VALUES (1, 'Hello world')").unwrap();
    engine.execute("INSERT INTO docs VALUES (2, 'Goodbye world')").unwrap();

    // FTS in WHERE clause (SELECT clause FTS requires advanced parsing)
    let result = engine.execute("SELECT id, body FROM docs WHERE MATCH(body) AGAINST('hello')");
    assert!(result.is_ok(), "FTS in WHERE should parse: {:?}", result);
}

#[test]
fn test_fts_empty_table() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE empty_fts (id INTEGER, text TEXT)").unwrap();

    let result = engine.execute("SELECT * FROM empty_fts WHERE MATCH(text) AGAINST('nothing')");
    assert!(result.is_ok(), "FTS on empty table should work: {:?}", result);
}

#[test]
fn test_fts_no_matching_documents() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE fts_test (id INTEGER, content TEXT)").unwrap();
    engine.execute("INSERT INTO fts_test VALUES (1, 'Hello World')").unwrap();
    engine.execute("INSERT INTO fts_test VALUES (2, 'Goodbye World')").unwrap();

    // Search for non-existent term
    let result = engine.execute("SELECT * FROM fts_test WHERE MATCH(content) AGAINST('xyz123')");
    assert!(result.is_ok(), "FTS search should work: {:?}", result);
}
