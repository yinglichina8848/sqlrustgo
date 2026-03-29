//! ParquetReader - Read Parquet files into Records
//!
//! Uses Arrow's parquet reader to read Parquet files and convert
//! them to SQLRustGo Record format.

use crate::engine::{Record, SqlError, SqlResult};
use arrow::array::{ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::DataType;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ProjectionMask;
use std::fs::File;

/// Read a Parquet file and return records
pub fn read_parquet_file(path: &str, columns: &[String]) -> SqlResult<Vec<Record>> {
    let file = File::open(path)
        .map_err(|e| SqlError::ExecutionError(format!("Failed to open Parquet file: {}", e)))?;

    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| SqlError::ExecutionError(format!("Failed to create Parquet reader: {}", e)))?;

    let record_batch_reader = if columns.is_empty() {
        // Read all columns
        builder.with_batch_size(1024).build().map_err(|e| {
            SqlError::ExecutionError(format!("Failed to build record reader: {}", e))
        })?
    } else {
        // Project only specified columns
        let parquet_schema = builder.parquet_schema();
        let field_indices: Vec<usize> = columns
            .iter()
            .filter_map(|col| {
                parquet_schema
                    .columns()
                    .iter()
                    .position(|c| c.name() == col)
            })
            .collect();

        if field_indices.is_empty() {
            return Err(SqlError::ExecutionError(format!(
                "No matching columns found: {:?}",
                columns
            )));
        }

        let mask = ProjectionMask::leaves(parquet_schema, field_indices.iter().copied());
        builder
            .with_projection(mask)
            .with_batch_size(1024)
            .build()
            .map_err(|e| {
                SqlError::ExecutionError(format!("Failed to build record reader: {}", e))
            })?
    };

    let mut records = Vec::new();

    for batch_result in record_batch_reader {
        let batch = batch_result
            .map_err(|e| SqlError::ExecutionError(format!("Failed to read batch: {}", e)))?;

        let num_rows = batch.num_rows();
        if num_rows == 0 {
            continue;
        }

        let arrays = batch.columns().to_vec();

        // Convert each row to a Record
        for row_idx in 0..num_rows {
            let mut record = Record::new();
            for arr in &arrays {
                let value = extract_value(arr, row_idx)?;
                record.push(value);
            }
            records.push(record);
        }
    }

    Ok(records)
}

/// Extract a Value from an Arrow array at a specific row index
fn extract_value(array: &ArrayRef, row_idx: usize) -> SqlResult<sqlrustgo_types::Value> {
    use sqlrustgo_types::Value;

    if array.is_null(row_idx) {
        return Ok(Value::Null);
    }

    match array.data_type() {
        DataType::Int64 => {
            let arr = array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                SqlError::ExecutionError("Failed to downcast to Int64Array".to_string())
            })?;
            Ok(Value::Integer(arr.value(row_idx)))
        }
        DataType::Float64 => {
            let arr = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| {
                    SqlError::ExecutionError("Failed to downcast to Float64Array".to_string())
                })?;
            Ok(Value::Float(arr.value(row_idx)))
        }
        DataType::Utf8 => {
            let arr = array
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    SqlError::ExecutionError("Failed to downcast to StringArray".to_string())
                })?;
            Ok(Value::Text(arr.value(row_idx).to_string()))
        }
        DataType::Boolean => {
            let arr = array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| {
                    SqlError::ExecutionError("Failed to downcast to BooleanArray".to_string())
                })?;
            Ok(Value::Boolean(arr.value(row_idx)))
        }
        DataType::Binary => {
            let arr = array
                .as_any()
                .downcast_ref::<arrow::array::BinaryArray>()
                .ok_or_else(|| {
                    SqlError::ExecutionError("Failed to downcast to BinaryArray".to_string())
                })?;
            Ok(Value::Blob(arr.value(row_idx).to_vec()))
        }
        DataType::Date32 => {
            let arr = array
                .as_any()
                .downcast_ref::<arrow::array::Date32Array>()
                .ok_or_else(|| {
                    SqlError::ExecutionError("Failed to downcast to Date32Array".to_string())
                })?;
            Ok(Value::Date(arr.value(row_idx)))
        }
        _ => Err(SqlError::ExecutionError(format!(
            "Unsupported Parquet type: {:?}",
            array.data_type()
        ))),
    }
}

/// ParquetReader struct for reading Parquet files
pub struct ParquetReader {
    path: String,
    columns: Vec<String>,
}

impl ParquetReader {
    /// Create a new ParquetReader
    pub fn new(path: &str, columns: Vec<String>) -> Self {
        Self {
            path: path.to_string(),
            columns,
        }
    }

    /// Read all records from the Parquet file
    pub fn read(&self) -> SqlResult<Vec<Record>> {
        read_parquet_file(&self.path, &self.columns)
    }

    /// Get the number of rows without reading all data
    pub fn row_count(&self) -> SqlResult<usize> {
        let file = File::open(&self.path)
            .map_err(|e| SqlError::ExecutionError(format!("Failed to open Parquet file: {}", e)))?;

        let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(|e| {
            SqlError::ExecutionError(format!("Failed to create Parquet reader: {}", e))
        })?;

        Ok(builder.metadata().file_metadata().num_rows() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_read_parquet_empty() {
        // This test requires an actual parquet file
        // In real usage, we first write a file then read it
    }
}
