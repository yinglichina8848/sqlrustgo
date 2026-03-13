//! Volcano Model Executor Trait
//!
//! This module defines the unified Executor trait following the Volcano iterator model.
//! Each executor implements the `Executor` trait with a `next()` method that returns
//! `Option<RecordBatch>` - the next batch of rows or None when exhausted.

use crate::types::{SqlResult, Value};
use std::sync::Arc;

/// RecordBatch represents a batch of rows returned by an executor
#[derive(Debug, Clone)]
pub struct RecordBatch {
    /// Column names
    pub columns: Vec<String>,
    /// Row data (inner vec is a row, outer vec is all rows in batch)
    pub rows: Vec<Vec<Value>>,
    /// Number of rows in this batch
    pub num_rows: usize,
}

impl RecordBatch {
    /// Create a new RecordBatch
    pub fn new(columns: Vec<String>, rows: Vec<Vec<Value>>) -> Self {
        let num_rows = rows.len();
        Self {
            columns,
            rows,
            num_rows,
        }
    }

    /// Create an empty RecordBatch
    pub fn empty() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
            num_rows: 0,
        }
    }
}

/// Executor trait - unified interface for all Volcano model operators
///
/// # Design
/// - Each operator implements this trait
/// - `next()` returns `Option<RecordBatch>` - next batch of rows or None
/// - Operators are composed by nesting: child executors are inputs
///
/// # Example
/// ```ignore
/// struct TableScanExecutor {
///     table: String,
///     child: Box<dyn Executor>,
/// }
///
/// impl Executor for TableScanExecutor {
///     fn next(&mut self) -> SqlResult<Option<RecordBatch>> {
///         // Scan table and return batches
///     }
/// }
/// ```
pub trait Executor: Send {
    /// Get the next batch of records
    ///
    /// Returns:
    /// - `Ok(Some(batch))` - next batch of rows
    /// - `Ok(None)` - no more data
    /// - `Err(e)` - execution error
    fn next(&mut self) -> SqlResult<Option<RecordBatch>>;

    /// Get the schema (column names) of this executor's output
    fn schema(&self) -> &[String];

    /// Optional: Initialize the executor (called before first next())
    /// Default implementation does nothing
    fn init(&mut self) -> SqlResult<()> {
        Ok(())
    }

    /// Optional: Close the executor and release resources
    /// Default implementation does nothing
    fn close(&mut self) -> SqlResult<()> {
        Ok(())
    }
}

/// Dynamic executor for runtime polymorphism
pub type BoxExecutor = Box<dyn Executor>;

/// Executor that produces no rows (used for NULL input or empty result)
pub struct NullExecutor {
    schema: Vec<String>,
}

impl NullExecutor {
    pub fn new(schema: Vec<String>) -> Self {
        Self { schema }
    }
}

impl Executor for NullExecutor {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>> {
        Ok(None)
    }

    fn schema(&self) -> &[String] {
        &self.schema
    }
}

/// Executor that produces a single batch then terminates
pub struct OnceExecutor {
    batch: Option<RecordBatch>,
}

impl OnceExecutor {
    pub fn new(batch: RecordBatch) -> Self {
        Self {
            batch: Some(batch),
        }
    }
}

impl Executor for OnceExecutor {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>> {
        Ok(self.batch.take())
    }

    fn schema(&self) -> &[String] {
        self.batch
            .as_ref()
            .map(|b| b.columns.as_slice())
            .unwrap_or(&[])
    }
}

/// Empty executor with no schema
pub struct EmptyExecutor;

impl EmptyExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Executor for EmptyExecutor {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>> {
        Ok(Some(RecordBatch::empty()))
    }

    fn schema(&self) -> &[String] {
        &[]
    }
}

// Re-export common types
pub use crate::executor::ExecutionEngine;
