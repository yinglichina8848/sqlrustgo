//! Benchmark tests for Stored Procedure and Trigger Executors

use sqlrustgo_catalog::{Catalog, StoredProcStatement, StoredProcedure};
use sqlrustgo_executor::stored_proc::StoredProcExecutor;
use sqlrustgo_storage::{MemoryStorage, StorageEngine};
use std::sync::{Arc, RwLock};

fn create_executor_with_proc(proc: StoredProcedure) -> StoredProcExecutor {
    let storage: Arc<RwLock<dyn StorageEngine>> = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut catalog = Catalog::new();
    catalog.add_stored_procedure(proc).unwrap();
    StoredProcExecutor::new(Arc::new(catalog), storage)
}

#[cfg(test)]
mod stored_proc_bench {
    use super::*;

    fn create_simple_proc(iterations: usize) -> StoredProcedure {
        let mut body = vec![];
        body.push(StoredProcStatement::Declare {
            name: "counter".to_string(),
            data_type: "INTEGER".to_string(),
            default_value: Some("0".to_string()),
        });

        body.push(StoredProcStatement::While {
            condition: "@counter < 1000".to_string(),
            body: vec![StoredProcStatement::Set {
                variable: "counter".to_string(),
                value: "@counter + 1".to_string(),
            }],
        });

        body.push(StoredProcStatement::Return {
            value: "@counter".to_string(),
        });

        StoredProcedure::new(format!("bench_{}", iterations), vec![], body)
    }

    #[test]
    fn test_simple_loop_1000_iterations() {
        let proc = create_simple_proc(1000);
        let executor = create_executor_with_proc(proc);

        let start = std::time::Instant::now();
        let result = executor.execute_call("bench_1000", vec![]);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        println!("1000 loop iterations took: {:?}", elapsed);
    }

    #[test]
    fn test_nested_loop_100x10() {
        let mut body = vec![];
        body.push(StoredProcStatement::Declare {
            name: "i".to_string(),
            data_type: "INTEGER".to_string(),
            default_value: Some("0".to_string()),
        });
        body.push(StoredProcStatement::Declare {
            name: "j".to_string(),
            data_type: "INTEGER".to_string(),
            default_value: Some("0".to_string()),
        });

        // Outer loop: i = 0 to 99
        body.push(StoredProcStatement::While {
            condition: "@i < 100".to_string(),
            body: vec![
                StoredProcStatement::Set {
                    variable: "i".to_string(),
                    value: "@i + 1".to_string(),
                },
                // Inner loop: j = 0 to 9
                StoredProcStatement::While {
                    condition: "@j < 10".to_string(),
                    body: vec![StoredProcStatement::Set {
                        variable: "j".to_string(),
                        value: "@j + 1".to_string(),
                    }],
                },
            ],
        });

        body.push(StoredProcStatement::Return {
            value: "@i".to_string(),
        });

        let proc = StoredProcedure::new("nested_loop".to_string(), vec![], body);
        let executor = create_executor_with_proc(proc);

        let start = std::time::Instant::now();
        let result = executor.execute_call("nested_loop", vec![]);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        println!("Nested loop 100x10 took: {:?}", elapsed);
    }

    #[test]
    fn test_multiple_conditionals() {
        let mut body = vec![];
        body.push(StoredProcStatement::Declare {
            name: "x".to_string(),
            data_type: "INTEGER".to_string(),
            default_value: Some("0".to_string()),
        });

        // 100 IF statements
        for i in 0..100 {
            body.push(StoredProcStatement::If {
                condition: format!("@x < {}", i),
                then_body: vec![StoredProcStatement::Set {
                    variable: "x".to_string(),
                    value: format!("{}", i),
                }],
                elseif_body: vec![],
                else_body: vec![],
            });
        }

        body.push(StoredProcStatement::Return {
            value: "@x".to_string(),
        });

        let proc = StoredProcedure::new("multiple_conditionals".to_string(), vec![], body);
        let executor = create_executor_with_proc(proc);

        let start = std::time::Instant::now();
        let result = executor.execute_call("multiple_conditionals", vec![]);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        println!("100 conditionals took: {:?}", elapsed);
    }

    #[test]
    fn test_deeply_nested_blocks() {
        // Create 50 nested BEGIN/END blocks
        let mut body = vec![];
        let mut innermost: Vec<StoredProcStatement> = vec![StoredProcStatement::Return {
            value: "1".to_string(),
        }];

        for i in (0..50).rev() {
            body = vec![StoredProcStatement::Block {
                label: Some(format!("block_{}", i)),
                body: innermost,
            }];
            innermost = body.clone();
        }

        let proc = StoredProcedure::new("deeply_nested".to_string(), vec![], body);
        let executor = create_executor_with_proc(proc);

        let start = std::time::Instant::now();
        let result = executor.execute_call("deeply_nested", vec![]);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        println!("50 deeply nested blocks took: {:?}", elapsed);
    }
}

#[cfg(test)]
mod trigger_bench {
    use sqlrustgo_executor::trigger::TriggerExecutor;
    use sqlrustgo_storage::{
        ColumnDefinition, MemoryStorage, StorageEngine, TableInfo,
        TriggerEvent as StorageTriggerEvent, TriggerInfo, TriggerTiming as StorageTriggerTiming,
    };
    use sqlrustgo_types::Value;
    use std::sync::{Arc, RwLock};

    fn create_storage_with_triggers(trigger_count: usize) -> Arc<RwLock<dyn StorageEngine>> {
        let mut storage = MemoryStorage::new();

        storage
            .create_table(&TableInfo {
                name: "test_table".to_string(),
                columns: vec![
                    ColumnDefinition::new("id", "INTEGER"),
                    ColumnDefinition::new("value", "INTEGER"),
                ],
            })
            .unwrap();

        for i in 0..trigger_count {
            storage
                .create_trigger(TriggerInfo {
                    name: format!("trigger_{}", i),
                    table_name: "test_table".to_string(),
                    timing: StorageTriggerTiming::Before,
                    event: StorageTriggerEvent::Insert,
                    body: format!("SET NEW.value = {}", i),
                })
                .unwrap();
        }

        Arc::new(RwLock::new(storage))
    }

    #[test]
    fn test_trigger_execution_overhead() {
        let storage = create_storage_with_triggers(10);
        let executor = TriggerExecutor::new(storage);

        let new_row = vec![Value::Integer(1), Value::Integer(0)];

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let result = executor.execute_before_insert("test_table", &new_row);
            assert!(result.is_ok());
        }
        let elapsed = start.elapsed();

        println!("10 triggers x 1000 executions took: {:?}", elapsed);
    }

    #[test]
    fn test_trigger_list_lookup_overhead() {
        let storage = create_storage_with_triggers(100);
        let executor = TriggerExecutor::new(storage);

        let new_row = vec![Value::Integer(1), Value::Integer(0)];

        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let result = executor.execute_before_insert("test_table", &new_row);
            assert!(result.is_ok());
        }
        let elapsed = start.elapsed();

        println!("100 triggers x 10000 executions took: {:?}", elapsed);
    }
}
