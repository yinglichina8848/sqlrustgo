//! Volcano Model Executor Trait
//!
//! This module defines the unified Executor trait following the Volcano iterator model.

use crate::types::{SqlResult, Value};

/// RecordBatch represents a batch of rows returned by an executor
#[derive(Debug, Clone)]
pub struct RecordBatch {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub num_rows: usize,
}

impl RecordBatch {
    pub fn new(columns: Vec<String>, rows: Vec<Vec<Value>>) -> Self {
        let num_rows = rows.len();
        Self { columns, rows, num_rows }
    }

    pub fn empty() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
            num_rows: 0,
        }
    }
}

/// Executor trait - unified interface for all Volcano model operators
pub trait Executor: Send {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>>;
    fn schema(&self) -> &[String];
    fn init(&mut self) -> SqlResult<()> { Ok(()) }
    fn close(&mut self) -> SqlResult<()> { Ok(()) }
}

pub type BoxExecutor = Box<dyn Executor>;

/// Executor that produces no rows
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
        Self { batch: Some(batch) }
    }
}

impl Executor for OnceExecutor {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>> {
        Ok(self.batch.take())
    }
    fn schema(&self) -> &[String] {
        self.batch.as_ref().map(|b| b.columns.as_slice()).unwrap_or(&[])
    }
}

pub use crate::executor::ExecutionEngine;
