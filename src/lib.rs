//! SQLRustGo Database System Library
//!
//! A Rust implementation of a SQL-92 compliant database system.
//! This crate re-exports functionality from the modular crates/ workspace.

pub use sqlrustgo_executor::{Executor, ExecutorResult};
pub use sqlrustgo_optimizer::Optimizer as QueryOptimizer;
pub use sqlrustgo_parser::lexer::tokenize;
pub use sqlrustgo_parser::{parse, Lexer, Statement, Token};
pub use sqlrustgo_planner::{LogicalPlan, Optimizer, PhysicalPlan, Planner};
use std::sync::Arc;

pub use sqlrustgo_storage::{
    BPlusTree, BufferPool, FileStorage, MemoryStorage, Page, StorageEngine,
};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};

pub struct ExecutionEngine {
    storage: Arc<dyn StorageEngine>,
}

impl ExecutionEngine {
    pub fn new(storage: Arc<dyn StorageEngine>) -> Self {
        Self { storage }
    }

    pub fn execute(&mut self, _statement: Statement) -> Result<ExecutorResult, SqlError> {
        Ok(ExecutorResult::empty())
    }

    pub fn execute_plan(&self, plan: &dyn PhysicalPlan) -> Result<ExecutorResult, SqlError> {
        match plan.name() {
            "SeqScan" => {
                let rows = self.storage.scan(plan.table_name())?;
                Ok(ExecutorResult::new(rows, 0))
            }
            "Filter" => {
                let filter_plan = plan
                    .as_any()
                    .downcast_ref::<sqlrustgo_planner::FilterExec>()
                    .ok_or_else(|| {
                        SqlError::ExecutionError("Failed to downcast FilterExec".to_string())
                    })?;

                let child = filter_plan.input();
                let mut input_result = self.execute_plan(child)?;

                let predicate = filter_plan.predicate();
                let schema = child.schema();
                let filtered_rows: Vec<Vec<Value>> = input_result
                    .rows
                    .into_iter()
                    .filter(|row| predicate.matches(row, schema))
                    .collect();

                Ok(ExecutorResult::new(filtered_rows, 0))
            }
            _ => Ok(ExecutorResult::empty()),
        }
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self {
            storage: Arc::new(MemoryStorage::new()),
        }
    }
}

/// Initialize the database system
pub fn init() {
    println!("SQLRustGo Database System initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
    }

    #[test]
    fn test_module_exports() {
        let _ = tokenize("SELECT 1");
        let _ = parse("SELECT 1");
        let _ = Value::Integer(1);
    }

    #[test]
    #[allow(clippy::unnecessary_literal_unwrap)]
    fn test_sql_result_alias() {
        let result: SqlResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_optimizer_alias() {
        let _: Option<Box<dyn sqlrustgo_optimizer::Optimizer>> = None;
    }

    #[test]
    fn test_physical_plan_trait() {
        let _: Option<Box<dyn PhysicalPlan>> = None;
    }

    #[test]
    fn test_execution_engine_new() {
        let mut engine = ExecutionEngine::default();
        let stmt = sqlrustgo_parser::parse("SELECT * FROM users").unwrap();
        assert_eq!(engine.execute(stmt).unwrap().rows.len(), 0);
    }

    #[test]
    fn test_execution_engine_default() {
        let mut engine = ExecutionEngine::default();
        let stmt = sqlrustgo_parser::parse("SELECT * FROM users").unwrap();
        assert_eq!(engine.execute(stmt).unwrap().affected_rows, 0);
    }

    #[test]
    fn test_execute_plan_seqscan() {
        use sqlrustgo_planner::{DataType, Field, Schema, SeqScanExec};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                ],
            )
            .unwrap();

        let engine = ExecutionEngine::new(Arc::new(storage));
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let plan = SeqScanExec::new("users".to_string(), schema);
        let result = engine.execute_plan(&plan).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], Value::Integer(1));
        assert_eq!(result.rows[0][1], Value::Text("Alice".to_string()));
    }

    #[test]
    fn test_execute_plan_filter() {
        use sqlrustgo_planner::{DataType, Expr, Field, FilterExec, Operator, Schema, SeqScanExec};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                    vec![Value::Integer(3), Value::Text("Charlie".to_string())],
                ],
            )
            .unwrap();

        let engine = ExecutionEngine::new(Arc::new(storage));
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let scan = SeqScanExec::new("users".to_string(), schema.clone());
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            Operator::Gt,
            Expr::literal(Value::Integer(1)),
        );
        let filter = FilterExec::new(Box::new(scan), predicate);
        let result = engine.execute_plan(&filter).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], Value::Integer(2));
    }

    #[test]
    fn test_storage_engine_export() {
        let _: Option<Box<dyn StorageEngine>> = None;
    }

    #[test]
    fn test_executor_export() {
        let _: Option<Box<dyn Executor>> = None;
    }

    #[test]
    fn test_planner_export() {
        let _: Option<Box<dyn Planner>> = None;
    }
}
