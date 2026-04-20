//! SQLRustGo Database System Library
//!
//! A Rust implementation of a SQL-92 compliant database system.
//! This crate re-exports functionality from the modular crates/ workspace.

#![allow(
    unused_mut,
    clippy::unused_enumerate_index,
    clippy::needless_borrow,
    renamed_and_removed_lints
)]

pub mod execution_engine;
pub mod planner;

pub use sqlrustgo_executor::{Executor, ExecutorResult};
pub use sqlrustgo_optimizer::Optimizer as QueryOptimizer;
pub use sqlrustgo_parser::lexer::tokenize;
pub use sqlrustgo_parser::{parse, Lexer, Statement, Token};
pub use sqlrustgo_planner::{
    AggregateExec, Column, DataType, DeleteExec, Expr, Field, FilterExec, HashJoinExec,
    IndexScanExec, LimitExec, LogicalPlan, Operator, Optimizer, PhysicalPlan, Planner,
    ProjectionExec, Schema, SeqScanExec, SortExec,
};
pub use sqlrustgo_storage::{
    BPlusTree, BufferPool, FileStorage, MemoryStorage, Page, StorageEngine,
};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};

pub use execution_engine::{ExecutionEngine, MemoryExecutionEngine};

/// Initialize the database system
pub fn init() {
    println!("SQLRustGo Database System initialized");
}
