use sqlrustgo::execution_engine::EngineConfig;
use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_fresh_engine() -> MemoryExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    MemoryExecutionEngine::new_with_config(storage, EngineConfig::default())
}

#[test]
fn test_execution_chain_insert_select() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE src (id INTEGER, name TEXT)");
    let _ = engine.execute("INSERT INTO src VALUES (1, 'Alice')");
    let _ = engine.execute("INSERT INTO src VALUES (2, 'Bob')");

    let result = engine.execute("CREATE TABLE dst (id INTEGER, name TEXT)");
    assert!(result.is_ok());

    let result = engine.execute("INSERT INTO dst SELECT * FROM src");
    assert!(result.is_ok());

    let result = engine.execute("SELECT COUNT(*) FROM dst");
    assert!(result.is_ok());
}

#[test]
fn test_execution_chain_savepoint_release() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t (id INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1)");

    let _ = engine.execute("SAVEPOINT sp1");

    let _ = engine.execute("INSERT INTO t VALUES (2)");
    let _ = engine.execute("INSERT INTO t VALUES (3)");

    let result = engine.execute("RELEASE SAVEPOINT sp1");
    assert!(result.is_ok());

    let count_result = engine.execute("SELECT COUNT(*) FROM t");
    assert!(count_result.is_ok());
}

#[test]
fn test_execution_chain_rollback_to_savepoint() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t (id INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1)");

    let _ = engine.execute("SAVEPOINT sp1");
    let _ = engine.execute("INSERT INTO t VALUES (2)");
    let _ = engine.execute("INSERT INTO t VALUES (3)");

    let result = engine.execute("ROLLBACK TO SAVEPOINT sp1");
    assert!(result.is_ok());

    let count_result = engine.execute("SELECT COUNT(*) FROM t");
    assert!(count_result.is_ok());
}

#[test]
fn test_execution_chain_set_transaction() {
    let mut engine = create_fresh_engine();

    let result = engine.execute("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE");
    assert!(result.is_ok());
}

#[test]
fn test_execution_chain_intersect() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t1 (id INTEGER)");
    let _ = engine.execute("CREATE TABLE t2 (id INTEGER)");
    let _ = engine.execute("INSERT INTO t1 VALUES (1), (2), (3)");
    let _ = engine.execute("INSERT INTO t2 VALUES (2), (3), (4)");

    let result = engine.execute("SELECT id FROM t1 INTERSECT SELECT id FROM t2");
    assert!(result.is_ok());
}

#[test]
fn test_execution_chain_except() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t1 (id INTEGER)");
    let _ = engine.execute("CREATE TABLE t2 (id INTEGER)");
    let _ = engine.execute("INSERT INTO t1 VALUES (1), (2), (3)");
    let _ = engine.execute("INSERT INTO t2 VALUES (2), (3), (4)");

    let result = engine.execute("SELECT id FROM t1 EXCEPT SELECT id FROM t2");
    assert!(result.is_ok());
}

#[ignore]
#[test]
fn test_execution_chain_grant_revoke() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE ROLE test_role");
    let _ = engine.execute("CREATE TABLE t (id INTEGER)");

    let result = engine.execute("GRANT SELECT ON t TO test_role");
    assert!(result.is_ok());

    let revoke_result = engine.execute("REVOKE SELECT ON t FROM test_role");
    assert!(revoke_result.is_ok());
}

#[ignore]
#[test]
fn test_execution_chain_multi_table_update() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t1 (id INTEGER, val INTEGER)");
    let _ = engine.execute("CREATE TABLE t2 (id INTEGER, val INTEGER)");
    let _ = engine.execute("INSERT INTO t1 VALUES (1, 100)");
    let _ = engine.execute("INSERT INTO t2 VALUES (1, 200)");

    let result = engine.execute("UPDATE t1, t2 SET t1.val = t2.val WHERE t1.id = t2.id");
    assert!(result.is_ok());
}

#[ignore]
#[test]
fn test_execution_chain_multi_table_delete() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t1 (id INTEGER, val INTEGER)");
    let _ = engine.execute("CREATE TABLE t2 (id INTEGER, val INTEGER)");
    let _ = engine.execute("INSERT INTO t1 VALUES (1, 100)");
    let _ = engine.execute("INSERT INTO t1 VALUES (2, 200)");
    let _ = engine.execute("INSERT INTO t2 VALUES (1, 200)");

    let result = engine.execute("DELETE t1 FROM t1 INNER JOIN t2 ON t1.id = t2.id");
    assert!(result.is_ok());
}

#[ignore]
#[test]
fn test_execution_chain_truncate() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t (id INTEGER, name TEXT)");
    let _ = engine.execute("INSERT INTO t VALUES (1, 'Alice')");
    let _ = engine.execute("INSERT INTO t VALUES (2, 'Bob')");

    let result = engine.execute("TRUNCATE TABLE t");
    assert!(result.is_ok());

    let count_result = engine.execute("SELECT COUNT(*) FROM t");
    assert!(count_result.is_ok());
}

#[ignore]
#[test]
fn test_execution_chain_rename_table() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE old_name (id INTEGER)");
    let _ = engine.execute("INSERT INTO old_name VALUES (1)");

    let result = engine.execute("RENAME TABLE old_name TO new_name");
    assert!(result.is_ok());

    let check = engine.execute("SELECT * FROM new_name");
    assert!(check.is_ok());
}

#[ignore]
#[test]
fn test_execution_chain_scalar_subquery() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t1 (id INTEGER, val INTEGER)");
    let _ = engine.execute("CREATE TABLE t2 (id INTEGER, val INTEGER)");
    let _ = engine.execute("INSERT INTO t1 VALUES (1, 100)");
    let _ = engine.execute("INSERT INTO t2 VALUES (1, 50)");

    let result = engine
        .execute("SELECT t1.id, (SELECT MAX(val) FROM t2 WHERE t2.id = t1.id) as max_val FROM t1");
    assert!(result.is_ok());
}

#[ignore]
#[test]
fn test_execution_chain_cte() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE sales (product TEXT, amount INTEGER)");
    let _ = engine.execute("INSERT INTO sales VALUES ('A', 100)");
    let _ = engine.execute("INSERT INTO sales VALUES ('B', 200)");

    let result =
        engine.execute("WITH total AS (SELECT SUM(amount) as s FROM sales) SELECT * FROM total");
    assert!(result.is_ok());
}

#[ignore]
#[test]
fn test_execution_chain_create_drop_view() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE TABLE t (id INTEGER, name TEXT)");
    let _ = engine.execute("INSERT INTO t VALUES (1, 'Alice')");

    let result = engine.execute("CREATE VIEW v AS SELECT * FROM t WHERE id = 1");
    assert!(result.is_ok());

    let view_result = engine.execute("SELECT * FROM v");
    assert!(view_result.is_ok());

    let drop_result = engine.execute("DROP VIEW v");
    assert!(drop_result.is_ok());
}

#[ignore]
#[test]
fn test_execution_chain_create_drop_database() {
    let mut engine = create_fresh_engine();

    let result = engine.execute("CREATE DATABASE test_db");
    assert!(result.is_ok());

    let drop_result = engine.execute("DROP DATABASE test_db");
    assert!(drop_result.is_ok());
}

#[ignore]
#[test]
fn test_execution_chain_create_drop_role() {
    let mut engine = create_fresh_engine();

    let result = engine.execute("CREATE ROLE test_role");
    assert!(result.is_ok());

    let drop_result = engine.execute("DROP ROLE test_role");
    assert!(drop_result.is_ok());
}

#[ignore]
#[test]
fn test_execution_chain_alter_user() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE USER test_user WITH PASSWORD 'pass'");

    let result = engine.execute("ALTER USER test_user RENAME TO test_user_renamed");
    assert!(result.is_ok());

    let _ = engine.execute("DROP USER test_user_renamed");
}

#[ignore]
#[test]
fn test_execution_chain_set_password() {
    let mut engine = create_fresh_engine();

    let _ = engine.execute("CREATE USER test_user WITH PASSWORD 'old_pass'");

    let result = engine.execute("SET PASSWORD FOR test_user = 'new_pass'");
    assert!(result.is_ok());

    let _ = engine.execute("DROP USER test_user");
}
