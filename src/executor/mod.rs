//! Query Execution Engine
//! Executes SQL statements and returns results

use crate::parser::{Statement, SelectStatement, InsertStatement};
use crate::types::{Value, SqlError, SqlResult};
use crate::storage::{BufferPool, Page, BPlusTree};

/// Execution result
#[derive(Debug)]
pub struct ExecutionResult {
    pub rows_affected: u64,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}

/// Query execution engine
pub struct ExecutionEngine {
    buffer_pool: BufferPool,
    tables: std::collections::HashMap<String, TableInfo>,
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub fn new() -> Self {
        Self {
            buffer_pool: BufferPool::new(100),
            tables: std::collections::HashMap::new(),
        }
    }

    /// Execute a SQL statement
    pub fn execute(&mut self, statement: Statement) -> SqlResult<ExecutionResult> {
        match statement {
            Statement::Select(s) => self.execute_select(s),
            Statement::Insert(s) => self.execute_insert(s),
            Statement::Update(_) => Err(SqlError::ExecutionError("UPDATE not implemented".to_string())),
            Statement::Delete(_) => Err(SqlError::ExecutionError("DELETE not implemented".to_string())),
            Statement::CreateTable(c) => self.execute_create_table(c),
            Statement::DropTable(d) => self.execute_drop_table(d),
        }
    }

    /// Execute SELECT
    fn execute_select(&mut self, stmt: SelectStatement) -> SqlResult<ExecutionResult> {
        // Check if table exists
        if !self.tables.contains_key(&stmt.table) {
            return Err(SqlError::TableNotFound(stmt.table));
        }

        // Return result with columns
        Ok(ExecutionResult {
            rows_affected: 0,
            columns: stmt.columns.iter().map(|c| c.name.clone()).collect(),
            rows: Vec::new(),
        })
    }

    /// Execute INSERT
    fn execute_insert(&mut self, stmt: InsertStatement) -> SqlResult<ExecutionResult> {
        // Check if table exists
        if !self.tables.contains_key(&stmt.table) {
            return Err(SqlError::TableNotFound(stmt.table));
        }

        // Simplified: just count inserted rows
        Ok(ExecutionResult {
            rows_affected: stmt.values.len() as u64,
            columns: Vec::new(),
            rows: Vec::new(),
        })
    }

    /// Execute CREATE TABLE
    fn execute_create_table(&mut self, stmt: crate::parser::CreateTableStatement) -> SqlResult<ExecutionResult> {
        self.tables.insert(stmt.name.clone(), TableInfo {
            name: stmt.name,
            columns: stmt.columns,
        });

        Ok(ExecutionResult {
            rows_affected: 0,
            columns: Vec::new(),
            rows: Vec::new(),
        })
    }

    /// Execute DROP TABLE
    fn execute_drop_table(&mut self, stmt: crate::parser::DropTableStatement) -> SqlResult<ExecutionResult> {
        self.tables.remove(&stmt.name);

        Ok(ExecutionResult {
            rows_affected: 0,
            columns: Vec::new(),
            rows: Vec::new(),
        })
    }

    /// Get table info
    pub fn get_table(&self, name: &str) -> Option<&TableInfo> {
        self.tables.get(name)
    }
}

/// Table metadata
#[derive(Debug)]
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

    #[test]
    fn test_execution_engine_create() {
        let engine = ExecutionEngine::new();
        assert!(engine.tables.is_empty());
    }

    #[test]
    fn test_execute_create_table() {
        let mut engine = ExecutionEngine::new();
        let result = engine.execute(
            crate::parser::parse("CREATE TABLE users").unwrap()
        );
        assert!(result.is_ok());
        assert!(engine.get_table("users").is_some());
    }

    #[test]
    fn test_execute_select() {
        let mut engine = ExecutionEngine::new();
        // Create table first
        engine.execute(crate::parser::parse("CREATE TABLE users").unwrap());
        
        // Select from existing table
        let result = engine.execute(
            crate::parser::parse("SELECT id FROM users").unwrap()
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_select_nonexistent_table() {
        let mut engine = ExecutionEngine::new();
        let result = engine.execute(
            crate::parser::parse("SELECT * FROM nonexistent").unwrap()
        );
        assert!(result.is_err());
    }
}
