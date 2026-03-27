use sqlrustgo_executor::session_config::SessionConfig;

#[test]
fn test_default_not_benchmark_mode() {
    std::env::remove_var("SQLRUSTGO_BENCHMARK_MODE");
    let config = SessionConfig::default();
    assert!(!config.benchmark_mode);
    assert!(config.cache_enabled);
    assert!(config.stats_enabled);
}

#[test]
fn test_benchmark_mode_from_env() {
    std::env::set_var("SQLRUSTGO_BENCHMARK_MODE", "1");
    let config = SessionConfig::default();
    assert!(config.benchmark_mode);
    assert!(!config.cache_enabled);
    assert!(!config.stats_enabled);
    std::env::remove_var("SQLRUSTGO_BENCHMARK_MODE");
}

#[test]
fn test_explicit_benchmark_mode() {
    let config = SessionConfig::new(true);
    assert!(config.benchmark_mode);
    assert!(!config.cache_enabled);
    assert!(!config.stats_enabled);
}

#[test]
fn test_explicit_normal_mode() {
    let config = SessionConfig::new(false);
    assert!(!config.benchmark_mode);
    assert!(config.cache_enabled);
    assert!(config.stats_enabled);
}

// ============================================================================
// SAVEPOINT Tests (Issue #892)
// ============================================================================

#[test]
fn test_savepoint_basic() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create table
    engine.execute(parse("CREATE TABLE savepoint_test (id INTEGER, value TEXT)").unwrap()).unwrap();

    // Begin transaction
    engine.execute(parse("BEGIN").unwrap()).unwrap();

    // Insert first row
    engine.execute(parse("INSERT INTO savepoint_test VALUES (1, 'before_savepoint')").unwrap()).unwrap();

    // Create savepoint
    engine.execute(parse("SAVEPOINT sp1").unwrap()).unwrap();

    // Insert second row after savepoint
    engine.execute(parse("INSERT INTO savepoint_test VALUES (2, 'after_savepoint')").unwrap()).unwrap();

    // Verify both rows exist before rollback
    let result = engine.execute(parse("SELECT COUNT(*) FROM savepoint_test").unwrap()).unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(2));

    // Rollback to savepoint
    engine.execute(parse("ROLLBACK TO SAVEPOINT sp1").unwrap()).unwrap();

    // Should still have 1 row (the one before savepoint)
    let result = engine.execute(parse("SELECT COUNT(*) FROM savepoint_test").unwrap()).unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(1));

    // Commit
    engine.execute(parse("COMMIT").unwrap()).unwrap();

    println!("✓ SAVEPOINT basic functionality works");
}

#[test]
fn test_savepoint_rollback_release() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine.execute(parse("CREATE TABLE sp_test (id INTEGER)").unwrap()).unwrap();

    // Begin
    engine.execute(parse("BEGIN").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO sp_test VALUES (1)").unwrap()).unwrap();

    // Savepoint 1
    engine.execute(parse("SAVEPOINT sp1").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO sp_test VALUES (2)").unwrap()).unwrap();

    // Savepoint 2
    engine.execute(parse("SAVEPOINT sp2").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO sp_test VALUES (3)").unwrap()).unwrap();

    // Rollback to sp1 (should remove rows 2 and 3)
    engine.execute(parse("ROLLBACK TO SAVEPOINT sp1").unwrap()).unwrap();

    // Should have 1 row (id=1)
    let result = engine.execute(parse("SELECT COUNT(*) FROM sp_test").unwrap()).unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(1));

    // Release savepoint sp1
    engine.execute(parse("RELEASE SAVEPOINT sp1").unwrap()).unwrap();

    // Commit
    engine.execute(parse("COMMIT").unwrap()).unwrap();

    println!("✓ SAVEPOINT rollback and release work");
}

#[test]
fn test_savepoint_nested() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine.execute(parse("CREATE TABLE nested_sp (id INTEGER)").unwrap()).unwrap();

    // Test nested savepoints
    engine.execute(parse("BEGIN").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO nested_sp VALUES (1)").unwrap()).unwrap();
    engine.execute(parse("SAVEPOINT level1").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO nested_sp VALUES (2)").unwrap()).unwrap();
    engine.execute(parse("SAVEPOINT level2").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO nested_sp VALUES (3)").unwrap()).unwrap();

    // Rollback to level1
    engine.execute(parse("ROLLBACK TO SAVEPOINT level1").unwrap()).unwrap();

    // Should have 1 row
    let result = engine.execute(parse("SELECT COUNT(*) FROM nested_sp").unwrap()).unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(1));

    engine.execute(parse("COMMIT").unwrap()).unwrap();

    println!("✓ Nested SAVEPOINT works");
}
