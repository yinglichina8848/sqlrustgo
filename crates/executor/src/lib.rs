//! SQLRustGo Executor Module
//!
//! This module provides query execution interfaces.

pub mod executor;

pub use executor::{Executor, ExecutorResult};

/// Execution result for high-level operations
#[derive(Debug)]
pub struct ExecutionResult {
    pub rows_affected: u64,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<sqlrustgo_types::Value>>,
}
