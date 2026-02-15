//! Query Execution Engine
//! Executes SQL statements and returns results

use crate::parser::{
    DeleteStatement, Expression, InsertStatement, SelectStatement, Statement, UpdateStatement,
};
use crate::storage::{BufferPool, FileStorage};
use crate::types::{SqlError, SqlResult, Value, parse_sql_literal};
use std::path::PathBuf;

/// Execution result
#[derive(Debug)]
pub struct ExecutionResult {
    pub rows_affected: u64,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}

/// Query execution engine
#[allow(dead_code)]
pub struct ExecutionEngine {
    buffer_pool: BufferPool,
    storage: FileStorage,
}

impl ExecutionEngine {
    /// Create a new execution engine with file-based storage
    pub fn new() -> Self {
        Self::with_data_dir(std::path::PathBuf::from("data"))
    }

    /// Create a new execution engine with custom data directory
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        let storage = FileStorage::new(data_dir).expect("Failed to initialize file storage");
        Self {
            buffer_pool: BufferPool::new(100),
            storage,
        }
    }

    /// Execute a SQL statement
    pub fn execute(&mut self, statement: Statement) -> SqlResult<ExecutionResult> {
        match statement {
            Statement::Select(s) => self.execute_select(s),
            Statement::Insert(s) => self.execute_insert(s),
            Statement::Update(s) => self.execute_update(s),
            Statement::Delete(s) => self.execute_delete(s),
            Statement::CreateTable(c) => self.execute_create_table(c),
            Statement::DropTable(d) => self.execute_drop_table(d),
        }
    }

    /// Execute SELECT
    fn execute_select(&mut self, stmt: SelectStatement) -> SqlResult<ExecutionResult> {
        // Check if table exists
        let table_data = self.storage
            .get_table(&stmt.table)
            .ok_or_else(|| SqlError::TableNotFound(stmt.table.clone()))?;

        // Get column names from table schema
        let table_columns: Vec<String> = table_data.info.columns.iter()
            .map(|c| c.name.clone())
            .collect();

        // Determine result columns (use table columns if SELECT *)
        let result_columns: Vec<String> = if stmt.columns.iter().any(|c| c.name == "*") {
            table_columns.clone()
        } else {
            stmt.columns.iter().map(|c| c.name.clone()).collect()
        };

        // Find column indices for projection
        let column_indices: Vec<usize> = if stmt.columns.iter().any(|c| c.name == "*") {
            (0..table_columns.len()).collect()
        } else {
            stmt.columns.iter().filter_map(|c| {
                table_columns.iter().position(|tc| tc == &c.name)
            }).collect()
        };

        // Build column index map for WHERE clause evaluation
        let column_map: std::collections::HashMap<String, usize> = table_columns.iter()
            .enumerate()
            .map(|(i, c)| (c.clone(), i))
            .collect();

        // Filter rows by WHERE clause
        let filtered_rows: Vec<Vec<Value>> = if let Some(ref where_expr) = stmt.where_clause {
            table_data.rows.iter()
                .filter(|row| evaluate_where(row, where_expr, &column_map))
                .cloned()
                .collect()
        } else {
            table_data.rows.clone()
        };

        // Project to result columns
        let result_rows: Vec<Vec<Value>> = filtered_rows.iter()
            .map(|row| {
                column_indices.iter()
                    .filter_map(|&idx| row.get(idx).cloned())
                    .collect()
            })
            .collect();

        Ok(ExecutionResult {
            rows_affected: result_rows.len() as u64,
            columns: result_columns,
            rows: result_rows,
        })
    }

    /// Execute INSERT (supports multi-row)
    fn execute_insert(&mut self, stmt: InsertStatement) -> SqlResult<ExecutionResult> {
        // Check if table exists
        if !self.storage.contains_table(&stmt.table) {
            return Err(SqlError::TableNotFound(stmt.table));
        }

        // Convert expressions to values (multiple rows)
        let mut inserted_rows: Vec<Vec<Value>> = Vec::new();

        {
            let table_data = self.storage.get_table_mut(&stmt.table).unwrap();
            for row_expr in &stmt.values {
                let row: Vec<Value> = row_expr.iter().map(expression_to_value_static).collect();
                table_data.rows.push(row.clone());
                inserted_rows.push(row);
            }
        }
        self.storage.persist_table(&stmt.table)?;

        let rows_affected = inserted_rows.len() as u64;

        Ok(ExecutionResult {
            rows_affected,
            columns: Vec::new(),
            rows: inserted_rows,
        })
    }

    /// Execute UPDATE (with dynamic column mapping)
    fn execute_update(&mut self, stmt: UpdateStatement) -> SqlResult<ExecutionResult> {
        // Check if table exists
        if !self.storage.contains_table(&stmt.table) {
            return Err(SqlError::TableNotFound(stmt.table));
        }

        let where_clause = stmt.where_clause.clone();
        let set_clauses = stmt.set_clauses.clone();

        // Build column index map from table schema
        let column_indices: std::collections::HashMap<String, usize> = {
            let table_data = self.storage.get_table(&stmt.table).unwrap();
            table_data
                .info
                .columns
                .iter()
                .enumerate()
                .map(|(i, c)| (c.name.clone(), i))
                .collect()
        };

        let rows_affected = {
            let table_data = self.storage.get_table_mut(&stmt.table).unwrap();
            let mut count = 0;

            // Evaluate WHERE clause if present
            for row in &mut table_data.rows {
                // If no WHERE clause, update all rows
                let matches = if let Some(ref where_expr) = where_clause {
                    evaluate_where(row, where_expr, &column_indices)
                } else {
                    true
                };

                if matches {
                    // Apply SET clauses with dynamic column mapping
                    for (column, value_expr) in &set_clauses {
                        if let Some(&idx) = column_indices.get(column) {
                            if idx < row.len() {
                                row[idx] = expression_to_value_static(value_expr);
                            }
                        }
                    }
                    count += 1;
                }
            }
            count
        };

        // Persist to disk
        self.storage.persist_table(&stmt.table)?;

        Ok(ExecutionResult {
            rows_affected,
            columns: Vec::new(),
            rows: Vec::new(),
        })
    }

    /// Execute DELETE
    fn execute_delete(&mut self, stmt: DeleteStatement) -> SqlResult<ExecutionResult> {
        // Check if table exists
        if !self.storage.contains_table(&stmt.table) {
            return Err(SqlError::TableNotFound(stmt.table));
        }

        let where_clause = stmt.where_clause.clone();

        // Build column index map for WHERE clause evaluation
        let column_indices: std::collections::HashMap<String, usize> = {
            let table_data = self.storage.get_table(&stmt.table).unwrap();
            table_data
                .info
                .columns
                .iter()
                .enumerate()
                .map(|(i, c)| (c.name.clone(), i))
                .collect()
        };

        let rows_affected = {
            let table_data = self.storage.get_table_mut(&stmt.table).unwrap();
            let original_count = table_data.rows.len();

            // If WHERE clause is present, filter rows; otherwise delete all
            if let Some(where_expr) = where_clause {
                table_data
                    .rows
                    .retain(|row| !evaluate_where(row, &where_expr, &column_indices));
            } else {
                table_data.rows.clear();
            }

            (original_count - table_data.rows.len()) as u64
        };

        // Persist to disk
        self.storage.persist_table(&stmt.table)?;

        Ok(ExecutionResult {
            rows_affected,
            columns: Vec::new(),
            rows: Vec::new(),
        })
    }

    /// Execute CREATE TABLE
    fn execute_create_table(
        &mut self,
        stmt: crate::parser::CreateTableStatement,
    ) -> SqlResult<ExecutionResult> {
        let table_data = TableData {
            info: TableInfo {
                name: stmt.name.clone(),
                columns: stmt.columns,
            },
            rows: Vec::new(),
        };
        self.storage
            .insert_table(stmt.name, table_data)?;

        Ok(ExecutionResult {
            rows_affected: 0,
            columns: Vec::new(),
            rows: Vec::new(),
        })
    }

    /// Execute DROP TABLE
    fn execute_drop_table(
        &mut self,
        stmt: crate::parser::DropTableStatement,
    ) -> SqlResult<ExecutionResult> {
        self.storage
            .drop_table(&stmt.name)?;

        Ok(ExecutionResult {
            rows_affected: 0,
            columns: Vec::new(),
            rows: Vec::new(),
        })
    }

    /// Get table data
    pub fn get_table(&self, name: &str) -> Option<&TableData> {
        self.storage.get_table(name)
    }
}

/// Convert expression to value (static function)
fn expression_to_value_static(expr: &Expression) -> Value {
    match expr {
        Expression::Literal(s) => parse_sql_literal(s),
        Expression::Identifier(s) => parse_sql_literal(s),
        Expression::BinaryOp(_, _, _) => Value::Null, // TODO: evaluate expression
    }
}

/// Evaluate WHERE clause for a row with dynamic column mapping
fn evaluate_where(
    row: &[Value],
    expr: &Expression,
    column_indices: &std::collections::HashMap<String, usize>,
) -> bool {
    match expr {
        Expression::BinaryOp(left, op, right) => {
            // Get left value (column reference)
            let left_val = match left.as_ref() {
                Expression::Identifier(name) => {
                    // Dynamic column lookup
                    column_indices
                        .get(name)
                        .and_then(|&idx| row.get(idx))
                        .cloned()
                        .unwrap_or(Value::Null)
                }
                Expression::Literal(s) => parse_sql_literal(s),
                _ => Value::Null,
            };

            // Get right value
            let right_val = match right.as_ref() {
                Expression::Identifier(name) => {
                    // Dynamic column lookup
                    column_indices
                        .get(name)
                        .and_then(|&idx| row.get(idx))
                        .cloned()
                        .unwrap_or(Value::Null)
                }
                Expression::Literal(s) => parse_sql_literal(s),
                _ => Value::Null,
            };

            // Evaluate based on operator
            match op.as_str() {
                "=" => left_val == right_val,
                "!=" => left_val != right_val,
                ">" => {
                    match (&left_val, &right_val) {
                        (Value::Integer(l), Value::Integer(r)) => l > r,
                        (Value::Float(l), Value::Float(r)) => l > r,
                        (Value::Text(l), Value::Text(r)) => l > r,
                        _ => false,
                    }
                }
                "<" => {
                    match (&left_val, &right_val) {
                        (Value::Integer(l), Value::Integer(r)) => l < r,
                        (Value::Float(l), Value::Float(r)) => l < r,
                        (Value::Text(l), Value::Text(r)) => l < r,
                        _ => false,
                    }
                }
                ">=" => {
                    match (&left_val, &right_val) {
                        (Value::Integer(l), Value::Integer(r)) => l >= r,
                        (Value::Float(l), Value::Float(r)) => l >= r,
                        (Value::Text(l), Value::Text(r)) => l >= r,
                        _ => false,
                    }
                }
                "<=" => {
                    match (&left_val, &right_val) {
                        (Value::Integer(l), Value::Integer(r)) => l <= r,
                        (Value::Float(l), Value::Float(r)) => l <= r,
                        (Value::Text(l), Value::Text(r)) => l <= r,
                        _ => false,
                    }
                }
                _ => false,
            }
        }
        _ => true,
    }
}

/// Table data with rows
#[derive(Debug, Clone)]
pub struct TableData {
    pub info: TableInfo,
    pub rows: Vec<Vec<Value>>,
}

/// Table metadata
#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<crate::parser::ColumnDefinition>,
}

/// Execute a SQL string
pub fn execute(sql: &str) -> SqlResult<ExecutionResult> {
    let statement = crate::parser::parse(sql)?;
    let mut engine = ExecutionEngine::new();
    engine.execute(statement)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_execution_engine_create() {
        // Use a unique temp directory for test isolation
        let temp_dir = env::temp_dir().join(format!("sqlrustgo_test_{}", std::process::id()));
        let engine = ExecutionEngine::with_data_dir(temp_dir.clone());
        assert!(engine.storage.table_names().is_empty());
        // Clean up
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_execute_create_table() {
        let mut engine = ExecutionEngine::new();
        let result = engine.execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap());
        assert!(result.is_ok());
        assert!(engine.get_table("users").is_some());
    }

    #[test]
    fn test_execute_select() {
        let mut engine = ExecutionEngine::new();
        // Create table first
        let _ = engine.execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap());

        // Select from existing table
        let result = engine.execute(crate::parser::parse("SELECT id FROM users").unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_select_nonexistent_table() {
        let mut engine = ExecutionEngine::new();
        let result = engine.execute(crate::parser::parse("SELECT * FROM nonexistent").unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_insert() {
        let mut engine = ExecutionEngine::new();
        // Create table first
        engine
            .execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
            .unwrap();

        // Insert a row
        let result =
            engine.execute(crate::parser::parse("INSERT INTO users VALUES (1, 'Alice')").unwrap());
        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert_eq!(exec_result.rows_affected, 1); // 1 row inserted

        // Verify row was stored
        let table = engine.get_table("users").unwrap();
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn test_execute_select_with_data() {
        let mut engine = ExecutionEngine::new();
        // Create table first
        engine
            .execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
            .unwrap();

        // Insert rows
        engine
            .execute(crate::parser::parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
            .unwrap();
        engine
            .execute(crate::parser::parse("INSERT INTO users VALUES (2, 'Bob')").unwrap())
            .unwrap();

        // Select all rows
        let result = engine
            .execute(crate::parser::parse("SELECT * FROM users").unwrap())
            .unwrap();
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.columns, vec!["id", "name"]);
    }

    #[test]
    fn test_execute_select_with_where() {
        let mut engine = ExecutionEngine::new();
        // Create table first
        engine
            .execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
            .unwrap();

        // Insert rows
        engine
            .execute(crate::parser::parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
            .unwrap();
        engine
            .execute(crate::parser::parse("INSERT INTO users VALUES (2, 'Bob')").unwrap())
            .unwrap();

        // Select with WHERE clause
        let result = engine
            .execute(crate::parser::parse("SELECT * FROM users WHERE id = 1").unwrap())
            .unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], crate::types::Value::Integer(1));
    }

    #[test]
    fn test_execute_update() {
        let mut engine = ExecutionEngine::new();
        // Create table first
        engine
            .execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
            .unwrap();

        // Insert a row
        engine
            .execute(crate::parser::parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
            .unwrap();

        // Update the row
        let result = engine
            .execute(crate::parser::parse("UPDATE users SET name = 'Bob' WHERE id = 1").unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_delete() {
        let mut engine = ExecutionEngine::new();
        // Create table first
        engine
            .execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
            .unwrap();

        // Insert a row
        engine
            .execute(crate::parser::parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
            .unwrap();

        // Delete the row
        let result =
            engine.execute(crate::parser::parse("DELETE FROM users WHERE id = 1").unwrap());
        assert!(result.is_ok());

        // Verify row was deleted
        let table = engine.get_table("users").unwrap();
        assert_eq!(table.rows.len(), 0);
    }
}
