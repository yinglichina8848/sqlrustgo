//! L4 E2E Test: Full idempotency flow from SQL to committed result

#[cfg(test)]
mod tests {
    use sqlrustgo::{ExecutionEngine, Value};
    use std::sync::{Arc, RwLock};

    fn create_engine() -> ExecutionEngine<sqlrustgo_storage::MemoryStorage> {
        let storage = Arc::new(RwLock::new(sqlrustgo_storage::MemoryStorage::new()));
        ExecutionEngine::new(storage)
    }

    #[test]
    fn test_mobile_offline_sync_scenario() {
        let mut engine = create_engine();

        engine
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, synced INTEGER)")
            .unwrap();
        engine
            .execute("INSERT INTO users (id, name, synced) VALUES (1, 'Alice', 0)")
            .unwrap();

        let req_uuid = "req-uuid-001";

        let begin_result = engine
            .execute(&format!("BEGIN IDEMPOTENT '{}'", req_uuid))
            .unwrap();
        assert_eq!(begin_result.affected_rows, 1);

        engine
            .execute("UPDATE users SET synced = 1, name = 'Alice' WHERE id = 1")
            .unwrap();

        let commit_result = engine.execute("COMMIT").unwrap();
        assert!(
            commit_result.is_ok(),
            "First sync should commit successfully"
        );

        let select_result = engine
            .execute("SELECT id, name, synced FROM users WHERE id = 1")
            .unwrap();
        assert_eq!(select_result.rows[0][1], Value::Text("Alice".to_string()));
        assert_eq!(select_result.rows[0][2], Value::Integer(1));

        let retry_result = engine
            .execute(&format!("BEGIN IDEMPOTENT '{}'", req_uuid))
            .unwrap();
        assert_eq!(retry_result.affected_rows, 0);
        assert_eq!(retry_result.rows[0][0], Value::Text(req_uuid.to_string()));

        let retry_select = engine
            .execute("SELECT id, name, synced FROM users WHERE id = 1")
            .unwrap();
        assert_eq!(retry_select.rows[0][2], Value::Integer(1));
    }

    #[test]
    fn test_mobile_offline_sync_scenario_with_keyword_form() {
        let mut engine = create_engine();

        engine
            .execute("CREATE TABLE orders (id INTEGER PRIMARY KEY, amount INTEGER, status TEXT)")
            .unwrap();
        engine
            .execute("INSERT INTO orders (id, amount, status) VALUES (1, 100, 'pending')")
            .unwrap();

        let idempotency_key = "order-update-uuid-002";

        let begin_result = engine
            .execute(&format!("BEGIN IDEMPOTENCY KEY '{}'", idempotency_key))
            .unwrap();
        assert_eq!(begin_result.affected_rows, 1);

        engine
            .execute("UPDATE orders SET status = 'confirmed' WHERE id = 1")
            .unwrap();
        engine.execute("COMMIT").unwrap();

        let retry_begin = engine
            .execute(&format!("BEGIN IDEMPOTENCY KEY '{}'", idempotency_key))
            .unwrap();
        assert_eq!(retry_begin.affected_rows, 0);
        assert_eq!(
            retry_begin.rows[0][0],
            Value::Text(idempotency_key.to_string())
        );
    }

    #[test]
    fn test_concurrent_idempotent_requests_same_key() {
        let mut engine = create_engine();

        engine
            .execute("CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance INTEGER)")
            .unwrap();
        engine
            .execute("INSERT INTO accounts (id, balance) VALUES (1, 1000)")
            .unwrap();

        let key = "concurrent-transfer-001";

        let result1 = engine
            .execute(&format!("BEGIN IDEMPOTENT '{}'", key))
            .unwrap();
        assert_eq!(result1.affected_rows, 1);

        let _ = engine.execute("BEGIN IDEMPOTENT 'other-key'");
        let _ = engine.execute("COMMIT");

        let result2 = engine
            .execute(&format!("BEGIN IDEMPOTENT '{}'", key))
            .unwrap();
        assert_eq!(result2.affected_rows, 0);
        assert_eq!(result2.rows[0][0], Value::Text(key.to_string()));
    }

    #[test]
    fn test_multiple_idempotent_keys_different_transactions() {
        let mut engine = create_engine();

        engine
            .execute(
                "CREATE TABLE events (id INTEGER PRIMARY KEY, event_type TEXT, processed INTEGER)",
            )
            .unwrap();

        let keys = vec!["evt-001", "evt-002", "evt-003"];

        for key in &keys {
            let begin_result = engine
                .execute(&format!("BEGIN IDEMPOTENT '{}'", key))
                .unwrap();
            assert_eq!(begin_result.affected_rows, 1);
            engine
                .execute(&format!(
                    "INSERT INTO events (event_type, processed) VALUES ('{}', 1)",
                    key
                ))
                .unwrap();
            engine.execute("COMMIT").unwrap();
        }

        for key in &keys {
            let replay_result = engine
                .execute(&format!("BEGIN IDEMPOTENT '{}'", key))
                .unwrap();
            assert_eq!(replay_result.affected_rows, 0);
            assert_eq!(replay_result.rows[0][0], Value::Text(key.to_string()));
        }

        let count_result = engine.execute("SELECT COUNT(*) FROM events").unwrap();
        assert_eq!(count_result.rows[0][0], Value::Integer(3));
    }

    #[test]
    fn test_idempotent_rollback_and_replay() {
        let mut engine = create_engine();

        engine
            .execute("CREATE TABLE inventory (id INTEGER PRIMARY KEY, stock INTEGER)")
            .unwrap();
        engine
            .execute("INSERT INTO inventory (id, stock) VALUES (1, 50)")
            .unwrap();

        let key = "inventory-update-001";

        let begin_result = engine
            .execute(&format!("BEGIN IDEMPOTENT '{}'", key))
            .unwrap();
        assert_eq!(begin_result.affected_rows, 1);

        engine
            .execute("UPDATE inventory SET stock = 45 WHERE id = 1")
            .unwrap();

        let rollback_result = engine.execute("ROLLBACK").unwrap();
        assert!(rollback_result.is_ok(), "ROLLBACK should succeed");

        let replay_result = engine
            .execute(&format!("BEGIN IDEMPOTENT '{}'", key))
            .unwrap();
        assert_eq!(replay_result.affected_rows, 1);

        engine
            .execute("UPDATE inventory SET stock = 40 WHERE id = 1")
            .unwrap();
        engine.execute("COMMIT").unwrap();

        let select_result = engine
            .execute("SELECT stock FROM inventory WHERE id = 1")
            .unwrap();
        assert_eq!(select_result.rows[0][0], Value::Integer(40));
    }

    #[test]
    fn test_gmp_audit_trail_with_idempotency() {
        let mut engine = create_engine();

        engine.execute("CREATE TABLE audit_log (id INTEGER PRIMARY KEY, action TEXT, idempotency_key TEXT, timestamp INTEGER)").unwrap();

        let actions = vec![
            ("audit-action-001", "CREATE_USER"),
            ("audit-action-002", "UPDATE_PERMISSION"),
            ("audit-action-003", "DELETE_RECORD"),
        ];

        for (key, action) in &actions {
            let begin_result = engine
                .execute(&format!("BEGIN IDEMPOTENT '{}'", key))
                .unwrap();
            assert_eq!(begin_result.affected_rows, 1);

            engine.execute(&format!(
                "INSERT INTO audit_log (action, idempotency_key, timestamp) VALUES ('{}', '{}', 1234567890)",
                action, key
            )).unwrap();

            engine.execute("COMMIT").unwrap();
        }

        for (key, action) in &actions {
            let replay_result = engine
                .execute(&format!("BEGIN IDEMPOTENT '{}'", key))
                .unwrap();
            assert_eq!(replay_result.affected_rows, 0);
            assert_eq!(replay_result.rows[0][0], Value::Text(key.to_string()));
        }

        let count_result = engine.execute("SELECT COUNT(*) FROM audit_log").unwrap();
        assert_eq!(count_result.rows[0][0], Value::Integer(3));
    }
}
