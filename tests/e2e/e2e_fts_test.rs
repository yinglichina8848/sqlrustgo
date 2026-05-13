use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_fts_basic_match_against() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE articles (id INTEGER, title TEXT, body TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO articles VALUES (1, 'Hello World', 'This is a test')")
        .unwrap();
    engine
        .execute("INSERT INTO articles VALUES (2, 'Rust Programming', 'Learn Rust')")
        .unwrap();
    engine
        .execute("INSERT INTO articles VALUES (3, 'Database Systems', 'Intro to DB')")
        .unwrap();

    let result = engine.execute("SELECT * FROM articles WHERE MATCH(title, body) AGAINST('rust')");
    assert!(result.is_ok(), "FTS query should parse: {:?}", result);
}

#[test]
fn test_fts_in_where_clause() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE docs (id INTEGER, content TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO docs VALUES (1, 'Hello world')")
        .unwrap();
    engine
        .execute("INSERT INTO docs VALUES (2, 'Goodbye world')")
        .unwrap();

    let result =
        engine.execute("SELECT id, content FROM docs WHERE MATCH(content) AGAINST('hello')");
    assert!(result.is_ok(), "FTS in WHERE should work: {:?}", result);
}

#[test]
fn test_fts_multiple_columns() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE posts (id INTEGER, title TEXT, content TEXT, author TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO posts VALUES (1, 'Rust Tips', 'Useful Rust tips', 'alice')")
        .unwrap();
    engine
        .execute("INSERT INTO posts VALUES (2, 'Go Tips', 'Useful Go tips', 'bob')")
        .unwrap();

    let result = engine.execute("SELECT * FROM posts WHERE MATCH(title, content) AGAINST('rust')");
    assert!(
        result.is_ok(),
        "FTS across multiple columns should work: {:?}",
        result
    );
}
