//! ParquetWriter - Write Records to Parquet files
//!
//! Uses Arrow's parquet writer to convert Records to Arrow arrays
//! and write them to Parquet format.

use crate::engine::{Record, SqlError, SqlResult};
use arrow::array::{ArrayRef, PrimitiveBuilder, StringBuilder};
use arrow::datatypes::{DataType, Date32Type, Field, Float64Type, Int64Type, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::sync::Arc;

// Import type aliases from arrow
use arrow::array::{BinaryBuilder, BooleanBuilder};

/// Write records to a Parquet file
pub fn write_parquet_file(
    path: &str,
    records: &[Record],
    column_names: &[String],
) -> SqlResult<()> {
    if records.is_empty() {
        return Err(SqlError::ExecutionError(
            "Cannot write empty records to Parquet".to_string(),
        ));
    }

    if column_names.is_empty() {
        return Err(SqlError::ExecutionError(
            "Column names required for Parquet export".to_string(),
        ));
    }

    let num_columns = records[0].len();
    if num_columns != column_names.len() {
        return Err(SqlError::ExecutionError(format!(
            "Column count mismatch: {} values but {} column names",
            num_columns,
            column_names.len()
        )));
    }

    // Build Arrow schema from column names and inferred types
    let schema = infer_schema(records, column_names)?;

    // Convert records to Arrow arrays
    let arrays = records_to_arrays(records, &schema)?;

    // Create the Parquet writer
    let file = File::create(path)
        .map_err(|e| SqlError::ExecutionError(format!("Failed to create Parquet file: {}", e)))?;

    let props = WriterProperties::builder().build();

    let mut writer = ArrowWriter::try_new(file, Arc::new(schema.clone()), Some(props))
        .map_err(|e| SqlError::ExecutionError(format!("Failed to create Arrow writer: {}", e)))?;

    // Write batch
    let batch = RecordBatch::try_new(Arc::new(schema), arrays)
        .map_err(|e| SqlError::ExecutionError(format!("Failed to create record batch: {}", e)))?;

    writer
        .write(&batch)
        .map_err(|e| SqlError::ExecutionError(format!("Failed to write batch: {}", e)))?;

    writer
        .close()
        .map_err(|e| SqlError::ExecutionError(format!("Failed to close writer: {}", e)))?;

    Ok(())
}

/// Infer Arrow schema from records
fn infer_schema(records: &[Record], column_names: &[String]) -> SqlResult<Schema> {
    use sqlrustgo_types::Value;

    if records.is_empty() {
        return Err(SqlError::ExecutionError(
            "Cannot infer schema from empty records".to_string(),
        ));
    }

    let mut fields = Vec::new();

    for (idx, col_name) in column_names.iter().enumerate() {
        // Infer type from first non-null value
        let mut data_type = DataType::Utf8; // Default to string

        for record in records {
            if idx < record.len() {
                match &record[idx] {
                    Value::Null => continue,
                    Value::Integer(_) => {
                        data_type = DataType::Int64;
                        break;
                    }
                    Value::Float(_) => {
                        data_type = DataType::Float64;
                        break;
                    }
                    Value::Boolean(_) => {
                        data_type = DataType::Boolean;
                        break;
                    }
                    Value::Text(_) => {
                        data_type = DataType::Utf8;
                        break;
                    }
                    Value::Blob(_) => {
                        data_type = DataType::Binary;
                        break;
                    }
                    Value::Date(_) => {
                        data_type = DataType::Date32;
                        break;
                    }
                    Value::Timestamp(_) => {
                        data_type = DataType::Int64;
                        break;
                    }
                    Value::Uuid(_) => {
                        data_type = DataType::FixedSizeBinary(16);
                        break;
                    }
                    Value::Array(_) => {
                        data_type = DataType::Utf8;
                        break;
                    }
                    Value::Enum(_, _) => {
                        data_type = DataType::Utf8;
                        break;
                    }
                }
            }
        }

        fields.push(Field::new(col_name, data_type, true));
    }

    Ok(Schema::new(fields))
}

/// Convert records to Arrow arrays
fn records_to_arrays(records: &[Record], schema: &Schema) -> SqlResult<Vec<ArrayRef>> {
    use sqlrustgo_types::Value;

    let mut arrays: Vec<ArrayRef> = Vec::new();

    for (idx, field) in schema.fields().iter().enumerate() {
        let arr: ArrayRef = match field.data_type() {
            DataType::Int64 => {
                let mut builder: PrimitiveBuilder<Int64Type> = PrimitiveBuilder::new();
                for record in records {
                    if idx < record.len() {
                        match &record[idx] {
                            Value::Null => builder.append_null(),
                            Value::Integer(v) => builder.append_value(*v),
                            _ => builder.append_null(), // Type mismatch
                        }
                    } else {
                        builder.append_null();
                    }
                }
                Arc::new(builder.finish()) as ArrayRef
            }
            DataType::Float64 => {
                let mut builder: PrimitiveBuilder<Float64Type> = PrimitiveBuilder::new();
                for record in records {
                    if idx < record.len() {
                        match &record[idx] {
                            Value::Null => builder.append_null(),
                            Value::Float(v) => builder.append_value(*v),
                            Value::Integer(v) => builder.append_value(*v as f64),
                            _ => builder.append_null(),
                        }
                    } else {
                        builder.append_null();
                    }
                }
                Arc::new(builder.finish()) as ArrayRef
            }
            DataType::Boolean => {
                let mut builder = BooleanBuilder::new();
                for record in records {
                    if idx < record.len() {
                        match &record[idx] {
                            Value::Null => builder.append_null(),
                            Value::Boolean(v) => builder.append_value(*v),
                            _ => builder.append_null(),
                        }
                    } else {
                        builder.append_null();
                    }
                }
                Arc::new(builder.finish()) as ArrayRef
            }
            DataType::Utf8 => {
                let mut builder = StringBuilder::new();
                for record in records {
                    if idx < record.len() {
                        match &record[idx] {
                            Value::Null => builder.append_null(),
                            Value::Text(s) => builder.append_value(s),
                            Value::Integer(i) => builder.append_value(i.to_string()),
                            Value::Float(f) => builder.append_value(f.to_string()),
                            Value::Boolean(b) => builder.append_value(b.to_string()),
                            _ => builder.append_null(),
                        }
                    } else {
                        builder.append_null();
                    }
                }
                Arc::new(builder.finish()) as ArrayRef
            }
            DataType::Binary => {
                let mut builder = BinaryBuilder::new();
                for record in records {
                    if idx < record.len() {
                        match &record[idx] {
                            Value::Null => builder.append_null(),
                            Value::Blob(v) => builder.append_value(v.as_slice()),
                            _ => builder.append_null(),
                        }
                    } else {
                        builder.append_null();
                    }
                }
                Arc::new(builder.finish()) as ArrayRef
            }
            DataType::Date32 => {
                let mut builder: PrimitiveBuilder<Date32Type> = PrimitiveBuilder::new();
                for record in records {
                    if idx < record.len() {
                        match &record[idx] {
                            Value::Null => builder.append_null(),
                            Value::Date(d) => builder.append_value(*d),
                            _ => builder.append_null(),
                        }
                    } else {
                        builder.append_null();
                    }
                }
                Arc::new(builder.finish()) as ArrayRef
            }
            _ => {
                return Err(SqlError::ExecutionError(format!(
                    "Unsupported Arrow type for export: {:?}",
                    field.data_type()
                )));
            }
        };
        arrays.push(arr);
    }

    Ok(arrays)
}

/// ParquetWriter struct for writing Parquet files
pub struct ParquetWriter {
    path: String,
    column_names: Vec<String>,
}

impl ParquetWriter {
    /// Create a new ParquetWriter
    pub fn new(path: &str, column_names: Vec<String>) -> Self {
        Self {
            path: path.to_string(),
            column_names,
        }
    }

    /// Write records to the Parquet file
    pub fn write(&self, records: &[Record]) -> SqlResult<()> {
        write_parquet_file(&self.path, records, &self.column_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_parquet_empty_records() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.parquet");

        let result = write_parquet_file(path.to_str().unwrap(), &[], &["id".to_string()]);
        assert!(result.is_err());
    }
}
