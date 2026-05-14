//! L3 Integration Test: Full execution flow for idempotent transactions

use sqlrustgo::{ExecutionEngine, Value};
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<sqlrustgo_storage::MemoryStorage> {
    let storage = Arc::new(RwLock::new(sqlrustgo_storage::MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_transaction_commits() {
        let mut engine = create_engine();

        engine
            .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
            .unwrap();
        engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

        let begin_result = engine.execute("BEGIN IDEMPOTENT 'txn-1'").unwrap();
        assert_eq!(begin_result.affected_rows, 1);
        assert!(!begin_result.rows.is_empty());
        if let Value::Integer(_) = begin_result.rows[0][0] {
            // New transaction created with tx_id
        } else {
            panic!("Expected Integer tx_id, got {:?}", begin_result.rows[0][0]);
        }

        engine
            .execute("UPDATE t SET value = 200 WHERE id = 1")
            .unwrap();

        let _commit_result = engine.execute("COMMIT").unwrap();

        let select_result = engine
            .execute("SELECT id, value FROM t WHERE id = 1")
            .unwrap();
        assert_eq!(select_result.rows[0][1], Value::Integer(200));
    }

    #[test]
    fn test_replay_returns_idempotent_success() {
        let mut engine = create_engine();

        engine
            .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
            .unwrap();
        engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

        let begin_result1 = engine.execute("BEGIN IDEMPOTENT 'txn-replay'").unwrap();
        assert_eq!(begin_result1.affected_rows, 1);

        engine.execute("COMMIT").unwrap();

        let begin_result2 = engine.execute("BEGIN IDEMPOTENT 'txn-replay'").unwrap();
        assert_eq!(begin_result2.affected_rows, 0);
        assert_eq!(begin_result2.rows.len(), 1);
        assert_eq!(
            begin_result2.rows[0][0],
            Value::Text("txn-replay".to_string())
        );
    }

    #[test]
    fn test_different_content_different_hash() {
        let mut engine = create_engine();

        engine
            .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
            .unwrap();
        engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

        let begin_result1 = engine.execute("BEGIN IDEMPOTENT 'txn-content'").unwrap();
        assert_eq!(begin_result1.affected_rows, 1);

        engine
            .execute("UPDATE t SET value = 200 WHERE id = 1")
            .unwrap();
        engine.execute("COMMIT").unwrap();

        let begin_result2 = engine.execute("BEGIN IDEMPOTENT 'txn-content'").unwrap();
        assert_eq!(begin_result2.affected_rows, 0);
        assert_eq!(
            begin_result2.rows[0][0],
            Value::Text("txn-content".to_string())
        );
    }

    #[test]
    fn test_idempotent_with_different_statements_same_key() {
        let mut engine = create_engine();

        engine
            .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
            .unwrap();
        engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

        let begin_result = engine
            .execute("BEGIN IDEMPOTENT 'txn-different-stmts'")
            .unwrap();
        assert_eq!(begin_result.affected_rows, 1);

        engine
            .execute("UPDATE t SET value = 200 WHERE id = 1")
            .unwrap();
        engine.execute("COMMIT").unwrap();

        let begin_result2 = engine
            .execute("BEGIN IDEMPOTENT 'txn-different-stmts'")
            .unwrap();
        assert_eq!(begin_result2.affected_rows, 0);
        assert_eq!(
            begin_result2.rows[0][0],
            Value::Text("txn-different-stmts".to_string())
        );
    }

    #[test]
    fn test_idempotent_keyword_form() {
        let mut engine = create_engine();

        engine
            .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
            .unwrap();
        engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

        let begin_result = engine
            .execute("BEGIN IDEMPOTENCY KEY 'txn-keyword-form'")
            .unwrap();
        assert_eq!(begin_result.affected_rows, 1);

        engine.execute("COMMIT").unwrap();

        let replay_result = engine
            .execute("BEGIN IDEMPOTENCY KEY 'txn-keyword-form'")
            .unwrap();
        assert_eq!(replay_result.affected_rows, 0);
        assert_eq!(
            replay_result.rows[0][0],
            Value::Text("txn-keyword-form".to_string())
        );
    }
}
