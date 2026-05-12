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

#[test]
fn test_fts_search_returns_correct_results() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE articles (id INTEGER, title TEXT, body TEXT)").unwrap();
    engine.execute("INSERT INTO articles VALUES (1, 'Rust Programming', 'Learn Rust programming language')").unwrap();
    engine.execute("INSERT INTO articles VALUES (2, 'Database Basics', 'Introduction to SQL databases')").unwrap();
    engine.execute("INSERT INTO articles VALUES (3, 'Web Development', 'Building web apps with Rust')").unwrap();

    // Search for 'rust' - should return rows 1 and 3
    let result = engine.execute("SELECT id, title FROM articles WHERE MATCH(title, body) AGAINST('rust')");
    assert!(result.is_ok(), "FTS query failed: {:?}", result);

    let result = result.unwrap();
    // Should find 2 matching documents (id=1 and id=3)
    assert_eq!(result.rows.len(), 2, "Expected 2 matching rows, got {}", result.rows.len());

    // Verify the IDs of returned rows
    let ids: Vec<i64> = result.rows.iter()
        .filter_map(|row| {
            if let sqlrustgo_types::Value::Integer(n) = &row[0] {
                Some(*n)
            } else {
                None
            }
        })
        .collect();
    assert!(ids.contains(&1), "Should contain row with id=1");
    assert!(ids.contains(&3), "Should contain row with id=3");
    assert!(!ids.contains(&2), "Should NOT contain row with id=2");
}

#[test]
fn test_fts_case_insensitive_search() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE docs (id INTEGER, content TEXT)").unwrap();
    engine.execute("INSERT INTO docs VALUES (1, 'Hello World')").unwrap();
    engine.execute("INSERT INTO docs VALUES (2, 'HELLO WORLD')").unwrap();
    engine.execute("INSERT INTO docs VALUES (3, 'hello world')").unwrap();

    // Search should be case-insensitive
    let result = engine.execute("SELECT id FROM docs WHERE MATCH(content) AGAINST('hello')");
    assert!(result.is_ok(), "FTS query failed: {:?}", result);

    let result = result.unwrap();
    assert_eq!(result.rows.len(), 3, "Case-insensitive search should find all 3 rows");
}

#[test]
fn test_fts_empty_search_term() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE test (id INTEGER, content TEXT)").unwrap();
    engine.execute("INSERT INTO test VALUES (1, 'Hello World')").unwrap();

    // Empty search term should return no results
    let result = engine.execute("SELECT * FROM test WHERE MATCH(content) AGAINST('')");
    assert!(result.is_ok(), "FTS query should handle empty term");
}

#[test]
fn test_fts_dml_auto_sync() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE articles (id INTEGER, title TEXT, body TEXT)").unwrap();

    // INSERT: new documents should be findable
    engine.execute("INSERT INTO articles VALUES (1, 'Rust Programming', 'Learn Rust')").unwrap();
    engine.execute("INSERT INTO articles VALUES (2, 'Database Basics', 'Introduction to SQL')").unwrap();

    // Search for 'rust' - should find row 1
    let result = engine.execute("SELECT id FROM articles WHERE MATCH(title, body) AGAINST('rust')").unwrap();
    assert_eq!(result.rows.len(), 1, "Should find 1 row with 'rust'");
    assert_eq!(result.rows[0][0], sqlrustgo_types::Value::Integer(1));

    // UPDATE: modified documents should reflect in search
    engine.execute("UPDATE articles SET title = 'Rust Deep Dive' WHERE id = 1").unwrap();

    // Row 1 should still be findable by new title
    let result = engine.execute("SELECT id FROM articles WHERE MATCH(title, body) AGAINST('deep')").unwrap();
    assert_eq!(result.rows.len(), 1, "Should find 1 row with 'deep' after UPDATE");

    // Original 'rust' search should still work (body contains 'Rust')
    let result = engine.execute("SELECT id FROM articles WHERE MATCH(title, body) AGAINST('rust')").unwrap();
    assert_eq!(result.rows.len(), 1, "Should still find row 1 by 'rust' in body");

    // DELETE: removed documents should not be findable
    engine.execute("DELETE FROM articles WHERE id = 1").unwrap();

    let result = engine.execute("SELECT id FROM articles WHERE MATCH(title, body) AGAINST('rust')").unwrap();
    assert_eq!(result.rows.len(), 0, "Deleted row should not be found");
}

#[test]
fn test_fts_select_list_expression() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE articles (id INTEGER, title TEXT, body TEXT)").unwrap();
    engine.execute("INSERT INTO articles VALUES (1, 'Rust Programming', 'Learn Rust programming language')").unwrap();
    engine.execute("INSERT INTO articles VALUES (2, 'Database Basics', 'Introduction to SQL databases')").unwrap();
    engine.execute("INSERT INTO articles VALUES (3, 'Web Development', 'Building web apps with Rust')").unwrap();

    // SELECT list FTS expression with alias
    let result = engine.execute("SELECT id, MATCH(title, body) AGAINST('rust') AS relevance FROM articles").unwrap();
    assert_eq!(result.rows.len(), 3, "Should return all 3 rows");

    // Rows containing 'rust' should have relevance = true (1)
    let rust_relevance: Vec<bool> = result.rows.iter()
        .map(|row| {
            if let sqlrustgo_types::Value::Boolean(b) = &row[1] {
                *b
            } else {
                false
            }
        })
        .collect();
    assert_eq!(rust_relevance, vec![true, false, true], "id=1 and id=3 should match 'rust'");

    // Test FTS in SELECT with WHERE filter
    let result = engine.execute("SELECT id, MATCH(title) AGAINST('database') AS title_match FROM articles WHERE id > 1").unwrap();
    assert_eq!(result.rows.len(), 2, "Should return 2 rows");
    let title_match: Vec<bool> = result.rows.iter()
        .map(|row| {
            if let sqlrustgo_types::Value::Boolean(b) = &row[1] {
                *b
            } else {
                false
            }
        })
        .collect();
    assert_eq!(title_match, vec![true, false], "id=2 title_match=true, id=3 title_match=false");
}