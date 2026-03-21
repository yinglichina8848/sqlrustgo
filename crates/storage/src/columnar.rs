//! Columnar Storage Module
//!
//! Provides columnar storage format and Parquet file format support
//! for analytical query optimization.

use crate::engine::{ColumnDefinition, Record, TableInfo};
use parquet::arrow::{ArrowWriter, ParquetFileArrowReader};
use parquet::file::reader::SerializedFileReader;
use parquet::file::writer::ParquetFileWriter;
use parquet::schema::parser::parse_message_type;
use serde::{Deserialize, Serialize};
use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Columnar storage format for analytical queries
/// Stores data column-by-column for better compression and scan performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnarTable {
    /// Table metadata
    pub info: TableInfo,
    /// Column data stored column-wise
    columns: Vec<ColumnarColumn>,
    /// Number of rows
    num_rows: usize,
}

impl ColumnarTable {
    /// Create a new columnar table
    pub fn new(info: TableInfo) -> Self {
        let columns = info
            .columns
            .iter()
            .map(|col| ColumnarColumn::new(col.clone()))
            .collect();
        Self {
            info,
            columns,
            num_rows: 0,
        }
    }

    /// Create from row-based records
    pub fn from_records(info: TableInfo, records: &[Record]) -> Self {
        let mut table = Self::new(info);
        for record in records {
            table.append_row(record);
        }
        table
    }

    /// Append a row to the table
    pub fn append_row(&mut self, record: &Record) {
        for (i, value) in record.iter().enumerate() {
            if i < self.columns.len() {
                self.columns[i].push(value.clone());
            }
        }
        self.num_rows += 1;
    }

    /// Get the number of rows
    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    /// Get the number of columns
    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    /// Get column by index
    pub fn column(&self, index: usize) -> Option<&ColumnarColumn> {
        self.columns.get(index)
    }

    /// Get projected columns (Projection Pushdown)
    pub fn project(&self, column_indices: &[usize]) -> Vec<&ColumnarColumn> {
        column_indices
            .iter()
            .filter_map(|&i| self.columns.get(i))
            .collect()
    }

    /// Convert to row-based records
    pub fn to_records(&self) -> Vec<Record> {
        let mut records = Vec::with_capacity(self.num_rows);
        for row_idx in 0..self.num_rows {
            let mut record = Vec::with_capacity(self.columns.len());
            for col in &self.columns {
                record.push(col.get(row_idx).unwrap_or(Value::Null));
            }
            records.push(record);
        }
        records
    }

    /// Get projected records (Projection Pushdown)
    pub fn project_to_records(&self, column_indices: &[usize]) -> Vec<Record> {
        let projected_cols = self.project(column_indices);
        let mut records = Vec::with_capacity(self.num_rows);
        for row_idx in 0..self.num_rows {
            let mut record = Vec::with_capacity(projected_cols.len());
            for col in &projected_cols {
                record.push(col.get(row_idx).unwrap_or(Value::Null));
            }
            records.push(record);
        }
        records
    }

    /// Get column statistics for query optimization
    pub fn column_stats(&self, column_index: usize) -> Option<ColumnStats> {
        self.columns.get(column_index).map(|col| col.stats())
    }

    /// Filter rows based on a predicate (columnar filter)
    pub fn filter(&self, predicate: &dyn Fn(&[Value]) -> bool) -> Vec<Record> {
        let mut result = Vec::new();
        for row_idx in 0..self.num_rows {
            let row: Vec<Value> = self.columns.iter().map(|col| col.get(row_idx).unwrap_or(Value::Null)).collect();
            if predicate(&row) {
                result.push(row);
            }
        }
        result
    }

    /// Get projected and filtered data (combined optimization)
    pub fn project_filter(
        &self,
        column_indices: &[usize],
        predicate: &dyn Fn(&[Value]) -> bool,
    ) -> Vec<Record> {
        let projected_cols: Vec<_> = self.project(column_indices).into_iter().cloned().collect();
        let mut result = Vec::new();
        for row_idx in 0..self.num_rows {
            let row: Vec<Value> = projected_cols
                .iter()
                .map(|col| col.get(row_idx).unwrap_or(Value::Null))
                .collect();
            if predicate(&row) {
                result.push(row);
            }
        }
        result
    }
}

/// A single column in columnar storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnarColumn {
    /// Column metadata
    pub definition: ColumnDefinition,
    /// Column values stored contiguously
    values: Vec<Value>,
    /// Null bitmap (optional optimization)
    null_bitmap: Option<Vec<bool>>,
}

impl ColumnarColumn {
    /// Create a new column
    pub fn new(definition: ColumnDefinition) -> Self {
        Self {
            definition,
            values: Vec::new(),
            null_bitmap: None,
        }
    }

    /// Append a value
    pub fn push(&mut self, value: Value) {
        if matches!(value, Value::Null) {
            if let Some(ref mut bitmap) = self.null_bitmap {
                bitmap.push(true);
            } else {
                self.null_bitmap = Some(vec![false; self.values.len()]);
                self.null_bitmap.as_mut().unwrap().push(true);
            }
        } else {
            if let Some(ref mut bitmap) = self.null_bitmap {
                bitmap.push(false);
            }
        }
        self.values.push(value);
    }

    /// Get value at index
    pub fn get(&self, index: usize) -> Option<Value> {
        self.values.get(index).cloned()
    }

    /// Get the number of values
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if column is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get column statistics
    pub fn stats(&self) -> ColumnStats {
        let mut min: Option<Value> = None;
        let mut max: Option<Value> = None;
        let mut null_count = 0;
        let mut sum_int: Option<i128> = None;

        for value in &self.values {
            match value {
                Value::Null => null_count += 1,
                Value::Integer(i) => {
                    min = match &min {
                        Some(Value::Integer(m)) if *i < *m => Some(value.clone()),
                        None => Some(value.clone()),
                        _ => min,
                    };
                    max = match &max {
                        Some(Value::Integer(m)) if *i > *m => Some(value.clone()),
                        None => Some(value.clone()),
                        _ => max,
                    };
                    sum_int = Some(sum_int.unwrap_or(0) + *i as i128);
                }
                _ => {}
            }
        }

        ColumnStats {
            column_name: self.definition.name.clone(),
            data_type: self.definition.data_type.clone(),
            num_values: self.values.len(),
            null_count,
            min_value: min,
            max_value: max,
            distinct_count: None,
        }
    }

    /// Get raw values reference
    pub fn values(&self) -> &[Value] {
        &self.values
    }
}

/// Column statistics for query optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStats {
    pub column_name: String,
    pub data_type: String,
    pub num_values: usize,
    pub null_count: usize,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
    pub distinct_count: Option<usize>,
}

impl ColumnStats {
    /// Estimate memory size in bytes
    pub fn estimate_memory_size(&self) -> usize {
        let base = std::mem::size_of::<Self>();
        let name_size = self.column_name.capacity() + self.data_type.capacity();
        let min_max_size = self.min_value.as_ref().map(|v| v.estimate_memory_size()).unwrap_or(0)
            + self.max_value.as_ref().map(|v| v.estimate_memory_size()).unwrap_or(0);
        base + name_size + min_max_size
    }

    /// Check if a value is within the column's range
    pub fn contains_value(&self, value: &Value) -> bool {
        if let Some(min) = &self.min_value {
            if let Some(max) = &self.max_value {
                return value >= min && value <= max;
            }
        }
        true // Unknown range, assume possible
    }

    /// Check if statistics are available
    pub fn is_available(&self) -> bool {
        self.min_value.is_some() && self.max_value.is_some()
    }
}

/// Parquet file reader for columnar storage
pub struct ParquetReader {
    path: String,
    projected_columns: Vec<String>,
}

impl ParquetReader {
    /// Create a new Parquet reader
    pub fn new(path: String) -> Self {
        Self {
            path,
            projected_columns: Vec::new(),
        }
    }

    /// Set projected columns for projection pushdown
    pub fn with_projection(mut self, columns: Vec<String>) -> Self {
        self.projected_columns = columns;
        self
    }

    /// Read all records from Parquet file
    pub fn read(&self) -> SqlResult<Vec<Record>> {
        let file = File::open(&self.path)
            .map_err(|e| SqlError::IoError(format!("Failed to open parquet file: {}", e)))?;
        
        let reader = SerializedFileReader::new(file)
            .map_err(|e| SqlError::ExecutionError(format!("Failed to read parquet: {}", e)))?;
        
        let mut arrow_reader = ParquetFileArrowReader::new(Arc::new(reader));
        let record_batch_reader = arrow_reader.get_record_reader(1024)
            .map_err(|e| SqlError::ExecutionError(format!("Failed to create record reader: {}", e)))?;

        let mut records = Vec::new();
        
        for batch_result in record_batch_reader {
            let batch = batch_result
                .map_err(|e| SqlError::ExecutionError(format!("Failed to read batch: {}", e)))?;
            
            for row_idx in 0..batch.num_rows() {
                let mut record = Vec::new();
                for col_idx in 0..batch.num_columns() {
                    let col = batch.column(col_idx);
                    let value = arrow_array_to_value(col, row_idx)?;
                    record.push(value);
                }
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Read projected records (Projection Pushdown)
    pub fn read_projected(&self, column_indices: &[usize]) -> SqlResult<Vec<Record>> {
        let file = File::open(&self.path)
            .map_err(|e| SqlError::IoError(format!("Failed to open parquet file: {}", e)))?;
        
        let reader = SerializedFileReader::new(file)
            .map_err(|e| SqlError::ExecutionError(format!("Failed to read parquet: {}", e)))?;
        
        let mut arrow_reader = ParquetFileArrowReader::new(Arc::new(reader));
        let record_batch_reader = arrow_reader.get_record_reader(1024)
            .map_err(|e| SqlError::ExecutionError(format!("Failed to create record reader: {}", e)))?;

        let mut records = Vec::new();
        
        for batch_result in record_batch_reader {
            let batch = batch_result
                .map_err(|e| SqlError::ExecutionError(format!("Failed to read batch: {}", e)))?;
            
            for row_idx in 0..batch.num_rows() {
                let mut record = Vec::new();
                for &col_idx in column_indices {
                    if col_idx < batch.num_columns() {
                        let col = batch.column(col_idx);
                        let value = arrow_array_to_value(col, row_idx)?;
                        record.push(value);
                    }
                }
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Read column statistics for query optimization
    pub fn read_stats(&self, column_name: &str) -> SqlResult<Option<ColumnStats>> {
        let file = File::open(&self.path)
            .map_err(|e| SqlError::IoError(format!("Failed to open parquet file: {}", e)))?;
        
        let reader = SerializedFileReader::new(file)
            .map_err(|e| SqlError::ExecutionError(format!("Failed to read parquet: {}", e)))?;

        let metadata = reader.metadata();
        let schema = metadata.file_metadata().schema();
        
        // Find column index by name
        let mut col_idx = None;
        for (i, field) in schema.get_fields().iter().enumerate() {
            if field.name() == column_name {
                col_idx = Some(i);
                break;
            }
        }

        if let Some(idx) = col_idx {
            let row_group_meta = metadata.row_group(0);
            if let Some(col_meta) = row_group_meta.column(idx) {
                return Ok(Some(ColumnStats {
                    column_name: column_name.to_string(),
                    data_type: format!("{:?}", col_meta.physical_type()),
                    num_values: col_meta.num_values() as usize,
                    null_count: col_meta.num_nulls() as usize,
                    min_value: parse_min_value(col_meta.physical_type(), col_meta.statistics().and_then(|s| s.min_bytes())),
                    max_value: parse_max_value(col_meta.physical_type(), col_meta.statistics().and_then(|s| s.max_bytes())),
                    distinct_count: None,
                }));
            }
        }

        Ok(None)
    }
}

/// Convert Arrow array element to Value
fn arrow_array_to_value(array: &arrow::array::ArrayRef, index: usize) -> SqlResult<Value> {
    use arrow::array::*;
    use arrow::datatypes::DataType;

    if array.is_null(index) {
        return Ok(Value::Null);
    }

    match array.data_type() {
        DataType::Int64 => {
            let arr = array.as_any().downcast_ref::<Int64Array>()
                .ok_or_else(|| SqlError::ExecutionError("Failed to downcast Int64".to_string()))?;
            Ok(Value::Integer(arr.value(index)))
        }
        DataType::Float64 => {
            let arr = array.as_any().downcast_ref::<Float64Array>()
                .ok_or_else(|| SqlError::ExecutionError("Failed to downcast Float64".to_string()))?;
            Ok(Value::Float(arr.value(index)))
        }
        DataType::Utf8 => {
            let arr = array.as_any().downcast_ref::<StringArray>()
                .ok_or_else(|| SqlError::ExecutionError("Failed to downcast String".to_string()))?;
            Ok(Value::Text(arr.value(index).to_string()))
        }
        DataType::Boolean => {
            let arr = array.as_any().downcast_ref::<BooleanArray>()
                .ok_or_else(|| SqlError::ExecutionError("Failed to downcast Boolean".to_string()))?;
            Ok(Value::Boolean(arr.value(index)))
        }
        DataType::Binary => {
            let arr = array.as_any().downcast_ref::<BinaryArray>()
                .ok_or_else(|| SqlError::ExecutionError("Failed to downcast Binary".to_string()))?;
            Ok(Value::Blob(arr.value(index).to_vec()))
        }
        DataType::Date32 => {
            let arr = array.as_any().downcast_ref::<Date32Array>()
                .ok_or_else(|| SqlError::ExecutionError("Failed to downcast Date32".to_string()))?;
            Ok(Value::Date(arr.value(index)))
        }
        DataType::Int32 => {
            let arr = array.as_any().downcast_ref::<Int32Array>()
                .ok_or_else(|| SqlError::ExecutionError("Failed to downcast Int32".to_string()))?;
            Ok(Value::Integer(arr.value(index) as i64))
        }
        _ => Ok(Value::Null),
    }
}

/// Parse min value from parquet statistics
fn parse_min_value(physical_type: parquet::basic::Type, data: Option<&[u8]>) -> Option<Value> {
    let data = data?;
    match physical_type {
        parquet::basic::Type::INT64 => {
            let val = i64::from_le_bytes(data.try_into().ok()?);
            Some(Value::Integer(val))
        }
        parquet::basic::Type::FLOAT => {
            let val = f64::from_le_bytes(data.try_into().ok()?);
            Some(Value::Float(val))
        }
        parquet::basic::Type::BOOLEAN => {
            Some(Value::Boolean(data[0] != 0))
        }
        _ => None,
    }
}

/// Parse max value from parquet statistics
fn parse_max_value(physical_type: parquet::basic::Type, data: Option<&[u8]>) -> Option<Value> {
    let data = data?;
    match physical_type {
        parquet::basic::Type::INT64 => {
            let val = i64::from_le_bytes(data.try_into().ok()?);
            Some(Value::Integer(val))
        }
        parquet::basic::Type::FLOAT => {
            let val = f64::from_le_bytes(data.try_into().ok()?);
            Some(Value::Float(val))
        }
        parquet::basic::Type::BOOLEAN => {
            Some(Value::Boolean(data[0] != 0))
        }
        _ => None,
    }
}

/// Parquet file writer for columnar storage
pub struct ParquetWriter {
    path: String,
    compression: parquet::basic::Compression,
}

impl ParquetWriter {
    /// Create a new Parquet writer
    pub fn new(path: String) -> Self {
        Self {
            path,
            compression: parquet::basic::Compression::SNAPPY,
        }
    }

    /// Set compression type
    pub fn with_compression(mut self, compression: parquet::basic::Compression) -> Self {
        self.compression = compression;
        self
    }

    /// Write records to Parquet file
    pub fn write(&self, info: &TableInfo, records: &[Record]) -> SqlResult<()> {
        let schema = build_parquet_schema(info);
        let file = File::create(&self.path)
            .map_err(|e| SqlError::IoError(format!("Failed to create parquet file: {}", e)))?;

        let props = parquet::file::properties::WriterProperties::builder()
            .set_compression(self.compression)
            .build();

        let writer = ParquetFileWriter::new(file, schema, props);
        let mut arrow_writer = ArrowWriter::try_new(writer, None)
            .map_err(|e| SqlError::ExecutionError(format!("Failed to create arrow writer: {}", e)))?;

        let batches = build_arrow_batches(info, records);
        for batch in batches {
            arrow_writer.write(&batch)
                .map_err(|e| SqlError::ExecutionError(format!("Failed to write batch: {}", e)))?;
        }

        arrow_writer.close()
            .map_err(|e| SqlError::ExecutionError(format!("Failed to close writer: {}", e)))?;

        Ok(())
    }

    /// Write columnar table to Parquet file
    pub fn write_columnar(&self, table: &ColumnarTable) -> SqlResult<()> {
        let records = table.to_records();
        self.write(&table.info, &records)
    }

    /// Write projected records (Projection Pushdown optimization)
    pub fn write_projected(&self, info: &TableInfo, records: &[Record], column_indices: &[usize]) -> SqlResult<()> {
        // Filter the columns based on projection
        let projected_columns: Vec<_> = info.columns.iter()
            .enumerate()
            .filter(|(i, _)| column_indices.contains(i))
            .map(|(_, c)| c.clone())
            .collect();
        
        let projected_info = TableInfo {
            name: info.name.clone(),
            columns: projected_columns,
        };

        let projected_records: Vec<Record> = records.iter()
            .map(|r| column_indices.iter().filter_map(|&i| r.get(i).cloned()).collect())
            .collect();

        self.write(&projected_info, &projected_records)
    }
}

/// Build Parquet schema from table info
fn build_parquet_schema(info: &TableInfo) -> Arc<parquet::schema::SchemaDescriptor> {
    let fields: Vec<parquet::schema::types::Type> = info.columns.iter()
        .map(|col| {
            let physical_type = match col.data_type.to_uppercase().as_str() {
                "INTEGER" | "BIGINT" => parquet::basic::Type::INT64,
                "FLOAT" | "DOUBLE" | "REAL" => parquet::basic::Type::FLOAT,
                "BOOLEAN" => parquet::basic::Type::BOOLEAN,
                "TEXT" | "VARCHAR" | "CHAR" | "STRING" => parquet::basic::Type::BYTE_ARRAY,
                "BLOB" | "BINARY" => parquet::basic::Type::BYTE_ARRAY,
                "DATE" => parquet::basic::Type::INT32,
                "TIMESTAMP" => parquet::basic::Type::INT64,
                _ => parquet::basic::Type::BYTE_ARRAY,
            };
            parquet::schema::types::Type::primitive_type_builder(&col.name, physical_type)
                .with_repetition(if col.nullable {
                    parquet::basic::Repetition::OPTIONAL
                } else {
                    parquet::basic::Repetition::REQUIRED
                })
                .build()
                .expect("Failed to build parquet type")
        })
        .collect();

    let message_type = parquet::schema::types::GroupType::group_type_builder("record")
        .with_fields(&fields)
        .build()
        .expect("Failed to build group type");

    Arc::new(parquet::schema::SchemaDescriptor::new(Arc::new(message_type)))
}

/// Build Arrow record batches from records
fn build_arrow_batches(info: &TableInfo, records: &[Record]) -> Vec<arrow::record_batch::RecordBatch> {
    use arrow::array::*;
    use arrow::datatypes::DataType;
    use arrow::builder::*;

    let batch_size = 1024.min(records.len().max(1));
    let mut batches = Vec::new();

    // Initialize column arrays
    let mut columns: Vec<Box<dyn ArrayBuilder>> = info.columns.iter()
        .map(|col| {
            match col.data_type.to_uppercase().as_str() {
                "INTEGER" | "BIGINT" | "TIMESTAMP" => Box::new(Int64Builder::with_capacity(batch_size)) as Box<dyn ArrayBuilder>,
                "FLOAT" | "DOUBLE" | "REAL" => Box::new(Float64Builder::with_capacity(batch_size)) as Box<dyn ArrayBuilder>,
                "BOOLEAN" => Box::new(BooleanBuilder::with_capacity(batch_size)) as Box<dyn ArrayBuilder>,
                "TEXT" | "VARCHAR" | "CHAR" | "STRING" => Box::new(StringBuilder::with_capacity(batch_size)) as Box<dyn ArrayBuilder>,
                "BLOB" | "BINARY" => Box::new(BinaryBuilder::with_capacity(batch_size)) as Box<dyn ArrayBuilder>,
                "DATE" => Box::new(Int32Builder::with_capacity(batch_size)) as Box<dyn ArrayBuilder>,
                _ => Box::new(StringBuilder::with_capacity(batch_size)) as Box<dyn ArrayBuilder>,
            }
        })
        .collect();

    for record in records {
        for (i, value) in record.iter().enumerate() {
            if let Some(builder) = columns.get_mut(i) {
                append_value_to_builder(builder, value);
            }
        }
    }

    // Build arrays and create batches
    let num_rows = if let Some(first) = records.first() {
        first.len()
    } else {
        0
    };

    let arrow_arrays: Vec<Arc<dyn Array>> = columns.iter_mut()
        .map(|b| b.finish())
        .map(|a| Arc::new(a) as Arc<dyn Array>)
        .collect();

    let schema = arrow::datatypes::Schema::new(
        info.columns.iter().map(|c| {
            arrow::datatypes::Field::new(&c.name, sql_type_to_arrow_type(&c.data_type), c.nullable)
        }).collect()
    );

    if let Ok(batch) = arrow::record_batch::RecordBatch::new(Arc::new(schema), arrow_arrays) {
        batches.push(batch);
    }

    batches
}

/// Append value to array builder
fn append_value_to_builder(builder: &mut Box<dyn ArrayBuilder>, value: &Value) {
    use arrow::array::*;

    match (builder.as_mut(), value) {
        (builder @ &mut Int64Builder { .. }, Value::Integer(v)) => {
            let b = builder.as_any_mut().downcast_mut::<Int64Builder>().unwrap();
            b.append_value(*v);
        }
        (builder @ &mut Float64Builder { .. }, Value::Float(v)) => {
            let b = builder.as_any_mut().downcast_mut::<Float64Builder>().unwrap();
            b.append_value(*v);
        }
        (builder @ &mut BooleanBuilder { .. }, Value::Boolean(v)) => {
            let b = builder.as_any_mut().downcast_mut::<BooleanBuilder>().unwrap();
            b.append_value(*v);
        }
        (builder @ &mut StringBuilder { .. }, Value::Text(v)) => {
            let b = builder.as_any_mut().downcast_mut::<StringBuilder>().unwrap();
            b.append_value(v);
        }
        (builder @ &mut StringBuilder { .. }, Value::Integer(v)) => {
            let b = builder.as_any_mut().downcast_mut::<StringBuilder>().unwrap();
            b.append_value(&v.to_string());
        }
        (builder @ &mut BinaryBuilder { .. }, Value::Blob(v)) => {
            let b = builder.as_any_mut().downcast_mut::<BinaryBuilder>().unwrap();
            b.append_value(v);
        }
        (builder @ &mut Int32Builder { .. }, Value::Date(v)) => {
            let b = builder.as_any_mut().downcast_mut::<Int32Builder>().unwrap();
            b.append_value(*v);
        }
        _ => {
            // For null values or type mismatches, append null
            if let Some(builder) = builder.as_any_mut().downcast_mut::<Int64Builder>() {
                builder.append_null();
            } else if let Some(builder) = builder.as_any_mut().downcast_mut::<Float64Builder>() {
                builder.append_null();
            } else if let Some(builder) = builder.as_any_mut().downcast_mut::<BooleanBuilder>() {
                builder.append_null();
            } else if let Some(builder) = builder.as_any_mut().downcast_mut::<StringBuilder>() {
                builder.append_null();
            } else if let Some(builder) = builder.as_any_mut().downcast_mut::<BinaryBuilder>() {
                builder.append_null();
            } else if let Some(builder) = builder.as_any_mut().downcast_mut::<Int32Builder>() {
                builder.append_null();
            }
        }
    }
}

/// Convert SQL type to Arrow data type
fn sql_type_to_arrow_type(sql_type: &str) -> arrow::datatypes::DataType {
    match sql_type.to_uppercase().as_str() {
        "INTEGER" | "BIGINT" => arrow::datatypes::DataType::Int64,
        "FLOAT" | "DOUBLE" | "REAL" => arrow::datatypes::DataType::Float64,
        "BOOLEAN" => arrow::datatypes::DataType::Boolean,
        "TEXT" | "VARCHAR" | "CHAR" | "STRING" => arrow::datatypes::DataType::Utf8,
        "BLOB" | "BINARY" => arrow::datatypes::DataType::Binary,
        "DATE" => arrow::datatypes::DataType::Date32,
        "TIMESTAMP" => arrow::datatypes::DataType::Int64,
        _ => arrow::datatypes::DataType::Utf8,
    }
}

/// Columnar storage manager for handling multiple tables
pub struct ColumnarStorage {
    /// In-memory columnar tables
    tables: HashMap<String, ColumnarTable>,
    /// Parquet file paths
    parquet_files: HashMap<String, String>,
}

impl ColumnarStorage {
    /// Create a new columnar storage manager
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            parquet_files: HashMap::new(),
        }
    }

    /// Insert records into a columnar table
    pub fn insert(&mut self, table_name: &str, records: Vec<Record>) -> SqlResult<()> {
        if let Some(table) = self.tables.get_mut(table_name) {
            for record in records {
                table.append_row(&record);
            }
            Ok(())
        } else {
            Err(SqlError::TableNotFound(table_name.to_string()))
        }
    }

    /// Create a new columnar table
    pub fn create_table(&mut self, info: TableInfo) -> SqlResult<()> {
        if self.tables.contains_key(&info.name) {
            return Err(SqlError::ExecutionError(format!(
                "Table {} already exists",
                info.name
            )));
        }
        let table = ColumnarTable::new(info);
        self.tables.insert(table.info.name.clone(), table);
        Ok(())
    }

    /// Scan a table with projection pushdown
    pub fn scan_with_projection(
        &self,
        table_name: &str,
        column_indices: &[usize],
    ) -> SqlResult<Vec<Record>> {
        if let Some(table) = self.tables.get(table_name) {
            Ok(table.project_to_records(column_indices))
        } else if let Some(path) = self.parquet_files.get(table_name) {
            let reader = ParquetReader::new(path.clone());
            reader.read_projected(column_indices)
        } else {
            Err(SqlError::TableNotFound(table_name.to_string()))
        }
    }

    /// Scan a table
    pub fn scan(&self, table_name: &str) -> SqlResult<Vec<Record>> {
        if let Some(table) = self.tables.get(table_name) {
            Ok(table.to_records())
        } else if let Some(path) = self.parquet_files.get(table_name) {
            let reader = ParquetReader::new(path.clone());
            reader.read()
        } else {
            Err(SqlError::TableNotFound(table_name.to_string()))
        }
    }

    /// Filter and project (combined optimization)
    pub fn filter_project(
        &self,
        table_name: &str,
        column_indices: &[usize],
        predicate: &dyn Fn(&[Value]) -> bool,
    ) -> SqlResult<Vec<Record>> {
        if let Some(table) = self.tables.get(table_name) {
            Ok(table.project_filter(column_indices, predicate))
        } else {
            Err(SqlError::TableNotFound(table_name.to_string()))
        }
    }

    /// Register a parquet file for a table
    pub fn register_parquet(&mut self, table_name: &str, path: String) {
        self.parquet_files.insert(table_name.to_string(), path);
    }

    /// Get table statistics
    pub fn get_table_stats(&self, table_name: &str) -> SqlResult<Option<Vec<ColumnStats>>> {
        if let Some(table) = self.tables.get(table_name) {
            let stats: Vec<ColumnStats> = (0..table.num_columns())
                .filter_map(|i| table.column_stats(i))
                .collect();
            Ok(Some(stats))
        } else {
            Ok(None)
        }
    }

    /// Check if table exists
    pub fn has_table(&self, table_name: &str) -> bool {
        self.tables.contains_key(table_name) || self.parquet_files.contains_key(table_name)
    }

    /// Export table to Parquet file
    pub fn export_to_parquet(&self, table_name: &str, path: &str) -> SqlResult<()> {
        if let Some(table) = self.tables.get(table_name) {
            let writer = ParquetWriter::new(path.to_string());
            writer.write_columnar(table)
        } else {
            Err(SqlError::TableNotFound(table_name.to_string()))
        }
    }

    /// Import table from Parquet file
    pub fn import_from_parquet(&mut self, table_name: &str, path: &str) -> SqlResult<()> {
        let reader = ParquetReader::new(path.to_string());
        let records = reader.read()?;
        
        // For simplicity, we just register the parquet file
        self.register_parquet(table_name.to_string(), path.to_string());
        
        // Also load into memory for smaller tables
        if records.len() < 10000 {
            let info = TableInfo {
                name: table_name.to_string(),
                columns: vec![], // Schema will be inferred
            };
            let table = ColumnarTable::from_records(info, &records);
            self.tables.insert(table_name.to_string(), table);
        }
        
        Ok(())
    }
}

impl Default for ColumnarStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_columnar_table_creation() {
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                },
            ],
        };
        let table = ColumnarTable::new(info);
        assert_eq!(table.num_rows(), 0);
        assert_eq!(table.num_columns(), 2);
    }

    #[test]
    fn test_columnar_table_append_row() {
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                },
            ],
        };
        let mut table = ColumnarTable::new(info);
        table.append_row(&vec![Value::Integer(1), Value::Text("Alice".to_string())]);
        table.append_row(&vec![Value::Integer(2), Value::Text("Bob".to_string())]);
        
        assert_eq!(table.num_rows(), 2);
        assert_eq!(table.num_columns(), 2);
    }

    #[test]
    fn test_columnar_table_to_records() {
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
            ],
        };
        let records = vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
        ];
        let table = ColumnarTable::from_records(info, &records);
        let result = table.to_records();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], Value::Integer(1));
        assert_eq!(result[1][0], Value::Integer(2));
    }

    #[test]
    fn test_projection_pushdown() {
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                },
                ColumnDefinition {
                    name: "email".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                },
            ],
        };
        let records = vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string()), Value::Text("alice@test.com".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string()), Value::Text("bob@test.com".to_string())],
        ];
        let table = ColumnarTable::from_records(info, &records);
        
        // Project only id and email (indices 0 and 2)
        let projected = table.project_to_records(&[0, 2]);
        
        assert_eq!(projected.len(), 2);
        assert_eq!(projected[0].len(), 2);
        assert_eq!(projected[0][0], Value::Integer(1));
        assert_eq!(projected[0][1], Value::Text("alice@test.com".to_string()));
    }

    #[test]
    fn test_columnar_filter() {
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
            ],
        };
        let records = vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
            vec![Value::Integer(4)],
        ];
        let table = ColumnarTable::from_records(info, &records);
        
        // Filter where id > 2
        let filtered = table.filter(&|row| {
            if let Value::Integer(id) = row[0] {
                id > 2
            } else {
                false
            }
        });
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], Value::Integer(3));
        assert_eq!(filtered[1][0], Value::Integer(4));
    }

    #[test]
    fn test_column_stats() {
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                },
            ],
        };
        let records = vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(5)],
            vec![Value::Integer(10)],
            vec![Value::Null],
        ];
        let table = ColumnarTable::from_records(info, &records);
        
        let stats = table.column_stats(0).unwrap();
        assert_eq!(stats.num_values, 4);
        assert_eq!(stats.null_count, 1);
        assert_eq!(stats.min_value, Some(Value::Integer(1)));
        assert_eq!(stats.max_value, Some(Value::Integer(10)));
    }

    #[test]
    fn test_columnar_storage_new() {
        let storage = ColumnarStorage::new();
        assert!(!storage.has_table("users"));
    }

    #[test]
    fn test_columnar_storage_create_table() {
        let mut storage = ColumnarStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
            ],
        };
        
        assert!(storage.create_table(info.clone()).is_ok());
        assert!(storage.has_table("users"));
    }

    #[test]
    fn test_columnar_storage_insert() {
        let mut storage = ColumnarStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
            ],
        };
        
        storage.create_table(info).unwrap();
        storage.insert("users", vec![vec![Value::Integer(1)]]).unwrap();
        
        let records = storage.scan("users").unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_projection_pushdown_in_storage() {
        let mut storage = ColumnarStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                },
            ],
        };
        
        storage.create_table(info).unwrap();
        storage.insert("users", vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
        ]).unwrap();
        
        // Project only id column
        let records = storage.scan_with_projection("users", &[0]).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].len(), 1);
        assert_eq!(records[0][0], Value::Integer(1));
    }

    #[test]
    fn test_filter_and_project() {
        let mut storage = ColumnarStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                },
            ],
        };
        
        storage.create_table(info).unwrap();
        storage.insert("users", vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
            vec![Value::Integer(3), Value::Text("Charlie".to_string())],
        ]).unwrap();
        
        // Filter where id > 1 and project only name
        let records = storage.filter_project("users", &[1], &|row| {
            if let Value::Integer(id) = row[0] {
                id > 1
            } else {
                false
            }
        }).unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0][0], Value::Text("Bob".to_string()));
        assert_eq!(records[1][0], Value::Text("Charlie".to_string()));
    }

    #[test]
    fn test_columnar_table_with_nulls() {
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                },
            ],
        };
        let mut table = ColumnarTable::new(info);
        table.append_row(&vec![Value::Integer(1), Value::Text("Alice".to_string())]);
        table.append_row(&vec![Value::Integer(2), Value::Null]);
        table.append_row(&vec![Value::Integer(3), Value::Text("Bob".to_string())]);
        
        assert_eq!(table.num_rows(), 3);
        
        let col1 = table.column(1).unwrap();
        assert_eq!(col1.get(0), Some(Value::Text("Alice".to_string())));
        assert_eq!(col1.get(1), Some(Value::Null));
        assert_eq!(col1.get(2), Some(Value::Text("Bob".to_string())));
    }
}
