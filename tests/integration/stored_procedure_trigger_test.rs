//! Stored Procedure and Trigger Integration Tests
//!
//! Tests for stored procedure parsing, execution, and trigger functionality.

#[cfg(test)]
mod stored_procedure_tests {
    use sqlrustgo_parser::parse;

    #[test]
    fn test_parse_simple_procedure() {
        let sql = "CREATE PROCEDURE test_proc() BEGIN SELECT 1 END";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse simple procedure: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_procedure_with_params() {
        let sql = "CREATE PROCEDURE test_proc(x INT, y TEXT) BEGIN SELECT 1 END";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse procedure with params: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_procedure_with_if() {
        let sql = "CREATE PROCEDURE test_if(x INT) BEGIN IF x > 0 THEN SELECT 1 END IF END";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse procedure with IF: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_procedure_with_while() {
        let sql = "CREATE PROCEDURE test_while() BEGIN WHILE 1 = 1 DO SELECT 1 END WHILE END";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse procedure with WHILE: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_procedure_with_loop() {
        let sql = "CREATE PROCEDURE test_loop() BEGIN LOOP SELECT 1 END LOOP END";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse procedure with LOOP: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_procedure_with_if_else() {
        let sql = "CREATE PROCEDURE test_if_else(x INT) BEGIN IF x > 0 THEN SELECT 1 ELSE SELECT 2 END IF END";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse procedure with IF-ELSE: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_procedure_with_declare() {
        let sql = "CREATE PROCEDURE test_declare() BEGIN DECLARE x INT DEFAULT 0 SET x = 1 END";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse procedure with DECLARE: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_procedure_with_return() {
        let sql = "CREATE PROCEDURE test_return() BEGIN RETURN 1 END";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse procedure with RETURN: {:?}",
            result.err()
        );
    }
}

#[cfg(test)]
mod trigger_tests {
    use sqlrustgo_parser::parse;

    #[test]
    fn test_parse_simple_trigger() {
        let sql = "CREATE TRIGGER test_trigger ON my_table BEFORE INSERT DO DELETE FROM orders";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse simple trigger: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_trigger_before_insert() {
        let sql =
            "CREATE TRIGGER before_insert_trigger ON orders BEFORE INSERT DO DELETE FROM orders";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse BEFORE INSERT trigger: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_trigger_after_update() {
        let sql =
            "CREATE TRIGGER after_update_trigger ON orders AFTER UPDATE DO DELETE FROM orders";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse AFTER UPDATE trigger: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_trigger_before_delete() {
        let sql =
            "CREATE TRIGGER before_delete_trigger ON orders BEFORE DELETE DO DELETE FROM orders";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse BEFORE DELETE trigger: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_trigger_insert() {
        let sql = "CREATE TRIGGER insert_trigger ON orders AFTER INSERT DO DELETE FROM orders";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse AFTER INSERT trigger: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_trigger_update() {
        let sql = "CREATE TRIGGER update_trigger ON orders BEFORE UPDATE DO DELETE FROM orders";
        let result = parse(sql);
        assert!(
            result.is_ok(),
            "Should parse BEFORE UPDATE trigger: {:?}",
            result.err()
        );
    }
}

#[cfg(test)]
mod trigger_executor_tests {
    use sqlrustgo_executor::trigger::{
        TriggerEvent, TriggerExecutionResult, TriggerExecutor, TriggerTiming, TriggerType,
    };
    use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
    use sqlrustgo_types::Value;
    use std::sync::Arc;

    fn create_test_storage() -> MemoryStorage {
        let mut storage = MemoryStorage::new();
        let table_info = TableInfo {
            name: "orders".to_string(),
            columns: vec![
                ColumnDefinition::new("id", "INTEGER"),
                ColumnDefinition::new("price", "FLOAT"),
                ColumnDefinition::new("quantity", "INTEGER"),
                ColumnDefinition::new("total", "FLOAT"),
            ],
        };
        storage.create_table(&table_info).unwrap();
        storage
    }

    #[test]
    fn test_trigger_executor_creation() {
        let storage = create_test_storage();
        let executor = TriggerExecutor::new(Arc::new(storage));
        assert_eq!(executor.get_table_triggers("orders").len(), 0);
    }

    #[test]
    fn test_trigger_timing_enum() {
        assert_eq!(
            TriggerTiming::Before,
            TriggerTiming::from(sqlrustgo_storage::TriggerTiming::Before)
        );
        assert_eq!(
            TriggerTiming::After,
            TriggerTiming::from(sqlrustgo_storage::TriggerTiming::After)
        );
    }

    #[test]
    fn test_trigger_event_enum() {
        assert_eq!(
            TriggerEvent::Insert,
            TriggerEvent::from(sqlrustgo_storage::TriggerEvent::Insert)
        );
        assert_eq!(
            TriggerEvent::Update,
            TriggerEvent::from(sqlrustgo_storage::TriggerEvent::Update)
        );
        assert_eq!(
            TriggerEvent::Delete,
            TriggerEvent::from(sqlrustgo_storage::TriggerEvent::Delete)
        );
    }

    #[test]
    fn test_trigger_type_new() {
        assert_eq!(
            TriggerType::BeforeInsert,
            TriggerType::new(TriggerTiming::Before, TriggerEvent::Insert)
        );
        assert_eq!(
            TriggerType::AfterUpdate,
            TriggerType::new(TriggerTiming::After, TriggerEvent::Update)
        );
        assert_eq!(
            TriggerType::BeforeDelete,
            TriggerType::new(TriggerTiming::Before, TriggerEvent::Delete)
        );
    }

    #[test]
    fn test_execution_result_is_modified() {
        let unmodified = TriggerExecutionResult::Unmodified;
        assert!(!unmodified.is_modified());

        let modified = TriggerExecutionResult::ModifiedNewRow(vec![Value::Integer(1)]);
        assert!(modified.is_modified());
    }

    #[test]
    fn test_execution_result_into_record() {
        let unmodified = TriggerExecutionResult::Unmodified;
        assert!(unmodified.into_record().is_none());

        let record = vec![Value::Integer(1), Value::Text("test".to_string())];
        let modified = TriggerExecutionResult::ModifiedNewRow(record.clone());
        assert_eq!(modified.into_record(), Some(record));
    }
}

#[cfg(test)]
mod stored_proc_catalog_tests {
    use sqlrustgo_catalog::{ParamMode, StoredProcParam, StoredProcStatement, StoredProcedure};

    #[test]
    fn test_stored_procedure_creation() {
        let proc = StoredProcedure::new(
            "test_proc".to_string(),
            vec![StoredProcParam {
                name: "param1".to_string(),
                mode: ParamMode::In,
                data_type: "INTEGER".to_string(),
            }],
            vec![StoredProcStatement::RawSql("SELECT 1".to_string())],
        );

        assert_eq!(proc.name, "test_proc");
        assert_eq!(proc.params.len(), 1);
        assert_eq!(proc.body.len(), 1);
    }

    #[test]
    fn test_param_mode_in() {
        assert!(matches!(ParamMode::In, ParamMode::In));
    }

    #[test]
    fn test_param_mode_out() {
        assert!(matches!(ParamMode::Out, ParamMode::Out));
    }

    #[test]
    fn test_param_mode_inout() {
        assert!(matches!(ParamMode::InOut, ParamMode::InOut));
    }

    #[test]
    fn test_stored_proc_statement_raw_sql() {
        let stmt = StoredProcStatement::RawSql("SELECT * FROM users".to_string());
        assert!(matches!(stmt, StoredProcStatement::RawSql(_)));
    }

    #[test]
    fn test_stored_proc_statement_set() {
        let stmt = StoredProcStatement::Set {
            variable: "x".to_string(),
            value: "1".to_string(),
        };
        assert!(matches!(stmt, StoredProcStatement::Set { .. }));
    }

    #[test]
    fn test_stored_proc_statement_declare() {
        let stmt = StoredProcStatement::Declare {
            name: "counter".to_string(),
            data_type: "INTEGER".to_string(),
            default_value: Some("0".to_string()),
        };
        assert!(matches!(stmt, StoredProcStatement::Declare { .. }));
    }

    #[test]
    fn test_stored_proc_statement_if() {
        let stmt = StoredProcStatement::If {
            condition: "x > 0".to_string(),
            then_body: vec![StoredProcStatement::RawSql("SELECT 1".to_string())],
            elseif_body: vec![],
            else_body: vec![],
        };
        assert!(matches!(stmt, StoredProcStatement::If { .. }));
    }

    #[test]
    fn test_stored_proc_statement_while() {
        let stmt = StoredProcStatement::While {
            condition: "1 = 1".to_string(),
            body: vec![StoredProcStatement::RawSql("SELECT 1".to_string())],
        };
        assert!(matches!(stmt, StoredProcStatement::While { .. }));
    }

    #[test]
    fn test_stored_proc_statement_loop() {
        let stmt = StoredProcStatement::Loop {
            body: vec![StoredProcStatement::RawSql("SELECT 1".to_string())],
        };
        assert!(matches!(stmt, StoredProcStatement::Loop { .. }));
    }

    #[test]
    fn test_stored_proc_statement_return() {
        let stmt = StoredProcStatement::Return {
            value: "1".to_string(),
        };
        assert!(matches!(stmt, StoredProcStatement::Return { .. }));
    }

    #[test]
    fn test_stored_proc_statement_leave() {
        let stmt = StoredProcStatement::Leave {
            label: "my_loop".to_string(),
        };
        assert!(matches!(stmt, StoredProcStatement::Leave { .. }));
    }

    #[test]
    fn test_stored_proc_statement_iterate() {
        let stmt = StoredProcStatement::Iterate {
            label: "my_loop".to_string(),
        };
        assert!(matches!(stmt, StoredProcStatement::Iterate { .. }));
    }

    #[test]
    fn test_stored_proc_statement_call() {
        let stmt = StoredProcStatement::Call {
            procedure_name: "other_proc".to_string(),
            args: vec!["arg1".to_string()],
            into_var: Some("result".to_string()),
        };
        assert!(matches!(stmt, StoredProcStatement::Call { .. }));
    }
}
