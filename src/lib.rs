//! SQLRustGo Database System Library
//!
//! A Rust implementation of a SQL-92 compliant database system.
//! This crate re-exports functionality from the modular crates/ workspace.

pub use sqlrustgo_executor::{Executor, ExecutorResult, LocalExecutor};
pub use sqlrustgo_optimizer::Optimizer as QueryOptimizer;
pub use sqlrustgo_parser::{parse, tokenize, Lexer, Statement, Token};
pub use sqlrustgo_planner::{LogicalPlan, Optimizer, PhysicalPlan, Planner};
pub use sqlrustgo_storage::{BPlusTree, BufferPool, FileStorage, Page, StorageEngine};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};

#[allow(dead_code)]
pub struct ExecutionEngine {
    executor: LocalExecutor,
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self {
            executor: LocalExecutor::new(),
        }
    }

    pub fn execute(&mut self, _statement: Statement) -> SqlResult<ExecutorResult> {
        Ok(ExecutorResult::empty())
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub fn init() {
    println!("SQLRustGo Database System initialized");
}
