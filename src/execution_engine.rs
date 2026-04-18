//! ExecutionEngine - high-level SQL execution API
//! Provides a simple interface for executing SQL statements against a storage backend.

use crate::{parse, SqlError, SqlResult, Value};
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_parser::parser::{
    CreateIndexStatement, CreateTableStatement, DropTableStatement, InsertStatement, SelectStatement,
};
use sqlrustgo_parser::{DeleteStatement, Statement, UpdateStatement};
use sqlrustgo_storage::{MemoryStorage, StorageEngine};
use std::sync::{Arc, RwLock};

/// Execution engine for SQL statements
pub struct ExecutionEngine<S: StorageEngine> {
    storage: Arc<RwLock<S>>,
}

/// Type alias for MemoryStorage-backed execution engine
pub type MemoryExecutionEngine = ExecutionEngine<MemoryStorage>;

impl<S: StorageEngine> ExecutionEngine<S> {
    /// Create a new execution engine
    pub fn new(storage: Arc<RwLock<S>>) -> Self {
        Self { storage }
    }

    /// Execute a SQL statement and return results
    pub fn execute(&mut self, sql: &str) -> SqlResult<ExecutorResult> {
        let statement = parse(sql).map_err(|e| SqlError::ParseError(e.to_string()))?;

        match statement {
            Statement::Select(ref select) => self.execute_select(select),
            Statement::Insert(ref insert) => self.execute_insert(insert),
            Statement::Update(ref update) => self.execute_update(update),
            Statement::Delete(ref delete) => self.execute_delete(delete),
            Statement::CreateTable(ref create) => self.execute_create_table(create),
            Statement::DropTable(ref drop) => self.execute_drop_table(drop),
            Statement::CreateIndex(ref idx) => self.execute_create_index(idx),
            Statement::Analyze(_) => Ok(ExecutorResult::empty()),
            _ => Err(SqlError::ExecutionError("Unsupported statement type".to_string())),
        }
    }

    fn execute_select(&self, select: &SelectStatement) -> SqlResult<ExecutorResult> {
        let storage = self.storage.read().unwrap();
        let rows = storage.scan(&select.table)?;
        Ok(ExecutorResult::new(rows, 0))
    }

    fn execute_insert(&self, insert: &InsertStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();

        // Get table info for FK validation
        let table_info = storage.get_table_info(&insert.table)?;

        // Build column name to index map
        let _col_indices: std::collections::HashMap<&str, usize> = if insert.columns.is_empty() {
            std::collections::HashMap::new()
        } else {
            insert
                .columns
                .iter()
                .enumerate()
                .map(|(i, c)| (c.as_str(), i))
                .collect()
        };

        // Convert expressions to records and validate FK constraints
        let mut all_records = Vec::new();
        for row_exprs in &insert.values {
            let mut record = Vec::with_capacity(row_exprs.len());
            for (_i, expr) in row_exprs.iter().enumerate() {
                let val = expression_to_value(expr);
                record.push(val);
            }

            // Validate foreign key constraints before insert
            if !table_info.foreign_keys.is_empty() {
                validate_foreign_keys(&*storage, &table_info, &record, &insert.columns)?;
            }

            all_records.push(record);
        }

        storage.insert(&insert.table, all_records)?;
        Ok(ExecutorResult::new(vec![], insert.values.len()))
    }

    fn execute_update(&self, update: &UpdateStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        let count = storage.update(&update.table, &[], &[])?;
        Ok(ExecutorResult::new(vec![], count))
    }

    fn execute_delete(&self, delete: &DeleteStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        let count = storage.delete(&delete.table, &[])?;
        Ok(ExecutorResult::new(vec![], count))
    }

    fn execute_create_table(&self, create: &CreateTableStatement) -> SqlResult<ExecutorResult> {
        use sqlrustgo_storage::{ColumnDefinition, TableInfo};
        let mut storage = self.storage.write().unwrap();
        let columns: Vec<ColumnDefinition> = create
            .columns
            .iter()
            .map(|c| ColumnDefinition {
                name: c.name.clone(),
                data_type: c.data_type.clone(),
                nullable: !c.primary_key,
                primary_key: c.primary_key,
            })
            .collect();
        let info = TableInfo {
            name: create.name.clone(),
            columns,
            foreign_keys: vec![],
            unique_constraints: vec![],
        };
        storage.create_table(&info)?;
        Ok(ExecutorResult::empty())
    }

    fn execute_drop_table(&self, drop: &DropTableStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        storage.drop_table(&drop.name)?;
        Ok(ExecutorResult::empty())
    }

    fn execute_create_index(&self, idx: &CreateIndexStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        let table_name = &idx.table;
        let col_name = idx
            .columns
            .first()
            .ok_or_else(|| SqlError::ExecutionError("No columns in index".to_string()))?;
        let table_info = storage.get_table_info(table_name)?;
        let col_idx = table_info
            .columns
            .iter()
            .position(|c| c.name == *col_name)
            .ok_or_else(|| SqlError::ExecutionError("Column not found".to_string()))?;
        storage.create_index(table_name, col_name, col_idx)?;
        Ok(ExecutorResult::empty())
    }
}

impl ExecutionEngine<MemoryStorage> {
    /// Create a new execution engine backed by MemoryStorage
    pub fn with_memory() -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
        }
    }
}

/// Convert a parser Expression to a Value (simple literal evaluation)
fn expression_to_value(expr: &sqlrustgo_parser::Expression) -> Value {
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
        sqlrustgo_parser::Expression::Identifier(name) => Value::Text(name.clone()),
        _ => Value::Null,
    }
}

/// Validate foreign key constraints for a row before insert
fn validate_foreign_keys(
    storage: &dyn StorageEngine,
    table_info: &sqlrustgo_storage::TableInfo,
    row: &[Value],
    insert_columns: &[String],
) -> SqlResult<()> {
    for fk in &table_info.foreign_keys {
        // Collect FK column values from the row
        let fk_values: Vec<Value> = fk
            .columns
            .iter()
            .filter_map(|col_name| {
                let col_idx = if insert_columns.is_empty() {
                    table_info.columns.iter().position(|c| c.name.eq_ignore_ascii_case(col_name))
                } else {
                    insert_columns.iter().position(|c| c.eq_ignore_ascii_case(col_name))
                };
                col_idx.and_then(|idx| row.get(idx).cloned())
            })
            .collect();

        // Skip if any FK value is NULL (NULL FKs are allowed)
        if fk_values.iter().any(|v| matches!(v, Value::Null)) {
            continue;
        }

        // Scan parent table to verify referenced row exists
        let parent_rows = storage.scan(&fk.referenced_table)?;

        // Find referenced column indices in parent table
        let ref_col_indices: Vec<usize> = fk
            .referenced_columns
            .iter()
            .filter_map(|col_name| {
                storage
                    .get_table_info(&fk.referenced_table)
                    .ok()?
                    .columns
                    .iter()
                    .position(|c| c.name.eq_ignore_ascii_case(col_name))
            })
            .collect();

        let parent_has_match = parent_rows.iter().any(|parent_row| {
            ref_col_indices
                .iter()
                .enumerate()
                .all(|(i, &col_idx)| parent_row.get(col_idx) == fk_values.get(i))
        });

        if !parent_has_match {
            return Err(SqlError::ExecutionError(format!(
                "Foreign key constraint failed: {} ({}) references {} ({}) which does not exist",
                table_info.name,
                fk.columns.join(", "),
                fk.referenced_table,
                fk.referenced_columns.join(", ")
            )));
        }
    }
    Ok(())
}
