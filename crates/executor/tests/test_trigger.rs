//! Integration tests for Trigger Executor

use sqlrustgo_executor::trigger::TriggerExecutor;
use sqlrustgo_storage::{
    ColumnDefinition, MemoryStorage, StorageEngine, TableInfo, TriggerInfo as StorageTriggerInfo,
};
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
        foreign_keys: vec![], unique_constraints: vec![],
    };

    storage.create_table(&table_info).unwrap();
    storage
}

// ============================================================================
// BEFORE Trigger Tests
// ============================================================================

#[test]
fn test_before_insert_trigger() {
    let mut storage = create_test_storage();

    let trigger = StorageTriggerInfo {
        name: "before_insert".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::Before,
        event: sqlrustgo_storage::TriggerEvent::Insert,
        body: "SET NEW.total = NEW.price * NEW.quantity".to_string(),
    };

    storage.create_trigger(trigger).unwrap();

    let new_row = vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Null,
    ];

    let trigger_exec = TriggerExecutor::new(Arc::new(storage));
    let result = trigger_exec.execute_before_insert("orders", &new_row);
    assert!(result.is_ok());
}

#[test]
fn test_before_update_trigger() {
    let mut storage = create_test_storage();

    let trigger = StorageTriggerInfo {
        name: "before_update".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::Before,
        event: sqlrustgo_storage::TriggerEvent::Update,
        body: "SET NEW.total = NEW.price * NEW.quantity".to_string(),
    };

    storage.create_trigger(trigger).unwrap();

    let old_row = vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Null,
    ];

    let new_row = vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(10),
        Value::Null,
    ];

    let trigger_exec = TriggerExecutor::new(Arc::new(storage));
    let result = trigger_exec.execute_before_update("orders", &old_row, &new_row);
    assert!(result.is_ok());
}

#[test]
fn test_before_delete_trigger() {
    let mut storage = create_test_storage();

    let trigger = StorageTriggerInfo {
        name: "before_delete".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::Before,
        event: sqlrustgo_storage::TriggerEvent::Delete,
        body: "SELECT 1".to_string(),
    };

    storage.create_trigger(trigger).unwrap();

    let old_row = vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Float(50.0),
    ];

    let trigger_exec = TriggerExecutor::new(Arc::new(storage));
    let result = trigger_exec.execute_before_delete("orders", &old_row);
    assert!(result.is_ok());
}

// ============================================================================
// AFTER Trigger Tests
// ============================================================================

#[test]
fn test_after_insert_trigger() {
    let mut storage = create_test_storage();

    let trigger = StorageTriggerInfo {
        name: "after_insert".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::After,
        event: sqlrustgo_storage::TriggerEvent::Insert,
        body: "SELECT 1".to_string(),
    };

    storage.create_trigger(trigger).unwrap();

    let new_row = vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Float(50.0),
    ];

    let trigger_exec = TriggerExecutor::new(Arc::new(storage));
    let result = trigger_exec.execute_after_insert("orders", &new_row);
    assert!(result.is_ok());
}

#[test]
fn test_after_update_trigger() {
    let mut storage = create_test_storage();

    let trigger = StorageTriggerInfo {
        name: "after_update".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::After,
        event: sqlrustgo_storage::TriggerEvent::Update,
        body: "SELECT 1".to_string(),
    };

    storage.create_trigger(trigger).unwrap();

    let old_row = vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Float(50.0),
    ];

    let new_row = vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(10),
        Value::Float(100.0),
    ];

    let trigger_exec = TriggerExecutor::new(Arc::new(storage));
    let result = trigger_exec.execute_after_update("orders", &old_row, &new_row);
    assert!(result.is_ok());
}

#[test]
fn test_after_delete_trigger() {
    let mut storage = create_test_storage();

    let trigger = StorageTriggerInfo {
        name: "after_delete".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::After,
        event: sqlrustgo_storage::TriggerEvent::Delete,
        body: "SELECT 1".to_string(),
    };

    storage.create_trigger(trigger).unwrap();

    let old_row = vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Float(50.0),
    ];

    let trigger_exec = TriggerExecutor::new(Arc::new(storage));
    let result = trigger_exec.execute_after_delete("orders", &old_row);
    assert!(result.is_ok());
}

// ============================================================================
// Multiple Triggers Tests
// ============================================================================

#[test]
fn test_multiple_triggers_order() {
    let mut storage = create_test_storage();

    let trigger1 = StorageTriggerInfo {
        name: "trigger_1".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::Before,
        event: sqlrustgo_storage::TriggerEvent::Insert,
        body: "SET NEW.total = 100".to_string(),
    };

    let trigger2 = StorageTriggerInfo {
        name: "trigger_2".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::Before,
        event: sqlrustgo_storage::TriggerEvent::Insert,
        body: "SET NEW.total = NEW.total + 50".to_string(),
    };

    storage.create_trigger(trigger1).unwrap();
    storage.create_trigger(trigger2).unwrap();

    let new_row = vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Null,
    ];

    let trigger_exec = TriggerExecutor::new(Arc::new(storage));
    let result = trigger_exec.execute_before_insert("orders", &new_row);
    assert!(result.is_ok());
}
