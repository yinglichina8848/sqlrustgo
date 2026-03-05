//! Query Service - decouples REPL from execution layer
//! Provides QueryService trait for query execution abstraction

use crate::executor::{ExecutionEngine, ExecutionResult, LocalExecutor};
use crate::parser::Statement;
use crate::types::error::SqlError;
use std::sync::{Arc, RwLock};

/// Query service result type
pub type QueryResult<T> = Result<T, SqlError>;

/// QueryService trait - abstraction for query execution
/// Enables decoupling REPL from execution engine and supports remote execution
pub trait QueryService: Send + Sync {
    /// Execute a SQL statement
    fn execute(&self, stmt: Statement) -> QueryResult<ExecutionResult>;

    /// Execute multiple statements in a transaction
    fn execute_batch(&self, stmts: Vec<Statement>) -> QueryResult<Vec<ExecutionResult>>;

    /// Check if the service is ready
    fn is_ready(&self) -> bool;
}

/// Default implementation of QueryService using ExecutionEngine
pub struct LocalQueryService {
    /// Execution engine
    engine: RwLock<ExecutionEngine>,
    /// Service name
    name: String,
}

impl LocalQueryService {
    /// Create a new local query service
    pub fn new() -> Self {
        Self {
            engine: RwLock::new(ExecutionEngine::new()),
            name: "local".to_string(),
        }
    }

    /// Create with custom data directory
    pub fn with_data_dir(data_dir: std::path::PathBuf) -> QueryResult<Self> {
        let engine = ExecutionEngine::with_data_dir(data_dir)?;
        Ok(Self {
            engine: RwLock::new(engine),
            name: "local".to_string(),
        })
    }

    /// Get service name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for LocalQueryService {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryService for LocalQueryService {
    fn execute(&self, stmt: Statement) -> QueryResult<ExecutionResult> {
        let mut engine = self.engine.write().map_err(|e| {
            SqlError::ExecutionError(format!("Failed to acquire engine lock: {}", e))
        })?;
        engine.execute(stmt)
    }

    fn execute_batch(&self, stmts: Vec<Statement>) -> QueryResult<Vec<ExecutionResult>> {
        let mut engine = self.engine.write().map_err(|e| {
            SqlError::ExecutionError(format!("Failed to acquire engine lock: {}", e))
        })?;
        stmts
            .into_iter()
            .map(|stmt| engine.execute(stmt))
            .collect()
    }

    fn is_ready(&self) -> bool {
        self.engine.try_read().is_ok()
    }
}

/// QueryService implementation using LocalExecutor (planner-based execution)
///
/// This implements E-004: integrates the LocalExecutor (Analyzer + Optimizer + Planner)
/// into the QueryService for full planner-based query execution.
pub struct PlannerQueryService {
    /// Local executor with full planner pipeline
    executor: RwLock<LocalExecutor>,
    /// Execution engine for DDL and other operations
    engine: RwLock<ExecutionEngine>,
    /// Service name
    name: String,
}

impl PlannerQueryService {
    /// Create a new planner-based query service
    pub fn new() -> Self {
        Self {
            executor: RwLock::new(LocalExecutor::new()),
            engine: RwLock::new(ExecutionEngine::new()),
            name: "planner".to_string(),
        }
    }

    /// Create with custom data directory
    pub fn with_data_dir(data_dir: std::path::PathBuf) -> QueryResult<Self> {
        let engine = ExecutionEngine::with_data_dir(data_dir)?;
        Ok(Self {
            executor: RwLock::new(LocalExecutor::new()),
            engine: RwLock::new(engine),
            name: "planner".to_string(),
        })
    }

    /// Get service name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the underlying executor for table registration
    pub fn executor(&self) -> &RwLock<LocalExecutor> {
        &self.executor
    }
}

impl Default for PlannerQueryService {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryService for PlannerQueryService {
    fn execute(&self, stmt: Statement) -> QueryResult<ExecutionResult> {
        // Try planner-based execution first (for SELECT queries)
        // Fall back to execution engine for DDL and other operations
        let planner_result = {
            let mut executor = self.executor.write().map_err(|e| {
                SqlError::ExecutionError(format!("Failed to acquire executor lock: {}", e))
            })?;
            executor.execute(stmt.clone())
        };

        match planner_result {
            Ok(rows) => Ok(ExecutionResult {
                rows_affected: rows.len() as u64,
                columns: vec![],
                rows,
            }),
            Err(_) => {
                // Fall back to execution engine for non-planner queries
                let mut engine = self.engine.write().map_err(|e| {
                    SqlError::ExecutionError(format!("Failed to acquire engine lock: {}", e))
                })?;
                engine.execute(stmt)
            }
        }
    }

    fn execute_batch(&self, stmts: Vec<Statement>) -> QueryResult<Vec<ExecutionResult>> {
        stmts.into_iter().map(|stmt| self.execute(stmt)).collect()
    }

    fn is_ready(&self) -> bool {
        self.executor.try_read().is_ok() && self.engine.try_read().is_ok()
    }
}

/// Thread-safe wrapper for QueryService
pub type QueryServiceHandle = Arc<dyn QueryService>;

/// Create a new query service handle
pub fn create_query_service() -> QueryServiceHandle {
    Arc::new(LocalQueryService::new())
}

/// Create query service with data directory
pub fn create_query_service_with_dir(
    data_dir: std::path::PathBuf,
) -> QueryResult<QueryServiceHandle> {
    Ok(Arc::new(LocalQueryService::with_data_dir(data_dir)?))
}

/// Create a planner-based query service handle
/// This uses the full planner pipeline (Analyzer + Optimizer + Planner)
pub fn create_planner_query_service() -> QueryServiceHandle {
    Arc::new(PlannerQueryService::new())
}

/// Create planner-based query service with data directory
pub fn create_planner_query_service_with_dir(
    data_dir: std::path::PathBuf,
) -> QueryResult<QueryServiceHandle> {
    Ok(Arc::new(PlannerQueryService::with_data_dir(data_dir)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_query_service() {
        let service = LocalQueryService::new();
        assert!(service.is_ready());
        assert_eq!(service.name(), "local");
    }

    #[test]
    fn test_execute_select() {
        let service = LocalQueryService::new();

        // Create table
        let create_stmt = crate::parser::parse("CREATE TABLE test (id INTEGER, name TEXT)").unwrap();
        service.execute(create_stmt).unwrap();

        // Insert data
        let insert_stmt = crate::parser::parse("INSERT INTO test VALUES (1, 'hello')").unwrap();
        service.execute(insert_stmt).unwrap();

        // Select
        let select_stmt = crate::parser::parse("SELECT * FROM test").unwrap();
        let result = service.execute(select_stmt).unwrap();
        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_query_service_handle() {
        let handle: QueryServiceHandle = Arc::new(LocalQueryService::new());
        assert!(handle.is_ready());
    }
}
