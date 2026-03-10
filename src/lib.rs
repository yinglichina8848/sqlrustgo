//! SQLRustGo Database System Library
//!
//! A Rust implementation of a SQL-92 compliant database system.
//! This crate re-exports functionality from the modular crates/ workspace.

pub use sqlrustgo_executor::{Executor, ExecutorResult};
pub use sqlrustgo_optimizer::Optimizer as QueryOptimizer;
pub use sqlrustgo_parser::lexer::tokenize;
pub use sqlrustgo_parser::{parse, Lexer, Statement, Token};
pub use sqlrustgo_planner::{LogicalPlan, Optimizer, PhysicalPlan, Planner};
pub use sqlrustgo_storage::{BPlusTree, BufferPool, FileStorage, Page, StorageEngine};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};

#[derive(Debug)]
pub struct ExecutionEngine {
    _private: (),
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self { _private: () }
    }

    pub fn execute(&mut self, _statement: Statement) -> Result<ExecutorResult, SqlError> {
        Ok(ExecutorResult::empty())
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
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
}
