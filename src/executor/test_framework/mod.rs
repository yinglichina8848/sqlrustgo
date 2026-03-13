//! Executor Test Suite
//!
//! This module provides testing utilities for the executor including:
//! - Mock storage for testing without file I/O
//! - Test data generators
//! - Test harness for common test scenarios

pub mod mock_storage;
pub mod test_data;
pub mod harness;
pub mod examples;

pub use mock_storage::MockStorage;
pub use test_data::{TestDataGenerator, TestTableBuilder, schemas};
pub use harness::{TestHarness, TestFixture, assertions};

// Re-export commonly used types
pub use crate::executor::{ExecutionEngine, ExecutionResult, TableData, TableInfo};
pub use crate::types::Value;
