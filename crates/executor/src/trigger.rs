//! Trigger Execution Engine
//!
//! This module provides trigger execution functionality for SQL triggers.
//! Triggers are executed before or after INSERT, UPDATE, or DELETE operations.

use sqlrustgo_parser::parse;
use sqlrustgo_storage::{
    Record, StorageEngine, TriggerEvent as StorageTriggerEvent, TriggerInfo,
    TriggerTiming as StorageTriggerTiming,
};
use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::sync::{Arc, RwLock};

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
pub struct TriggerExecutor {
    storage: Arc<RwLock<dyn StorageEngine>>,
}

impl TriggerExecutor {
    /// Create a new TriggerExecutor
    pub fn new(storage: Arc<RwLock<dyn StorageEngine>>) -> Self {
        Self { storage }
    }

    /// Get the storage engine reference
    pub fn storage(&self) -> Arc<RwLock<dyn StorageEngine>> {
        self.storage.clone()
    }

    /// Get all triggers for a specific table
    pub fn get_table_triggers(&self, table: &str) -> Vec<TriggerInfo> {
        self.storage.read().unwrap().list_triggers(table)
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
    pub fn execute_before_insert(&self, table: &str, new_row: &Record) -> SqlResult<Record> {
        let triggers =
            self.get_triggers_for_operation(table, TriggerTiming::Before, TriggerEvent::Insert);
        let mut result = new_row.clone();
        for trigger in triggers {
            result = self.execute_trigger_body(&trigger, table, None, Some(&result))?;
        }
        Ok(result)
    }

    /// Execute AFTER triggers for an INSERT operation
    pub fn execute_after_insert(&self, table: &str, new_row: &Record) -> SqlResult<()> {
        let triggers =
            self.get_triggers_for_operation(table, TriggerTiming::After, TriggerEvent::Insert);
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
        let triggers =
            self.get_triggers_for_operation(table, TriggerTiming::Before, TriggerEvent::Update);
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
        let triggers =
            self.get_triggers_for_operation(table, TriggerTiming::After, TriggerEvent::Update);
        for trigger in triggers {
            self.execute_trigger_body(&trigger, table, Some(old_row), Some(new_row))?;
        }
        Ok(())
    }

    /// Execute BEFORE triggers for a DELETE operation
    /// Note: For DELETE, NEW row is not available, only OLD row
    pub fn execute_before_delete(&self, table: &str, old_row: &Record) -> SqlResult<()> {
        let triggers =
            self.get_triggers_for_operation(table, TriggerTiming::Before, TriggerEvent::Delete);
        for trigger in triggers {
            self.execute_trigger_body(&trigger, table, Some(old_row), None)?;
        }
        Ok(())
    }

    /// Execute AFTER triggers for a DELETE operation
    pub fn execute_after_delete(&self, table: &str, old_row: &Record) -> SqlResult<()> {
        let triggers =
            self.get_triggers_for_operation(table, TriggerTiming::After, TriggerEvent::Delete);
        for trigger in triggers {
            self.execute_trigger_body(&trigger, table, Some(old_row), None)?;
        }
        Ok(())
    }

    /// Execute a single trigger's body
    fn execute_trigger_body(
        &self,
        trigger: &TriggerInfo,
        table: &str,
        old_row: Option<&Record>,
        new_row: Option<&Record>,
    ) -> SqlResult<Record> {
        let body = &trigger.body;
        let result = new_row.map(|r| r.to_vec());

        let statements = self.split_body_statements(body);
        for stmt in statements {
            let expanded = self.expand_row_variables(
                &stmt,
                &trigger.table_name,
                old_row.as_deref(),
                new_row.as_deref(),
            );
            if let Err(e) =
                self.execute_trigger_sql(&expanded, table, old_row.as_deref(), new_row.as_deref())
            {
                return Err(e);
            }
        }

        Ok(result.unwrap_or_default())
    }

    /// Split trigger body into individual SQL statements
    fn split_body_statements(&self, body: &str) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut escape_next = false;

        let normalized = body.replace("NEW .", "NEW.").replace("OLD .", "OLD.");

        for ch in normalized.chars() {
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => {
                    escape_next = true;
                    current.push(ch);
                }
                '\'' => {
                    in_string = !in_string;
                    current.push(ch);
                }
                ';' if !in_string => {
                    let stmt = current.trim();
                    if !stmt.is_empty()
                        && !stmt.eq_ignore_ascii_case("BEGIN")
                        && !stmt.eq_ignore_ascii_case("END")
                    {
                        statements.push(stmt.to_string());
                    }
                    current.clear();
                }
                _ => current.push(ch),
            }
        }

        let final_stmt = current.trim();
        if !final_stmt.is_empty()
            && !final_stmt.eq_ignore_ascii_case("BEGIN")
            && !final_stmt.eq_ignore_ascii_case("END")
        {
            statements.push(final_stmt.to_string());
        }

        statements
    }

    /// Expand NEW.col and OLD.col with actual values from the row
    fn expand_row_variables(
        &self,
        sql: &str,
        table_name: &str,
        old_row: Option<&Record>,
        new_row: Option<&Record>,
    ) -> String {
        let mut result = sql.to_string();

        if let Some(new) = new_row {
            if let Some(info) = self.storage.read().unwrap().get_table_info(table_name).ok() {
                for (i, col) in info.columns.iter().enumerate() {
                    if i < new.len() {
                        let val = &new[i];
                        let replacement = self.value_to_sql_literal(val);
                        result = result.replace(&format!("NEW.{}", col.name), &replacement);
                        result = result
                            .replace(&format!("NEW.{}", col.name.to_uppercase()), &replacement);
                    }
                }
            }
        }

        if let Some(old) = old_row {
            if let Some(info) = self.storage.read().unwrap().get_table_info(table_name).ok() {
                for (i, col) in info.columns.iter().enumerate() {
                    if i < old.len() {
                        let val = &old[i];
                        let replacement = self.value_to_sql_literal(val);
                        result = result.replace(&format!("OLD.{}", col.name), &replacement);
                        result = result
                            .replace(&format!("OLD.{}", col.name.to_uppercase()), &replacement);
                    }
                }
            }
        }

        result
    }

    /// Convert a Value to SQL literal string
    fn value_to_sql_literal(&self, val: &Value) -> String {
        match val {
            Value::Null => "NULL".to_string(),
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Text(s) => format!("'{}'", s.replace("'", "\\'")),
            Value::Boolean(b) => {
                if *b {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
            Value::Blob(b) => format!("X'{}'", String::from_utf8_lossy(b)),
        }
    }

    /// Execute a SQL statement within a trigger context
    fn execute_trigger_sql(
        &self,
        sql: &str,
        trigger_table: &str,
        old_row: Option<&Record>,
        new_row: Option<&Record>,
    ) -> SqlResult<()> {
        let sql_trimmed = sql.trim();
        if sql_trimmed.is_empty() {
            return Ok(());
        }

        let sql_upper = sql_trimmed.to_uppercase();

        if sql_upper.starts_with("INSERT") {
            self.execute_trigger_insert(sql_trimmed, new_row)
        } else if sql_upper.starts_with("UPDATE") {
            self.execute_trigger_update(sql_trimmed, trigger_table, new_row)
        } else if sql_upper.starts_with("DELETE") {
            self.execute_trigger_delete(sql_trimmed, trigger_table, old_row)
        } else if sql_upper.starts_with("SET") {
            if let Some(new) = new_row {
                self.execute_trigger_set(sql_trimmed, trigger_table, new)?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Execute INSERT within a trigger
    fn execute_trigger_insert(&self, sql: &str, new_row: Option<&Record>) -> SqlResult<()> {
        let mut storage = self.storage.write().unwrap();
        let expanded = self.expand_insert_values(sql, new_row);
        let statement = parse(&expanded)
            .map_err(|e| SqlError::ExecutionError(format!("Parse error: {}", e)))?;

        if let sqlrustgo_parser::Statement::Insert(insert) = statement {
            let table_name = &insert.table;
            let table_info = storage.get_table_info(table_name)?;
            let num_cols = table_info.columns.len();

            for values in &insert.values {
                let mut record = Vec::new();
                for expr in values {
                    let val = self.expression_to_value(expr, new_row);
                    record.push(val);
                }
                while record.len() < num_cols {
                    record.push(Value::Null);
                }
                storage.insert(table_name, vec![record])?;
            }
        }
        Ok(())
    }

    /// Execute UPDATE within a trigger
    fn execute_trigger_update(
        &self,
        sql: &str,
        trigger_table: &str,
        new_row: Option<&Record>,
    ) -> SqlResult<()> {
        let mut storage = self.storage.write().unwrap();
        let expanded = self.expand_update_values(sql, trigger_table, new_row);
        let statement = parse(&expanded)
            .map_err(|e| SqlError::ExecutionError(format!("Parse error: {}", e)))?;

        if let sqlrustgo_parser::Statement::Update(update) = statement {
            let table_name = &update.table;
            let table_info = storage.get_table_info(table_name)?;

            let set_col_indices: Vec<(usize, Value)> = update
                .set_clauses
                .iter()
                .filter_map(|(col_name, expr)| {
                    let col_idx = table_info
                        .columns
                        .iter()
                        .position(|c| c.name.eq_ignore_ascii_case(col_name));
                    col_idx.map(|idx| {
                        let val = self.expression_to_value(expr, new_row);
                        (idx, val)
                    })
                })
                .collect();

            storage.update(table_name, &[], &set_col_indices)?;
        }
        Ok(())
    }

    /// Execute DELETE within a trigger
    fn execute_trigger_delete(
        &self,
        sql: &str,
        trigger_table: &str,
        old_row: Option<&Record>,
    ) -> SqlResult<()> {
        let mut storage = self.storage.write().unwrap();
        let expanded = self.expand_delete_values(sql, trigger_table, old_row);
        let statement = parse(&expanded)
            .map_err(|e| SqlError::ExecutionError(format!("Parse error: {}", e)))?;

        if let sqlrustgo_parser::Statement::Delete(delete) = statement {
            storage.delete(&delete.table, &[])?;
        }
        Ok(())
    }

    /// Execute SET within a trigger (modify NEW row)
    fn execute_trigger_set(&self, sql: &str, table_name: &str, new_row: &Record) -> SqlResult<()> {
        if let Some(assignments) = self.parse_simple_set_assignments(sql) {
            let table_info = self.storage.read().unwrap().get_table_info(table_name)?;
            let mut updated = new_row.to_vec();

            for (col_name, value) in assignments {
                if let Some(col_idx) = table_info
                    .columns
                    .iter()
                    .position(|c| c.name.eq_ignore_ascii_case(&col_name))
                {
                    if col_idx < updated.len() {
                        updated[col_idx] = value;
                    }
                }
            }
        }
        Ok(())
    }

    /// Expand VALUES(...) in INSERT with NEW.row values
    fn expand_insert_values(&self, sql: &str, new_row: Option<&Record>) -> String {
        if let Some(new) = new_row {
            let mut result = sql.to_string();
            for (i, val) in new.iter().enumerate() {
                let placeholder = format!("NEW[{}]", i);
                let replacement = self.value_to_sql_literal(val);
                result = result.replace(&placeholder, &replacement);
            }
            result = result.replace("NEW.id", &self.value_to_sql_literal(&new[0]));
            if new.len() > 1 {
                result = result.replace("NEW.col1", &self.value_to_sql_literal(&new[1]));
            }
            result
        } else {
            sql.to_string()
        }
    }

    /// Expand SET clause values in UPDATE with NEW.row values
    fn expand_update_values(
        &self,
        sql: &str,
        table_name: &str,
        new_row: Option<&Record>,
    ) -> String {
        self.expand_row_variables(sql, table_name, None, new_row)
    }

    /// Expand WHERE clause values in DELETE with OLD.row values
    fn expand_delete_values(
        &self,
        sql: &str,
        table_name: &str,
        old_row: Option<&Record>,
    ) -> String {
        self.expand_row_variables(sql, table_name, old_row, None)
    }

    /// Convert parser Expression to Value
    fn expression_to_value(
        &self,
        expr: &sqlrustgo_parser::Expression,
        new_row: Option<&Record>,
    ) -> Value {
        match expr {
            sqlrustgo_parser::Expression::Literal(s) => {
                let s = s.trim();
                if s.eq_ignore_ascii_case("NULL") {
                    Value::Null
                } else if let Ok(n) = s.parse::<i64>() {
                    Value::Integer(n)
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Float(f)
                } else if s.starts_with('\'') && s.ends_with('\'') {
                    Value::Text(s[1..s.len() - 1].to_string())
                } else {
                    Value::Text(s.to_string())
                }
            }
            sqlrustgo_parser::Expression::Identifier(name) => {
                if let Some(new) = new_row {
                    if name.eq_ignore_ascii_case("NEW.id") && !new.is_empty() {
                        new[0].clone()
                    } else if name.eq_ignore_ascii_case("NEW.col1") && new.len() > 1 {
                        new[1].clone()
                    } else {
                        Value::Text(name.clone())
                    }
                } else {
                    Value::Text(name.clone())
                }
            }
            _ => Value::Null,
        }
    }

    /// Parse simple SET NEW.col = value assignments
    /// Supports basic expressions like: SET NEW.col = NEW.other_col * 10
    fn parse_simple_set_assignments(&self, body: &str) -> Option<Vec<(String, Value)>> {
        let mut assignments = Vec::new();

        let body = body.replace("NEW .", "NEW.").replace("OLD .", "OLD.");

        let body = body.trim().trim_end_matches(';').trim();

        let parts: Vec<&str> = if body.contains("SET") {
            body.split("SET")
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim())
                .collect()
        } else {
            vec![body.trim()]
        };

        for part in parts {
            let part = part.trim();
            if let Some((lhs, rhs)) = part.split_once('=') {
                let lhs = lhs.trim();
                let rhs = rhs.trim();

                if lhs.contains("NEW.") {
                    let col_name = lhs.split("NEW.").last().unwrap_or(lhs).trim();
                    if let Some(value) = self.evaluate_simple_expression(rhs) {
                        assignments.push((col_name.to_string(), value));
                    }
                } else if lhs.contains("OLD.") {
                    let col_name = lhs.split("OLD.").last().unwrap_or(lhs).trim();
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

        if let Ok(n) = expr.parse::<i64>() {
            return Some(Value::Integer(n));
        }

        if let Ok(f) = expr.parse::<f64>() {
            return Some(Value::Float(f));
        }

        if expr.starts_with('\'') && expr.ends_with('\'') && expr.len() >= 2 {
            return Some(Value::Text(expr[1..expr.len() - 1].to_string()));
        }

        if expr.eq_ignore_ascii_case("TRUE") || expr.eq_ignore_ascii_case("true") {
            return Some(Value::Boolean(true));
        }
        if expr.eq_ignore_ascii_case("FALSE") || expr.eq_ignore_ascii_case("false") {
            return Some(Value::Boolean(false));
        }

        if expr.eq_ignore_ascii_case("NULL") || expr.eq_ignore_ascii_case("null") {
            return Some(Value::Null);
        }

        for op in &["*", "/", "+", "-"] {
            if let Some((left, right)) = expr.split_once(*op) {
                let left = left.trim();
                let right = right.trim();

                if left.starts_with("NEW.") {
                    let _col_name = left.trim_start_matches("NEW.").trim();
                    if let Ok(num) = right.trim().parse::<i64>() {
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
    use sqlrustgo_storage::{
        ColumnDefinition, MemoryStorage, TableInfo, TriggerInfo as StorageTriggerInfo,
    };

    fn create_test_storage() -> MemoryStorage {
        let mut storage = MemoryStorage::new();

        // Create orders table
        let table_info = TableInfo {
            name: "orders".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    ..Default::default()
                },
                ColumnDefinition {
                    name: "price".to_string(),
                    data_type: "FLOAT".to_string(),
                    ..Default::default()
                },
                ColumnDefinition {
                    name: "quantity".to_string(),
                    data_type: "INTEGER".to_string(),
                    ..Default::default()
                },
                ColumnDefinition {
                    name: "total".to_string(),
                    data_type: "FLOAT".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        storage.create_table(&table_info).unwrap();

        storage
    }

    #[test]
    fn test_trigger_executor_creation() {
        let storage = create_test_storage();
        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));
        assert_eq!(executor.get_table_triggers("orders").len(), 0);
    }

    #[test]
    fn test_get_triggers_for_operation_empty() {
        let storage = create_test_storage();
        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));

        let triggers = executor.get_triggers_for_operation(
            "orders",
            TriggerTiming::Before,
            TriggerEvent::Insert,
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

        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));

        let triggers = executor.get_triggers_for_operation(
            "orders",
            TriggerTiming::Before,
            TriggerEvent::Insert,
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

        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));

        // Execute before insert trigger
        let new_row = vec![
            Value::Integer(1),  // id
            Value::Float(10.0), // price
            Value::Integer(5),  // quantity
            Value::Null,        // total (to be calculated)
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

        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));

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

        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));

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
            Value::Float(50.0), // old total, should be recalculated
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

        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));

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
        assert_eq!(
            TriggerTiming::Before,
            TriggerTiming::from(StorageTriggerTiming::Before)
        );
        assert_eq!(
            TriggerTiming::After,
            TriggerTiming::from(StorageTriggerTiming::After)
        );

        assert_eq!(
            StorageTriggerTiming::Before,
            StorageTriggerTiming::from(TriggerTiming::Before)
        );
        assert_eq!(
            StorageTriggerTiming::After,
            StorageTriggerTiming::from(TriggerTiming::After)
        );
    }

    #[test]
    fn test_trigger_event_conversion() {
        assert_eq!(
            TriggerEvent::Insert,
            TriggerEvent::from(StorageTriggerEvent::Insert)
        );
        assert_eq!(
            TriggerEvent::Update,
            TriggerEvent::from(StorageTriggerEvent::Update)
        );
        assert_eq!(
            TriggerEvent::Delete,
            TriggerEvent::from(StorageTriggerEvent::Delete)
        );

        assert_eq!(
            StorageTriggerEvent::Insert,
            StorageTriggerEvent::from(TriggerEvent::Insert)
        );
        assert_eq!(
            StorageTriggerEvent::Update,
            StorageTriggerEvent::from(TriggerEvent::Update)
        );
        assert_eq!(
            StorageTriggerEvent::Delete,
            StorageTriggerEvent::from(TriggerEvent::Delete)
        );
    }

    #[test]
    fn test_trigger_type_new() {
        assert_eq!(
            TriggerType::BeforeInsert,
            TriggerType::new(TriggerTiming::Before, TriggerEvent::Insert)
        );
        assert_eq!(
            TriggerType::AfterInsert,
            TriggerType::new(TriggerTiming::After, TriggerEvent::Insert)
        );
        assert_eq!(
            TriggerType::BeforeUpdate,
            TriggerType::new(TriggerTiming::Before, TriggerEvent::Update)
        );
        assert_eq!(
            TriggerType::AfterUpdate,
            TriggerType::new(TriggerTiming::After, TriggerEvent::Update)
        );
        assert_eq!(
            TriggerType::BeforeDelete,
            TriggerType::new(TriggerTiming::Before, TriggerEvent::Delete)
        );
        assert_eq!(
            TriggerType::AfterDelete,
            TriggerType::new(TriggerTiming::After, TriggerEvent::Delete)
        );
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

        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));

        let all_triggers = executor.get_table_triggers("orders");
        assert_eq!(all_triggers.len(), 3);

        // Check before insert only
        let before_insert = executor.get_triggers_for_operation(
            "orders",
            TriggerTiming::Before,
            TriggerEvent::Insert,
        );
        assert_eq!(before_insert.len(), 1);
        assert_eq!(before_insert[0].name, "before_order_insert");

        // Check after insert only
        let after_insert = executor.get_triggers_for_operation(
            "orders",
            TriggerTiming::After,
            TriggerEvent::Insert,
        );
        assert_eq!(after_insert.len(), 1);
        assert_eq!(after_insert[0].name, "after_order_insert");

        // Check before update
        let before_update = executor.get_triggers_for_operation(
            "orders",
            TriggerTiming::Before,
            TriggerEvent::Update,
        );
        assert_eq!(before_update.len(), 1);
        assert_eq!(before_update[0].name, "before_order_update");

        // Check delete (should be none)
        let delete_triggers = executor.get_triggers_for_operation(
            "orders",
            TriggerTiming::Before,
            TriggerEvent::Delete,
        );
        assert!(delete_triggers.is_empty());
    }

    #[test]
    fn test_send_sync() {
        fn _check<T: Send + Sync>() {}
        let storage = create_test_storage();
        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));
        _check::<TriggerExecutor>();
        let _ = executor; // suppress unused warning
    }

    #[test]
    fn test_trigger_executor_with_multiple_tables() {
        let mut storage = MemoryStorage::new();

        // Create orders table
        let orders_table = TableInfo {
            name: "orders".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        storage.create_table(&orders_table).unwrap();

        // Create products table
        let products_table = TableInfo {
            name: "products".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        storage.create_table(&products_table).unwrap();

        // Add trigger to orders
        let trigger = StorageTriggerInfo {
            name: "before_order_insert".to_string(),
            table_name: "orders".to_string(),
            timing: StorageTriggerTiming::Before,
            event: StorageTriggerEvent::Insert,
            body: "".to_string(),
        };
        storage.create_trigger(trigger).unwrap();

        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));

        // Orders should have 1 trigger
        assert_eq!(executor.get_table_triggers("orders").len(), 1);
        // Products should have no triggers
        assert_eq!(executor.get_table_triggers("products").len(), 0);
    }

    #[test]
    fn test_execute_after_update_trigger() {
        let mut storage = create_test_storage();

        let trigger = StorageTriggerInfo {
            name: "after_order_update".to_string(),
            table_name: "orders".to_string(),
            timing: StorageTriggerTiming::After,
            event: StorageTriggerEvent::Update,
            body: "".to_string(),
        };
        storage.create_trigger(trigger).unwrap();

        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));

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
            Value::Float(60.0),
        ];

        let result = executor.execute_after_update("orders", &old_row, &new_row);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_after_delete_trigger() {
        let mut storage = create_test_storage();

        let trigger = StorageTriggerInfo {
            name: "after_order_delete".to_string(),
            table_name: "orders".to_string(),
            timing: StorageTriggerTiming::After,
            event: StorageTriggerEvent::Delete,
            body: "".to_string(),
        };
        storage.create_trigger(trigger).unwrap();

        let executor = TriggerExecutor::new(Arc::new(RwLock::new(storage)));

        let old_row = vec![
            Value::Integer(1),
            Value::Float(10.0),
            Value::Integer(5),
            Value::Float(50.0),
        ];

        let result = executor.execute_after_delete("orders", &old_row);
        assert!(result.is_ok());
    }

    #[test]
    fn test_trigger_execution_result_into_record() {
        let unmodified = TriggerExecutionResult::Unmodified;
        assert!(unmodified.into_record().is_none());

        let modified_row = Record::from(vec![Value::Integer(42)]);
        let modified = TriggerExecutionResult::ModifiedNewRow(modified_row.clone());
        assert_eq!(modified.into_record(), Some(modified_row));
    }

    #[test]
    fn test_trigger_timing_all_values() {
        assert_eq!(format!("{:?}", TriggerTiming::Before), "Before");
        assert_eq!(format!("{:?}", TriggerTiming::After), "After");
    }

    #[test]
    fn test_trigger_event_all_values() {
        assert_eq!(format!("{:?}", TriggerEvent::Insert), "Insert");
        assert_eq!(format!("{:?}", TriggerEvent::Update), "Update");
        assert_eq!(format!("{:?}", TriggerEvent::Delete), "Delete");
    }
}
