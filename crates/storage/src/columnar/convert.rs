//! Conversion utilities between storage types
//!
//! This module provides utilities to convert between storage columnar formats.
//! For integration with the executor, use the executor's conversion utilities.

use crate::ColumnChunk;
use sqlrustgo_types::Value;

/// Represents columnar data for vectorized operations
/// This is a simplified version that storage returns, and executor converts as needed
#[derive(Debug, Clone)]
pub enum StorageColumnArray {
    Int64(Vec<i64>),
    Float64(Vec<f64>),
    Boolean(Vec<bool>),
    Text(Vec<String>),
    Null,
}

impl StorageColumnArray {
    pub fn len(&self) -> usize {
        match self {
            StorageColumnArray::Int64(v) => v.len(),
            StorageColumnArray::Float64(v) => v.len(),
            StorageColumnArray::Boolean(v) => v.len(),
            StorageColumnArray::Text(v) => v.len(),
            StorageColumnArray::Null => 0,
        }
    }
}

/// Extension trait for converting ColumnChunk to StorageColumnArray
pub trait IntoStorageColumnArray {
    fn into_storage_column_array(self) -> StorageColumnArray;
}

impl IntoStorageColumnArray for ColumnChunk {
    fn into_storage_column_array(self) -> StorageColumnArray {
        let values = self.data();

        // First pass: determine if all values are the same type
        let mut int64_count = 0;
        let mut float64_count = 0;
        let mut bool_count = 0;
        let mut text_count = 0;
        let mut null_count = 0;

        for v in &values {
            match v {
                Value::Integer(_) => int64_count += 1,
                Value::Float(_) => float64_count += 1,
                Value::Boolean(_) => bool_count += 1,
                Value::Text(_) => text_count += 1,
                Value::Null => null_count += 1,
                _ => {}
            }
        }

        // Determine the dominant type
        let total = values.len();
        if total == 0 {
            return StorageColumnArray::Null;
        }

        if int64_count == total {
            let arr: Vec<i64> = values
                .iter()
                .filter_map(|v| {
                    if let Value::Integer(n) = v {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .collect();
            StorageColumnArray::Int64(arr)
        } else if float64_count == total {
            let arr: Vec<f64> = values
                .iter()
                .filter_map(|v| {
                    if let Value::Float(n) = v {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .collect();
            StorageColumnArray::Float64(arr)
        } else if bool_count == total {
            let arr: Vec<bool> = values
                .iter()
                .filter_map(|v| {
                    if let Value::Boolean(b) = v {
                        Some(*b)
                    } else {
                        None
                    }
                })
                .collect();
            StorageColumnArray::Boolean(arr)
        } else if text_count == total {
            let arr: Vec<String> = values
                .iter()
                .filter_map(|v| {
                    if let Value::Text(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect();
            StorageColumnArray::Text(arr)
        } else if null_count == total {
            StorageColumnArray::Null
        } else {
            // Mixed types - return as Int64 if most are integers, otherwise Null
            if int64_count >= float64_count
                && int64_count >= bool_count
                && int64_count >= text_count
            {
                let arr: Vec<i64> = values
                    .iter()
                    .filter_map(|v| {
                        if let Value::Integer(n) = v {
                            Some(*n)
                        } else {
                            Some(0) // Default for non-integer values
                        }
                    })
                    .collect();
                StorageColumnArray::Int64(arr)
            } else {
                StorageColumnArray::Null
            }
        }
    }
}

/// Convert a single Value to StorageColumnArray variant
impl From<Value> for StorageColumnArray {
    fn from(value: Value) -> Self {
        match value {
            Value::Integer(n) => StorageColumnArray::Int64(vec![n]),
            Value::Float(n) => StorageColumnArray::Float64(vec![n]),
            Value::Boolean(b) => StorageColumnArray::Boolean(vec![b]),
            Value::Text(s) => StorageColumnArray::Text(vec![s]),
            Value::Null => StorageColumnArray::Null,
            _ => StorageColumnArray::Null,
        }
    }
}
