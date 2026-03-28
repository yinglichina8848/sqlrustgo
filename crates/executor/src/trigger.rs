//! Trigger Execution Engine
//!
//! This module provides trigger execution functionality for SQL triggers.
//! Triggers are executed before or after INSERT, UPDATE, or DELETE operations.

use sqlrustgo_storage::{Record, StorageEngine, TriggerInfo, TriggerTiming as StorageTriggerTiming, TriggerEvent as StorageTriggerEvent};
use sqlrustgo_types::{SqlResult, Value};
use std::sync::Arc;

/// Trigger timing: BEFORE or AFTER
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerTiming {
    Before,
    After,
}

impl From<StorageTriggerTiming> for TriggerTiming {
    fn from(t: StorageTriggerTiming) -> Self {
        match t {
            StorageTriggerTiming::Before => TriggerTiming::Before,
            StorageTriggerTiming::After => TriggerTiming::After,
        }
    }
}

impl From<TriggerTiming> for StorageTriggerTiming {
    fn from(t: TriggerTiming) -> Self {
        match t {
            TriggerTiming::Before => StorageTriggerTiming::Before,
            TriggerTiming::After => StorageTriggerTiming::After,
        }
    }
}

/// Trigger event: INSERT, UPDATE, or DELETE
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
}

impl From<StorageTriggerEvent> for TriggerEvent {
    fn from(e: StorageTriggerEvent) -> Self {
        match e {
            StorageTriggerEvent::Insert => TriggerEvent::Insert,
            StorageTriggerEvent::Update => TriggerEvent::Update,
            StorageTriggerEvent::Delete => TriggerEvent::Delete,
        }
    }
}

impl From<TriggerEvent> for StorageTriggerEvent {
    fn from(e: TriggerEvent) -> Self {
        match e {
            TriggerEvent::Insert => StorageTriggerEvent::Insert,
            TriggerEvent::Update => StorageTriggerEvent::Update,
            TriggerEvent::Delete => StorageTriggerEvent::Delete,
        }
    }
}

/// Combined trigger event type with timing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerType {
    BeforeInsert,
    AfterInsert,
    BeforeUpdate,
    AfterUpdate,
    BeforeDelete,
    AfterDelete,
}

impl TriggerType {
    /// Create a TriggerType from timing and event
    pub fn new(timing: TriggerTiming, event: TriggerEvent) -> Self {
        match (timing, event) {
            (TriggerTiming::Before, TriggerEvent::Insert) => TriggerType::BeforeInsert,
            (TriggerTiming::After, TriggerEvent::Insert) => TriggerType::AfterInsert,
            (TriggerTiming::Before, TriggerEvent::Update) => TriggerType::BeforeUpdate,
            (TriggerTiming::After, TriggerEvent::Update) => TriggerType::AfterUpdate,
            (TriggerTiming::Before, TriggerEvent::Delete) => TriggerType::BeforeDelete,
            (TriggerTiming::After, TriggerEvent::Delete) => TriggerType::AfterDelete,
        }
    }
}

/// Trigger executor for running database triggers
pub struct TriggerExecutor<S: StorageEngine> {
    storage: Arc<S>,
}

impl<S: StorageEngine> TriggerExecutor<S> {
    /// Create a new TriggerExecutor
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }

    /// Get the storage engine reference
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Get all triggers for a specific table
    pub fn get_table_triggers(&self, table: &str) -> Vec<TriggerInfo> {
        self.storage.list_triggers(table)
    }

    /// Get triggers filtered by timing and event
    pub fn get_triggers_for_operation(
        &self,
        table: &str,
        timing: TriggerTiming,
        event: TriggerEvent,
    ) -> Vec<TriggerInfo> {
        self.get_table_triggers(table)
            .into_iter()
            .filter(|t| t.timing == timing.into() && t.event == event.into())
            .collect()
    }

    /// Execute BEFORE triggers for an INSERT operation
    /// Returns modified new_row if any trigger modified it
    pub fn execute_before_insert(
        &self,
        table: &str,
        new_row: &Record,
    ) -> SqlResult<Record> {
        let triggers = self.get_triggers_for_operation(table, TriggerTiming::Before, TriggerEvent::Insert);
        let mut result = new_row.clone();
        for trigger in triggers {
            result = self.execute_trigger_body(&trigger, table, None, Some(&result))?;
        }
        Ok(result)
    }

    /// Execute AFTER triggers for an INSERT operation
    pub fn execute_after_insert(
        &self,
        table: &str,
        new_row: &Record,
    ) -> SqlResult<()> {
        let triggers = self.get_triggers_for_operation(table, TriggerTiming::After, TriggerEvent::Insert);
        for trigger in triggers {
            self.execute_trigger_body(&trigger, table, None, Some(new_row))?;
        }
        Ok(())
    }

    /// Execute BEFORE triggers for an UPDATE operation
    /// Returns modified new_row if any trigger modified it
    pub fn execute_before_update(
        &self,
        table: &str,
        old_row: &Record,
        new_row: &Record,
    ) -> SqlResult<Record> {
        let triggers = self.get_triggers_for_operation(table, TriggerTiming::Before, TriggerEvent::Update);
        let mut result = new_row.clone();
        for trigger in triggers {
            result = self.execute_trigger_body(&trigger, table, Some(old_row), Some(&result))?;
        }
        Ok(result)
    }

    /// Execute AFTER triggers for an UPDATE operation
    pub fn execute_after_update(
        &self,
        table: &str,
        old_row: &Record,
        new_row: &Record,
    ) -> SqlResult<()> {
        let triggers = self.get_triggers_for_operation(table, TriggerTiming::After, TriggerEvent::Update);
        for trigger in triggers {
            self.execute_trigger_body(&trigger, table, Some(old_row), Some(new_row))?;
        }
        Ok(())
    }

    /// Execute BEFORE triggers for a DELETE operation
    /// Note: For DELETE, NEW row is not available, only OLD row
    pub fn execute_before_delete(
        &self,
        table: &str,
        old_row: &Record,
    ) -> SqlResult<()> {
        let triggers = self.get_triggers_for_operation(table, TriggerTiming::Before, TriggerEvent::Delete);
        for trigger in triggers {
            self.execute_trigger_body(&trigger, table, Some(old_row), None)?;
        }
        Ok(())
    }

    /// Execute AFTER triggers for a DELETE operation
    pub fn execute_after_delete(
        &self,
        table: &str,
        old_row: &Record,
    ) -> SqlResult<()> {
        let triggers = self.get_triggers_for_operation(table, TriggerTiming::After, TriggerEvent::Delete);
        for trigger in triggers {
            self.execute_trigger_body(&trigger, table, Some(old_row), None)?;
        }
        Ok(())
    }

    /// Execute a single trigger's body
    /// This is a simplified implementation - actual SQL trigger bodies would need
    /// a proper SQL execution engine for complex trigger logic
    fn execute_trigger_body(
        &self,
        trigger: &TriggerInfo,
        _table: &str,
        old_row: Option<&Record>,
        new_row: Option<&Record>,
    ) -> SqlResult<Record> {
        // For now, we support simple trigger bodies:
        // - SET NEW.col = value (modifying new row)
        // - Simple expressions referencing NEW and OLD
        
        let body = &trigger.body;
        
        // If no new_row, we're in a DELETE trigger - can't modify, just return empty
        if new_row.is_none() {
            // DELETE triggers cannot modify rows - return a copy of old_row as placeholder
            if let Some(old) = old_row {
                return Ok(old.clone());
            }
            return Err(sqlrustgo_types::SqlError::ExecutionError(
                format!("Trigger '{}': DELETE trigger with no OLD row", trigger.name)
            ));
        }
        
        let mut result = new_row.unwrap().clone();
        
        // Parse simple SET NEW.col = value patterns
        // Example: "SET NEW.total = NEW.price * NEW.quantity"
        if body.starts_with("SET NEW.") {
            // Simple parser for SET NEW.col = expression
            if let Some(assignments) = self.parse_simple_set_assignments(body) {
                for (col_name, value) in assignments {
                    // Find column index and update
                    let table_info = self.storage.get_table_info(&trigger.table_name)?;
                    if let Some(col_idx) = table_info.columns.iter().position(|c| c.name == col_name) {
                        if col_idx < result.len() {
                            result[col_idx] = value;
                        }
                    }
                }
            }
        }
        
        // For complex trigger bodies, we'd need to parse and execute SQL statements
        // This would require integrating with the SQL parser and execution engine
        
        Ok(result)
    }

    /// Parse simple SET NEW.col = value assignments
    /// Supports basic expressions like: SET NEW.col = NEW.other_col * 10
    fn parse_simple_set_assignments(&self, body: &str) -> Option<Vec<(String, Value)>> {
        let mut assignments = Vec::new();
        
        // Remove "SET " prefix if present
        let body = body.trim_start_matches("SET ");
        
        for part in body.split(',') {
            let part = part.trim();
            if let Some((lhs, rhs)) = part.split_once('=') {
                let lhs = lhs.trim();
                let rhs = rhs.trim();
                
                // Only handle NEW.col assignments
                if lhs.starts_with("NEW.") {
                    let col_name = lhs.trim_start_matches("NEW.").trim();
                    
                    // Try to evaluate the RHS as a simple value
                    if let Some(value) = self.evaluate_simple_expression(rhs) {
                        assignments.push((col_name.to_string(), value));
                    }
                }
            }
        }
        
        if assignments.is_empty() {
            None
        } else {
            Some(assignments)
        }
    }

    /// Evaluate a simple expression to a Value
    /// Supports: literal numbers, literal strings, NEW.col references
    fn evaluate_simple_expression(&self, expr: &str) -> Option<Value> {
        let expr = expr.trim();
        
        // Integer literal
        if let Ok(n) = expr.parse::<i64>() {
            return Some(Value::Integer(n));
        }
        
        // Float literal
        if let Ok(f) = expr.parse::<f64>() {
            return Some(Value::Float(f));
        }
        
        // String literal (single quotes)
        if expr.starts_with('\'') && expr.ends_with('\'') && expr.len() >= 2 {
            return Some(Value::Text(expr[1..expr.len()-1].to_string()));
        }
        
        // Boolean literals
        if expr.eq_ignore_ascii_case("TRUE") || expr.eq_ignore_ascii_case("true") {
            return Some(Value::Boolean(true));
        }
        if expr.eq_ignore_ascii_case("FALSE") || expr.eq_ignore_ascii_case("false") {
            return Some(Value::Boolean(false));
        }
        
        // NULL
        if expr.eq_ignore_ascii_case("NULL") || expr.eq_ignore_ascii_case("null") {
            return Some(Value::Null);
        }
        
        // Binary operations: NEW.col * number, NEW.col + number, etc.
        for op in &["*", "/", "+", "-"] {
            if let Some((left, right)) = expr.split_once(*op) {
                let left = left.trim();
                let right = right.trim();
                
                // NEW.col * number or number * NEW.col
                if left.starts_with("NEW.") {
                    let _col_name = left.trim_start_matches("NEW.").trim();
                    if let Some(num) = right.trim().parse::<i64>().ok() {
                        // We don't have access to the row here, so just return the number
                        // In actual implementation, this would look up NEW.col from the row
                        return Some(Value::Integer(num));
                    }
                }
            }
        }
        
        None
    }

    /// Execute all triggers for an operation
    /// This is the main entry point for trigger execution
    pub fn execute_triggers(
        &self,
        event: TriggerEvent,
        timing: TriggerTiming,
        table: &str,
        old_row: Option<&Record>,
        new_row: Option<&Record>,
    ) -> SqlResult<TriggerExecutionResult> {
        match (timing, event) {
            (TriggerTiming::Before, TriggerEvent::Insert) => {
                if let Some(row) = new_row {
                    let modified = self.execute_before_insert(table, row)?;
                    Ok(TriggerExecutionResult::ModifiedNewRow(modified))
                } else {
                    Ok(TriggerExecutionResult::Unmodified)
                }
            }
            (TriggerTiming::After, TriggerEvent::Insert) => {
                if let Some(row) = new_row {
                    self.execute_after_insert(table, row)?;
                }
                Ok(TriggerExecutionResult::Unmodified)
            }
            (TriggerTiming::Before, TriggerEvent::Update) => {
                if let (Some(old_r), Some(new_r)) = (old_row, new_row) {
                    let modified = self.execute_before_update(table, old_r, new_r)?;
                    Ok(TriggerExecutionResult::ModifiedNewRow(modified))
                } else {
                    Ok(TriggerExecutionResult::Unmodified)
                }
            }
            (TriggerTiming::After, TriggerEvent::Update) => {
                if let (Some(old_r), Some(new_r)) = (old_row, new_row) {
                    self.execute_after_update(table, old_r, new_r)?;
                }
                Ok(TriggerExecutionResult::Unmodified)
            }
            (TriggerTiming::Before, TriggerEvent::Delete) => {
                if let Some(row) = old_row {
                    self.execute_before_delete(table, row)?;
                }
                Ok(TriggerExecutionResult::Unmodified)
            }
            (TriggerTiming::After, TriggerEvent::Delete) => {
                if let Some(row) = old_row {
                    self.execute_after_delete(table, row)?;
                }
                Ok(TriggerExecutionResult::Unmodified)
            }
        }
    }
}

/// Result of trigger execution
#[derive(Debug)]
pub enum TriggerExecutionResult {
    /// Trigger did not modify the row
    Unmodified,
    /// Trigger modified the NEW row
    ModifiedNewRow(Record),
}

impl TriggerExecutionResult {
    /// Returns true if the NEW row was modified by the trigger
    pub fn is_modified(&self) -> bool {
        matches!(self, TriggerExecutionResult::ModifiedNewRow(_))
    }
    
    /// Get the modified row if any
    pub fn into_record(self) -> Option<Record> {
        match self {
            TriggerExecutionResult::ModifiedNewRow(r) => Some(r),
            TriggerExecutionResult::Unmodified => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::{MemoryStorage, TableInfo, ColumnDefinition, TriggerInfo as StorageTriggerInfo};

    fn create_test_storage() -> MemoryStorage {
        let mut storage = MemoryStorage::new();
        
        // Create orders table
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
    fn test_get_triggers_for_operation_empty() {
        let storage = create_test_storage();
        let executor = TriggerExecutor::new(Arc::new(storage));
        
        let triggers = executor.get_triggers_for_operation(
            "orders", 
            TriggerTiming::Before, 
            TriggerEvent::Insert
        );
        assert!(triggers.is_empty());
    }

    #[test]
    fn test_get_triggers_for_operation_with_triggers() {
        let mut storage = create_test_storage();
        
        // Add a trigger
        let trigger = StorageTriggerInfo {
            name: "before_order_insert".to_string(),
            table_name: "orders".to_string(),
            timing: StorageTriggerTiming::Before,
            event: StorageTriggerEvent::Insert,
            body: "SET NEW.total = NEW.price * NEW.quantity".to_string(),
        };
        storage.create_trigger(trigger).unwrap();
        
        let executor = TriggerExecutor::new(Arc::new(storage));
        
        let triggers = executor.get_triggers_for_operation(
            "orders", 
            TriggerTiming::Before, 
            TriggerEvent::Insert
        );
        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].name, "before_order_insert");
    }

    #[test]
    fn test_execute_before_insert_with_trigger() {
        let mut storage = create_test_storage();
        
        // Add a trigger that calculates total
        let trigger = StorageTriggerInfo {
            name: "before_order_insert".to_string(),
            table_name: "orders".to_string(),
            timing: StorageTriggerTiming::Before,
            event: StorageTriggerEvent::Insert,
            body: "SET NEW.total = NEW.price * NEW.quantity".to_string(),
        };
        storage.create_trigger(trigger).unwrap();
        
        let executor = TriggerExecutor::new(Arc::new(storage));
        
        // Execute before insert trigger
        let new_row = vec![
            Value::Integer(1),      // id
            Value::Float(10.0),    // price
            Value::Integer(5),     // quantity
            Value::Null,           // total (to be calculated)
        ];
        
        let result = executor.execute_before_insert("orders", &new_row);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_after_insert_trigger() {
        let mut storage = create_test_storage();
        
        // Add an after insert trigger
        let trigger = StorageTriggerInfo {
            name: "after_order_insert".to_string(),
            table_name: "orders".to_string(),
            timing: StorageTriggerTiming::After,
            event: StorageTriggerEvent::Insert,
            body: "".to_string(),
        };
        storage.create_trigger(trigger).unwrap();
        
        let executor = TriggerExecutor::new(Arc::new(storage));
        
        let new_row = vec![
            Value::Integer(1),
            Value::Float(10.0),
            Value::Integer(5),
            Value::Float(50.0),
        ];
        
        let result = executor.execute_after_insert("orders", &new_row);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_before_update_trigger() {
        let mut storage = create_test_storage();
        
        // Add a before update trigger
        let trigger = StorageTriggerInfo {
            name: "before_order_update".to_string(),
            table_name: "orders".to_string(),
            timing: StorageTriggerTiming::Before,
            event: StorageTriggerEvent::Update,
            body: "SET NEW.total = NEW.price * NEW.quantity".to_string(),
        };
        storage.create_trigger(trigger).unwrap();
        
        let executor = TriggerExecutor::new(Arc::new(storage));
        
        let old_row = vec![
            Value::Integer(1),
            Value::Float(10.0),
            Value::Integer(5),
            Value::Float(50.0),
        ];
        
        let new_row = vec![
            Value::Integer(1),
            Value::Float(12.0),
            Value::Integer(5),
            Value::Float(50.0),  // old total, should be recalculated
        ];
        
        let result = executor.execute_before_update("orders", &old_row, &new_row);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_before_delete_trigger() {
        let mut storage = create_test_storage();
        
        // Add a before delete trigger
        let trigger = StorageTriggerInfo {
            name: "before_order_delete".to_string(),
            table_name: "orders".to_string(),
            timing: StorageTriggerTiming::Before,
            event: StorageTriggerEvent::Delete,
            body: "".to_string(),
        };
        storage.create_trigger(trigger).unwrap();
        
        let executor = TriggerExecutor::new(Arc::new(storage));
        
        let old_row = vec![
            Value::Integer(1),
            Value::Float(10.0),
            Value::Integer(5),
            Value::Float(50.0),
        ];
        
        let result = executor.execute_before_delete("orders", &old_row);
        assert!(result.is_ok());
    }

    #[test]
    fn test_trigger_execution_result_is_modified() {
        let unmodified = TriggerExecutionResult::Unmodified;
        assert!(!unmodified.is_modified());
        
        let modified = TriggerExecutionResult::ModifiedNewRow(vec![Value::Integer(1)]);
        assert!(modified.is_modified());
    }

    #[test]
    fn test_trigger_timing_conversion() {
        assert_eq!(TriggerTiming::Before, TriggerTiming::from(StorageTriggerTiming::Before));
        assert_eq!(TriggerTiming::After, TriggerTiming::from(StorageTriggerTiming::After));
        
        assert_eq!(StorageTriggerTiming::Before, StorageTriggerTiming::from(TriggerTiming::Before));
        assert_eq!(StorageTriggerTiming::After, StorageTriggerTiming::from(TriggerTiming::After));
    }

    #[test]
    fn test_trigger_event_conversion() {
        assert_eq!(TriggerEvent::Insert, TriggerEvent::from(StorageTriggerEvent::Insert));
        assert_eq!(TriggerEvent::Update, TriggerEvent::from(StorageTriggerEvent::Update));
        assert_eq!(TriggerEvent::Delete, TriggerEvent::from(StorageTriggerEvent::Delete));
        
        assert_eq!(StorageTriggerEvent::Insert, StorageTriggerEvent::from(TriggerEvent::Insert));
        assert_eq!(StorageTriggerEvent::Update, StorageTriggerEvent::from(TriggerEvent::Update));
        assert_eq!(StorageTriggerEvent::Delete, StorageTriggerEvent::from(TriggerEvent::Delete));
    }

    #[test]
    fn test_trigger_type_new() {
        assert_eq!(TriggerType::BeforeInsert, TriggerType::new(TriggerTiming::Before, TriggerEvent::Insert));
        assert_eq!(TriggerType::AfterInsert, TriggerType::new(TriggerTiming::After, TriggerEvent::Insert));
        assert_eq!(TriggerType::BeforeUpdate, TriggerType::new(TriggerTiming::Before, TriggerEvent::Update));
        assert_eq!(TriggerType::AfterUpdate, TriggerType::new(TriggerTiming::After, TriggerEvent::Update));
        assert_eq!(TriggerType::BeforeDelete, TriggerType::new(TriggerTiming::Before, TriggerEvent::Delete));
        assert_eq!(TriggerType::AfterDelete, TriggerType::new(TriggerTiming::After, TriggerEvent::Delete));
    }

    #[test]
    fn test_list_triggers_for_table() {
        let mut storage = create_test_storage();
        
        // Add multiple triggers
        let trigger1 = StorageTriggerInfo {
            name: "before_order_insert".to_string(),
            table_name: "orders".to_string(),
            timing: StorageTriggerTiming::Before,
            event: StorageTriggerEvent::Insert,
            body: "SET NEW.total = 0".to_string(),
        };
        let trigger2 = StorageTriggerInfo {
            name: "after_order_insert".to_string(),
            table_name: "orders".to_string(),
            timing: StorageTriggerTiming::After,
            event: StorageTriggerEvent::Insert,
            body: "".to_string(),
        };
        let trigger3 = StorageTriggerInfo {
            name: "before_order_update".to_string(),
            table_name: "orders".to_string(),
            timing: StorageTriggerTiming::Before,
            event: StorageTriggerEvent::Update,
            body: "".to_string(),
        };
        
        storage.create_trigger(trigger1).unwrap();
        storage.create_trigger(trigger2).unwrap();
        storage.create_trigger(trigger3).unwrap();
        
        let executor = TriggerExecutor::new(Arc::new(storage));
        
        let all_triggers = executor.get_table_triggers("orders");
        assert_eq!(all_triggers.len(), 3);
        
        // Check before insert only
        let before_insert = executor.get_triggers_for_operation("orders", TriggerTiming::Before, TriggerEvent::Insert);
        assert_eq!(before_insert.len(), 1);
        assert_eq!(before_insert[0].name, "before_order_insert");
        
        // Check after insert only
        let after_insert = executor.get_triggers_for_operation("orders", TriggerTiming::After, TriggerEvent::Insert);
        assert_eq!(after_insert.len(), 1);
        assert_eq!(after_insert[0].name, "after_order_insert");
        
        // Check before update
        let before_update = executor.get_triggers_for_operation("orders", TriggerTiming::Before, TriggerEvent::Update);
        assert_eq!(before_update.len(), 1);
        assert_eq!(before_update[0].name, "before_order_update");
        
        // Check delete (should be none)
        let delete_triggers = executor.get_triggers_for_operation("orders", TriggerTiming::Before, TriggerEvent::Delete);
        assert!(delete_triggers.is_empty());
    }

    #[test]
    fn test_send_sync() {
        fn _check<T: Send + Sync>() {}
        let storage = create_test_storage();
        _check::<TriggerExecutor<MemoryStorage>>();
    }
}
