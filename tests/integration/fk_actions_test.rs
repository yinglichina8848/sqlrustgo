// Foreign Key DELETE/UPDATE Actions Tests (Issue #888)

use sqlrustgo::{parse, ExecutionEngine, MemoryStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

#[test]
fn test_fk_cascade_parsing() {
    // Test ON DELETE CASCADE parsing
    let result = parse("CREATE TABLE child (id INTEGER, parent_id INTEGER REFERENCES parent(id) ON DELETE CASCADE)");
    assert!(
        result.is_ok(),
        "ON DELETE CASCADE should parse: {:?}",
        result.err()
    );

    // Test ON UPDATE CASCADE
    let result = parse("CREATE TABLE child2 (id INTEGER, parent_id INTEGER REFERENCES parent(id) ON UPDATE CASCADE)");
    assert!(result.is_ok(), "ON UPDATE CASCADE should parse");

    println!("✓ FK CASCADE parsing works");
}

#[test]
fn test_fk_set_null_parsing() {
    // Test ON DELETE SET NULL
    let result = parse("CREATE TABLE child (id INTEGER, parent_id INTEGER REFERENCES parent(id) ON DELETE SET NULL)");
    assert!(result.is_ok(), "ON DELETE SET NULL should parse");

    // Test ON UPDATE SET NULL
    let result = parse("CREATE TABLE child2 (id INTEGER, parent_id INTEGER REFERENCES parent(id) ON UPDATE SET NULL)");
    assert!(result.is_ok(), "ON UPDATE SET NULL should parse");

    println!("✓ FK SET NULL parsing works");
}

#[test]
fn test_fk_restrict_parsing() {
    // Test ON DELETE RESTRICT
    let result = parse("CREATE TABLE child (id INTEGER, parent_id INTEGER REFERENCES parent(id) ON DELETE RESTRICT)");
    assert!(result.is_ok(), "ON DELETE RESTRICT should parse");

    // Test ON UPDATE RESTRICT
    let result = parse("CREATE TABLE child2 (id INTEGER, parent_id INTEGER REFERENCES parent(id) ON UPDATE RESTRICT)");
    assert!(result.is_ok(), "ON UPDATE RESTRICT should parse");

    println!("✓ FK RESTRICT parsing works");
}

#[test]
fn test_fk_no_action_parsing() {
    // Test ON DELETE NO ACTION
    let result = parse("CREATE TABLE child (id INTEGER, parent_id INTEGER REFERENCES parent(id) ON DELETE NO ACTION)");
    assert!(result.is_ok(), "ON DELETE NO ACTION should parse");

    println!("✓ FK NO ACTION parsing works");
}

#[test]
fn test_fk_combined_actions_parsing() {
    // Test combined FK actions
    let result = parse(
        "CREATE TABLE child (
        id INTEGER PRIMARY KEY,
        parent_id INTEGER REFERENCES parent(id) ON DELETE CASCADE ON UPDATE SET NULL
    )",
    );
    assert!(result.is_ok(), "Combined FK actions should parse");

    println!("✓ Combined FK actions parsing works");
}
