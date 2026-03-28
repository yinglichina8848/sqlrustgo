//! Columnar file format for persistent storage
//!
//! Provides a Parquet-like binary columnar format with compression and statistics.

use crate::columnar::chunk::{Bitmap, ColumnChunk, ColumnStats};
use crate::columnar::segment::CompressionType;
use crate::engine::{ColumnDefinition, TableInfo};
use serde::{Deserialize, Serialize};
use sqlrustgo_types::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParquetCompatError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

pub type ParquetResult<T> = Result<T, ParquetCompatError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializedColumnChunk {
    values: Vec<SerializedValue>,
    null_count: usize,
    min_value: Option<SerializedValue>,
    max_value: Option<SerializedValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SerializedValue {
    Null,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Text(String),
    Blob(Vec<u8>),
}

impl SerializedValue {
    fn type_code(&self) -> u8 {
        match self {
            SerializedValue::Null => 0,
            SerializedValue::Integer(_) => 1,
            SerializedValue::Float(_) => 2,
            SerializedValue::Boolean(_) => 3,
            SerializedValue::Text(_) => 4,
            SerializedValue::Blob(_) => 5,
        }
    }
}

impl From<&Value> for SerializedValue {
    fn from(v: &Value) -> Self {
        match v {
            Value::Null => SerializedValue::Null,
            Value::Integer(i) => SerializedValue::Integer(*i),
            Value::Float(f) => SerializedValue::Float(*f),
            Value::Boolean(b) => SerializedValue::Boolean(*b),
            Value::Text(s) => SerializedValue::Text(s.clone()),
            Value::Blob(b) => SerializedValue::Blob(b.clone()),
            _ => SerializedValue::Null,
        }
    }
}

impl From<&SerializedValue> for Value {
    fn from(sv: &SerializedValue) -> Self {
        match sv {
            SerializedValue::Null => Value::Null,
            SerializedValue::Integer(i) => Value::Integer(*i),
            SerializedValue::Float(f) => Value::Float(*f),
            SerializedValue::Boolean(b) => Value::Boolean(*b),
            SerializedValue::Text(s) => Value::Text(s.clone()),
            SerializedValue::Blob(b) => Value::Blob(b.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializedChunkData {
    columns: Vec<SerializedColumnChunk>,
    num_rows: usize,
}

pub struct ParquetCompatReader {
    path: String,
}

impl ParquetCompatReader {
    pub fn new(path: String) -> ParquetResult<Self> {
        Ok(Self { path })
    }

    pub fn read(&self) -> ParquetResult<Vec<Vec<Value>>> {
        let mut file = File::open(&self.path)?;
        Self::validate_magic(&mut file)?;

        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf)?;
        let data_len = u32::from_le_bytes(len_buf) as usize;

        let mut data = vec![0u8; data_len];
        file.read_exact(&mut data)?;

        let decompressed = Self::decompress_all(&data)?;

        let chunk_data: SerializedChunkData = serde_json::from_slice(&decompressed)?;

        let mut records: Vec<Vec<Value>> = Vec::with_capacity(chunk_data.num_rows);
        for row_idx in 0..chunk_data.num_rows {
            let mut record = Vec::with_capacity(chunk_data.columns.len());
            for col in &chunk_data.columns {
                let value = if row_idx < col.values.len() {
                    Value::from(&col.values[row_idx])
                } else {
                    Value::Null
                };
                record.push(value);
            }
            records.push(record);
        }

        Ok(records)
    }

    pub fn read_columnar(&self) -> ParquetResult<Vec<ColumnChunk>> {
        let mut file = File::open(&self.path)?;
        Self::validate_magic(&mut file)?;

        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf)?;
        let data_len = u32::from_le_bytes(len_buf) as usize;

        let mut data = vec![0u8; data_len];
        file.read_exact(&mut data)?;

        let decompressed = Self::decompress_all(&data)?;

        let chunk_data: SerializedChunkData = serde_json::from_slice(&decompressed)?;

        let mut chunks = Vec::with_capacity(chunk_data.columns.len());
        for col_data in chunk_data.columns {
            let mut chunk = ColumnChunk::new();
            for sv in &col_data.values {
                let v: Value = sv.into();
                if matches!(v, Value::Null) {
                    chunk.push_null();
                } else {
                    chunk.push(v);
                }
            }
            chunks.push(chunk);
        }

        Ok(chunks)
    }

    pub fn get_statistics(&self) -> ParquetResult<Vec<ColumnStats>> {
        let mut file = File::open(&self.path)?;
        Self::validate_magic(&mut file)?;

        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf)?;
        let data_len = u32::from_le_bytes(len_buf) as usize;

        let mut data = vec![0u8; data_len];
        file.read_exact(&mut data)?;

        let decompressed = Self::decompress_all(&data)?;
        let chunk_data: SerializedChunkData = serde_json::from_slice(&decompressed)?;

        Ok(chunk_data
            .columns
            .iter()
            .map(|col| {
                let mut stats = ColumnStats::new();
                stats.null_count = col.null_count;
                if let Some(min) = &col.min_value {
                    stats.min_value = Some(min.into());
                }
                if let Some(max) = &col.max_value {
                    stats.max_value = Some(max.into());
                }
                stats
            })
            .collect())
    }

    pub fn num_rows(&self) -> ParquetResult<u64> {
        let mut file = File::open(&self.path)?;
        Self::validate_magic(&mut file)?;

        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf)?;
        let data_len = u32::from_le_bytes(len_buf) as usize;

        let mut data = vec![0u8; data_len];
        file.read_exact(&mut data)?;

        let decompressed = Self::decompress_all(&data)?;
        let chunk_data: SerializedChunkData = serde_json::from_slice(&decompressed)?;

        Ok(chunk_data.num_rows as u64)
    }

    pub fn num_columns(&self) -> ParquetResult<u64> {
        let mut file = File::open(&self.path)?;
        Self::validate_magic(&mut file)?;

        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf)?;
        let data_len = u32::from_le_bytes(len_buf) as usize;

        let mut data = vec![0u8; data_len];
        file.read_exact(&mut data)?;

        let decompressed = Self::decompress_all(&data)?;
        let chunk_data: SerializedChunkData = serde_json::from_slice(&decompressed)?;

        Ok(chunk_data.columns.len() as u64)
    }

    fn validate_magic(file: &mut File) -> ParquetResult<()> {
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)?;
        if &magic != b"PARQCKT\n" {
            return Err(ParquetCompatError::InvalidFormat(
                "Not a valid ParquetCompat file".to_string(),
            ));
        }
        Ok(())
    }

    fn decompress_all(data: &[u8]) -> ParquetResult<Vec<u8>> {
        if data.len() < 4 {
            return Err(ParquetCompatError::InvalidFormat(
                "Data too short".to_string(),
            ));
        }

        let marker = data[0];
        let rest = &data[1..];

        match marker {
            0 => Ok(rest.to_vec()),
            1 => {
                let mut decoder = snap::read::FrameDecoder::new(rest);
                let mut decompressed = Vec::new();
                std::io::copy(&mut decoder, &mut decompressed)
                    .map_err(|e| ParquetCompatError::Compression(e.to_string()))?;
                Ok(decompressed)
            }
            2 => zstd::decode_all(rest).map_err(|e| ParquetCompatError::Compression(e.to_string())),
            _ => Err(ParquetCompatError::InvalidFormat(format!(
                "Unknown compression marker: {}",
                marker
            ))),
        }
    }
}

pub struct ParquetCompatWriter {
    path: String,
    compression: CompressionType,
}

impl ParquetCompatWriter {
    pub fn new(path: String) -> Self {
        Self {
            path,
            compression: CompressionType::Snappy,
        }
    }

    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = compression;
        self
    }

    pub fn write(&self, info: &TableInfo, records: &[Vec<Value>]) -> ParquetResult<()> {
        let mut file = File::create(&self.path)?;
        file.write_all(b"PARQCKT\n")?;

        let num_rows = records.len();
        let num_columns = info.columns.len();

        let mut columns: Vec<SerializedColumnChunk> = Vec::with_capacity(num_columns);
        for col_idx in 0..num_columns {
            let mut values = Vec::with_capacity(num_rows);
            let mut null_count = 0;
            let mut min_value: Option<SerializedValue> = None;
            let mut max_value: Option<SerializedValue> = None;

            for record in records {
                let value = if col_idx < record.len() {
                    record[col_idx].clone()
                } else {
                    Value::Null
                };

                if matches!(value, Value::Null) {
                    null_count += 1;
                }

                let sv: SerializedValue = (&value).into();
                values.push(sv.clone());

                if !matches!(value, Value::Null) {
                    match &min_value {
                        None => min_value = Some(sv.clone()),
                        Some(min) => {
                            if Self::value_less_than(&sv, min) {
                                min_value = Some(sv.clone());
                            }
                        }
                    }
                    match &max_value {
                        None => max_value = Some(sv.clone()),
                        Some(max) => {
                            if Self::value_greater_than(&sv, max) {
                                max_value = Some(sv.clone());
                            }
                        }
                    }
                }
            }

            columns.push(SerializedColumnChunk {
                values,
                null_count,
                min_value,
                max_value,
            });
        }

        let chunk_data = SerializedChunkData { columns, num_rows };

        let json_data = serde_json::to_vec(&chunk_data)?;
        let compressed = self.compress(&json_data)?;

        file.write_all(&(compressed.len() as u32).to_le_bytes())?;
        file.write_all(&compressed)?;

        Ok(())
    }

    pub fn write_columnar(&self, info: &TableInfo, chunks: &[ColumnChunk]) -> ParquetResult<()> {
        if chunks.is_empty() {
            return self.write(info, &[]);
        }

        let num_rows = chunks[0].len();
        let num_columns = chunks.len();

        let mut records: Vec<Vec<Value>> = Vec::with_capacity(num_rows);
        for row_idx in 0..num_rows {
            let mut record = Vec::with_capacity(num_columns);
            for chunk in chunks {
                if let Some(value) = chunk.get(row_idx) {
                    record.push(value.clone());
                } else {
                    record.push(Value::Null);
                }
            }
            records.push(record);
        }

        self.write(info, &records)
    }

    fn compress(&self, data: &[u8]) -> ParquetResult<Vec<u8>> {
        match self.compression {
            CompressionType::None => {
                let mut result = vec![0u8];
                result.extend_from_slice(data);
                Ok(result)
            }
            CompressionType::Snappy => {
                let mut encoder = snap::write::FrameEncoder::new(Vec::new());
                encoder
                    .write_all(data)
                    .map_err(|e| ParquetCompatError::Compression(e.to_string()))?;
                encoder
                    .flush()
                    .map_err(|e| ParquetCompatError::Compression(e.to_string()))?;
                let mut result = vec![1u8];
                result.extend(
                    encoder
                        .into_inner()
                        .map_err(|e| ParquetCompatError::Compression(e.to_string()))?,
                );
                Ok(result)
            }
            CompressionType::Zstd => {
                let compressed = zstd::encode_all(data, 0)
                    .map_err(|e| ParquetCompatError::Compression(e.to_string()))?;
                let mut result = vec![2u8];
                result.extend(compressed);
                Ok(result)
            }
        }
    }

    fn value_less_than(a: &SerializedValue, b: &SerializedValue) -> bool {
        match (a, b) {
            (SerializedValue::Integer(a), SerializedValue::Integer(b)) => a < b,
            (SerializedValue::Float(a), SerializedValue::Float(b)) => a < b,
            (SerializedValue::Text(a), SerializedValue::Text(b)) => a < b,
            _ => false,
        }
    }

    fn value_greater_than(a: &SerializedValue, b: &SerializedValue) -> bool {
        match (a, b) {
            (SerializedValue::Integer(a), SerializedValue::Integer(b)) => a > b,
            (SerializedValue::Float(a), SerializedValue::Float(b)) => a > b,
            (SerializedValue::Text(a), SerializedValue::Text(b)) => a > b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_parquet_compat_write_read_roundtrip() {
        let dir = temp_dir();
        let path = dir.join("test_roundtrip.parquet");

        let info = TableInfo {
            name: "test_table".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        let records = vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
            vec![Value::Integer(3), Value::Null],
        ];

        let writer = ParquetCompatWriter::new(path.to_string_lossy().to_string());
        writer.write(&info, &records).unwrap();

        let reader = ParquetCompatReader::new(path.to_string_lossy().to_string()).unwrap();
        let read_records = reader.read().unwrap();

        assert_eq!(read_records.len(), 3);
        assert_eq!(read_records[0][0], Value::Integer(1));
        assert_eq!(read_records[0][1], Value::Text("Alice".to_string()));
        assert_eq!(read_records[2][0], Value::Integer(3));
        assert_eq!(read_records[2][1], Value::Null);

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_parquet_compat_with_compression() {
        let dir = temp_dir();
        let path = dir.join("test_compressed.parquet");

        let info = TableInfo {
            name: "compressed_table".to_string(),
            columns: vec![ColumnDefinition {
                name: "data".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        let records: Vec<Vec<Value>> = (0..100)
            .map(|i| vec![Value::Text(format!("value_{}", i))])
            .collect();

        let writer = ParquetCompatWriter::new(path.to_string_lossy().to_string())
            .with_compression(CompressionType::Snappy);
        writer.write(&info, &records).unwrap();

        let reader = ParquetCompatReader::new(path.to_string_lossy().to_string()).unwrap();
        let read_records = reader.read().unwrap();

        assert_eq!(read_records.len(), 100);
        assert_eq!(read_records[50][0], Value::Text("value_50".to_string()));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_parquet_compat_zstd_compression() {
        let dir = temp_dir();
        let path = dir.join("test_zstd.parquet");

        let info = TableInfo {
            name: "zstd_table".to_string(),
            columns: vec![ColumnDefinition {
                name: "num".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        let records: Vec<Vec<Value>> = (0..50).map(|i| vec![Value::Integer(i)]).collect();

        let writer = ParquetCompatWriter::new(path.to_string_lossy().to_string())
            .with_compression(CompressionType::Zstd);
        writer.write(&info, &records).unwrap();

        let reader = ParquetCompatReader::new(path.to_string_lossy().to_string()).unwrap();
        let read_records = reader.read().unwrap();

        assert_eq!(read_records.len(), 50);
        assert_eq!(read_records[25][0], Value::Integer(25));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_parquet_compat_statistics() {
        let dir = temp_dir();
        let path = dir.join("test_stats.parquet");

        let info = TableInfo {
            name: "stats_table".to_string(),
            columns: vec![ColumnDefinition {
                name: "value".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        let records = vec![
            vec![Value::Integer(10)],
            vec![Value::Integer(20)],
            vec![Value::Null],
            vec![Value::Integer(5)],
        ];

        let writer = ParquetCompatWriter::new(path.to_string_lossy().to_string());
        writer.write(&info, &records).unwrap();

        let reader = ParquetCompatReader::new(path.to_string_lossy().to_string()).unwrap();
        let stats = reader.get_statistics().unwrap();

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].null_count, 1);

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_parquet_compat_columnar_roundtrip() {
        let dir = temp_dir();
        let path = dir.join("test_columnar.parquet");

        let info = TableInfo {
            name: "columnar_table".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "score".to_string(),
                    data_type: "FLOAT".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        let mut chunk1 = ColumnChunk::new();
        let mut chunk2 = ColumnChunk::new();
        for i in 0..5 {
            chunk1.push(Value::Integer(i as i64));
            if i % 2 == 0 {
                chunk2.push(Value::Float(i as f64 * 1.5));
            } else {
                chunk2.push(Value::Null);
            }
        }

        let writer = ParquetCompatWriter::new(path.to_string_lossy().to_string());
        writer.write_columnar(&info, &[chunk1, chunk2]).unwrap();

        let reader = ParquetCompatReader::new(path.to_string_lossy().to_string()).unwrap();
        let records = reader.read().unwrap();

        assert_eq!(records.len(), 5);
        assert_eq!(records[0][0], Value::Integer(0));
        assert_eq!(records[0][1], Value::Float(0.0));
        assert_eq!(records[1][1], Value::Null);

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_parquet_compat_invalid_format() {
        let dir = temp_dir();
        let path = dir.join("invalid.parquet");

        std::fs::write(&path, b"not a parquet file").unwrap();

        let result = ParquetCompatReader::new(path.to_string_lossy().to_string())
            .unwrap()
            .read();
        assert!(result.is_err());

        std::fs::remove_file(path).ok();
    }
}
