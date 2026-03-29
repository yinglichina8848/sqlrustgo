//! Columnar Storage - ColumnSegment Disk Layout
//!
//! Handles serialization, compression, and disk I/O for column data.

use crate::columnar::chunk::{Bitmap, ColumnStats};
use serde::{Deserialize, Serialize};
use sqlrustgo_types::Value;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use thiserror::Error;

/// Errors for column segment operations
#[derive(Error, Debug)]
pub enum SegmentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Decompression error: {0}")]
    Decompression(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub type SegmentResult<T> = Result<T, SegmentError>;

/// Compression type for column data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    /// No compression
    None,
    /// Snappy compression
    Snappy,
    /// Zstd compression
    Zstd,
}

impl CompressionType {
    /// Get the magic bytes for this compression type
    pub fn magic_bytes(&self) -> [u8; 4] {
        match self {
            CompressionType::None => *b"NONE",
            CompressionType::Snappy => *b"SNAP",
            CompressionType::Zstd => *b"ZSTD",
        }
    }

    /// Detect compression type from magic bytes
    pub fn from_magic(magic: &[u8; 4]) -> Option<Self> {
        match magic {
            b"NONE" => Some(CompressionType::None),
            b"SNAP" => Some(CompressionType::Snappy),
            b"ZSTD" => Some(CompressionType::Zstd),
            _ => None,
        }
    }
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::None
    }
}

/// Statistics stored on disk for a column segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStatsDisk {
    /// Number of null values
    pub null_count: usize,
    /// Minimum value as serialized string
    pub min_value: Option<String>,
    /// Maximum value as serialized string
    pub max_value: Option<String>,
    /// Number of distinct values
    pub distinct_count: Option<usize>,
}

impl From<&ColumnStats> for ColumnStatsDisk {
    fn from(stats: &ColumnStats) -> Self {
        Self {
            null_count: stats.null_count,
            min_value: stats.min_value.as_ref().map(|v| value_to_string(v)),
            max_value: stats.max_value.as_ref().map(|v| value_to_string(v)),
            distinct_count: stats.distinct_count,
        }
    }
}

/// Convert Value to string representation for storage
fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_string(),
        Value::Boolean(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Text(s) => s.clone(),
        Value::Blob(b) => hex::encode(b),
        Value::Date(d) => d.to_string(),
        Value::Timestamp(ts) => ts.to_string(),
    }
}

/// Parse value from string representation
fn value_from_string(s: &str, value_type: &str) -> SegmentResult<Value> {
    match value_type {
        "NULL" => Ok(Value::Null),
        "TRUE" => Ok(Value::Boolean(true)),
        "FALSE" => Ok(Value::Boolean(false)),
        "INTEGER" => s
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| SegmentError::InvalidData(format!("Invalid integer: {}", s))),
        "FLOAT" => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| SegmentError::InvalidData(format!("Invalid float: {}", s))),
        "TEXT" => Ok(Value::Text(s.to_string())),
        "BLOB" => hex::decode(s)
            .map(Value::Blob)
            .map_err(|_| SegmentError::InvalidData(format!("Invalid blob: {}", s))),
        "DATE" => s
            .parse::<i32>()
            .map(Value::Date)
            .map_err(|_| SegmentError::InvalidData(format!("Invalid date: {}", s))),
        "TIMESTAMP" => s
            .parse::<i64>()
            .map(Value::Timestamp)
            .map_err(|_| SegmentError::InvalidData(format!("Invalid timestamp: {}", s))),
        _ => Err(SegmentError::InvalidData(format!(
            "Unknown type: {}",
            value_type
        ))),
    }
}

/// Column segment header stored at the beginning of each segment file
#[derive(Debug, Serialize, Deserialize)]
struct SegmentHeader {
    /// Column ID
    column_id: u32,
    /// Number of values in this segment
    num_values: u64,
    /// Compression type
    compression: CompressionType,
    /// Stats
    stats: ColumnStatsDisk,
    /// Bitmap size in bytes
    bitmap_size: u64,
    /// Data size in bytes (uncompressed)
    data_size: u64,
    /// Compressed data size (0 if uncompressed)
    compressed_size: u64,
}

/// ColumnSegment - disk layout for a column's data
#[derive(Debug, Clone)]
pub struct ColumnSegment {
    /// Column ID this segment belongs to
    column_id: u32,
    /// File offset where data starts
    offset: u64,
    /// Length of data in bytes
    length: u64,
    /// Compression type used
    compression: CompressionType,
    /// Statistics for this segment
    pub stats: ColumnStatsDisk,
    /// Number of values in this segment
    pub num_values: u64,
}

impl ColumnSegment {
    /// Create a new column segment
    pub fn new(column_id: u32) -> Self {
        Self {
            column_id,
            offset: 0,
            length: 0,
            compression: CompressionType::None,
            stats: ColumnStatsDisk {
                null_count: 0,
                min_value: None,
                max_value: None,
                distinct_count: None,
            },
            num_values: 0,
        }
    }

    /// Create with compression type
    pub fn with_compression(column_id: u32, compression: CompressionType) -> Self {
        Self {
            column_id,
            compression,
            ..Self::new(column_id)
        }
    }

    /// Get the column ID
    pub fn column_id(&self) -> u32 {
        self.column_id
    }

    /// Get the file offset
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Get the length
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Get compression type
    pub fn compression(&self) -> CompressionType {
        self.compression
    }

    /// Get statistics
    pub fn stats(&self) -> &ColumnStatsDisk {
        &self.stats
    }

    /// Get number of values
    pub fn num_values(&self) -> u64 {
        self.num_values
    }

    /// Set statistics
    pub fn set_stats(&mut self, stats: ColumnStatsDisk) {
        self.stats = stats;
    }

    /// Set number of values
    pub fn set_num_values(&mut self, num_values: u64) {
        self.num_values = num_values;
    }

    /// Serialize segment metadata to bytes
    pub fn serialize_metadata(&self) -> SegmentResult<Vec<u8>> {
        let header = SegmentHeader {
            column_id: self.column_id,
            num_values: self.num_values,
            compression: self.compression,
            stats: self.stats.clone(),
            bitmap_size: 0, // Will be set during data serialization
            data_size: 0,
            compressed_size: 0,
        };

        serde_json::to_vec(&header).map_err(|e| SegmentError::Serialization(e.to_string()))
    }

    /// Deserialize segment metadata from bytes
    pub fn deserialize_metadata(data: &[u8]) -> SegmentResult<Self> {
        let header: SegmentHeader =
            serde_json::from_slice(data).map_err(|e| SegmentError::Serialization(e.to_string()))?;

        Ok(Self {
            column_id: header.column_id,
            offset: 0,
            length: data.len() as u64,
            compression: header.compression,
            stats: header.stats,
            num_values: header.num_values,
        })
    }

    /// Write segment data to a file
    pub fn write_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        values: &[Value],
        null_bitmap: Option<&Bitmap>,
    ) -> SegmentResult<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        // Serialize values
        let values_json =
            serde_json::to_vec(values).map_err(|e| SegmentError::Serialization(e.to_string()))?;

        // Serialize bitmap
        let bitmap_data = null_bitmap.map(|b| b.bits.clone()).unwrap_or_default();
        let bitmap_json = serde_json::to_vec(&bitmap_data)
            .map_err(|e| SegmentError::Serialization(e.to_string()))?;

        // Apply compression if needed
        let (compressed_data, actual_compression) = match self.compression {
            CompressionType::None => (values_json.clone(), CompressionType::None),
            CompressionType::Snappy => {
                let compressed = compress_snappy(&values_json)?;
                if compressed.len() < values_json.len() {
                    (compressed, CompressionType::Snappy)
                } else {
                    (values_json.clone(), CompressionType::None)
                }
            }
            CompressionType::Zstd => {
                let compressed = compress_zstd(&values_json, 0)?;
                if compressed.len() < values_json.len() {
                    (compressed, CompressionType::Zstd)
                } else {
                    (values_json.clone(), CompressionType::None)
                }
            }
        };

        // Build header
        let header = SegmentHeader {
            column_id: self.column_id,
            num_values: values.len() as u64,
            compression: actual_compression,
            stats: self.stats.clone(),
            bitmap_size: bitmap_json.len() as u64,
            data_size: values_json.len() as u64,
            compressed_size: compressed_data.len() as u64,
        };

        let header_json =
            serde_json::to_vec(&header).map_err(|e| SegmentError::Serialization(e.to_string()))?;

        // Write header length
        let header_len = header_json.len() as u32;
        file.write_all(&header_len.to_le_bytes())?;

        // Write header
        file.write_all(&header_json)?;

        // Write bitmap length and data
        let bitmap_len = bitmap_json.len() as u32;
        file.write_all(&bitmap_len.to_le_bytes())?;
        file.write_all(&bitmap_json)?;

        // Write data length and data
        let data_len = compressed_data.len() as u32;
        file.write_all(&data_len.to_le_bytes())?;
        file.write_all(&compressed_data)?;

        Ok(())
    }

    /// Read segment data from a file
    pub fn read_from_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> SegmentResult<(Vec<Value>, Option<Bitmap>)> {
        let mut file = File::open(path)?;

        // Read header length
        let mut header_len_bytes = [0u8; 4];
        file.read_exact(&mut header_len_bytes)?;
        let header_len = u32::from_le_bytes(header_len_bytes);

        // Read header
        let mut header_json = vec![0u8; header_len as usize];
        file.read_exact(&mut header_json)?;
        let header: SegmentHeader = serde_json::from_slice(&header_json)
            .map_err(|e| SegmentError::Serialization(e.to_string()))?;

        self.column_id = header.column_id;
        self.num_values = header.num_values;
        self.compression = header.compression;
        self.stats = header.stats;
        self.offset = 0;

        // Read bitmap
        let mut bitmap_len_bytes = [0u8; 4];
        file.read_exact(&mut bitmap_len_bytes)?;
        let bitmap_len = u32::from_le_bytes(bitmap_len_bytes);

        let bitmap_data = if bitmap_len > 0 {
            let mut data = vec![0u8; bitmap_len as usize];
            file.read_exact(&mut data)?;
            Some(data)
        } else {
            None
        };

        // Read data
        let mut data_len_bytes = [0u8; 4];
        file.read_exact(&mut data_len_bytes)?;
        let data_len = u32::from_le_bytes(data_len_bytes);

        let mut compressed_data = vec![0u8; data_len as usize];
        file.read_exact(&mut compressed_data)?;

        self.length =
            (4 + header_json.len() + 4 + bitmap_len as usize + 4 + data_len as usize) as u64;

        // Decompress if needed
        let decompressed = match header.compression {
            CompressionType::None => compressed_data,
            CompressionType::Snappy => decompress_snappy(&compressed_data)?,
            CompressionType::Zstd => decompress_zstd(&compressed_data)?,
        };

        // Deserialize values
        let values: Vec<Value> = serde_json::from_slice(&decompressed)
            .map_err(|e| SegmentError::Serialization(e.to_string()))?;

        // Deserialize bitmap
        let null_bitmap = if let Some(data) = bitmap_data {
            let bits: Vec<u64> = serde_json::from_slice(&data)
                .map_err(|e| SegmentError::Serialization(e.to_string()))?;
            let len = bits.len() * 64;
            Some(Bitmap { bits, len })
        } else {
            None
        };

        Ok((values, null_bitmap))
    }
}

/// Compress data using Snappy
fn compress_snappy(data: &[u8]) -> SegmentResult<Vec<u8>> {
    use std::io::Write;

    let mut encoder = snap::write::FrameEncoder::new(Vec::new());
    encoder
        .write_all(data)
        .map_err(|e| SegmentError::Compression(e.to_string()))?;
    encoder
        .flush()
        .map_err(|e| SegmentError::Compression(e.to_string()))?;
    encoder
        .into_inner()
        .map_err(|e| SegmentError::Compression(format!("Snappy error: {:?}", e)))
}

/// Decompress data using Snappy
fn decompress_snappy(data: &[u8]) -> SegmentResult<Vec<u8>> {
    let mut decoder = snap::read::FrameDecoder::new(data);
    let mut decompressed = Vec::new();
    std::io::copy(&mut decoder, &mut decompressed)
        .map_err(|e| SegmentError::Decompression(e.to_string()))?;
    Ok(decompressed)
}

/// Compress data using Zstd
fn compress_zstd(data: &[u8], level: i32) -> SegmentResult<Vec<u8>> {
    let mut compressed = Vec::with_capacity(data.len());
    let mut encoder = zstd::Encoder::new(&mut compressed, level)
        .map_err(|e| SegmentError::Compression(format!("Zstd init error: {:?}", e)))?;
    std::io::Write::write_all(&mut encoder, data)
        .map_err(|e| SegmentError::Compression(e.to_string()))?;
    encoder
        .finish()
        .map_err(|e| SegmentError::Compression(format!("Zstd finish error: {:?}", e)))?;
    Ok(compressed)
}

/// Decompress data using Zstd
fn decompress_zstd(data: &[u8]) -> SegmentResult<Vec<u8>> {
    let mut decompressed = Vec::with_capacity(data.len() * 4);
    let mut decoder = zstd::Decoder::new(data)
        .map_err(|e| SegmentError::Decompression(format!("Zstd init error: {:?}", e)))?;
    std::io::copy(&mut decoder, &mut decompressed)
        .map_err(|e| SegmentError::Decompression(e.to_string()))?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_compression_type_magic() {
        assert_eq!(CompressionType::None.magic_bytes(), *b"NONE");
        assert_eq!(CompressionType::Snappy.magic_bytes(), *b"SNAP");
        assert_eq!(CompressionType::Zstd.magic_bytes(), *b"ZSTD");

        assert_eq!(
            CompressionType::from_magic(b"NONE"),
            Some(CompressionType::None)
        );
        assert_eq!(
            CompressionType::from_magic(b"SNAP"),
            Some(CompressionType::Snappy)
        );
        assert_eq!(
            CompressionType::from_magic(b"ZSTD"),
            Some(CompressionType::Zstd)
        );
        assert_eq!(CompressionType::from_magic(b"XXXX"), None);
    }

    #[test]
    fn test_column_segment_new() {
        let segment = ColumnSegment::new(1);
        assert_eq!(segment.column_id(), 1);
        assert_eq!(segment.offset(), 0);
        assert_eq!(segment.length(), 0);
        assert_eq!(segment.compression(), CompressionType::None);
    }

    #[test]
    fn test_column_segment_with_compression() {
        let segment = ColumnSegment::with_compression(1, CompressionType::Zstd);
        assert_eq!(segment.column_id(), 1);
        assert_eq!(segment.compression(), CompressionType::Zstd);
    }

    #[test]
    fn test_stats_disk_from_column_stats() {
        use crate::columnar::chunk::ColumnStats;

        let mut stats = ColumnStats::new();
        stats.update(&Value::Integer(10), false);
        stats.update(&Value::Integer(20), false);
        stats.update(&Value::Null, true);

        let disk_stats = ColumnStatsDisk::from(&stats);
        assert_eq!(disk_stats.null_count, 1);
        assert_eq!(disk_stats.min_value, Some("10".to_string()));
        assert_eq!(disk_stats.max_value, Some("20".to_string()));
    }

    #[test]
    fn test_write_and_read_no_compression() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("segment_test.bin");

        let values = vec![
            Value::Integer(10),
            Value::Integer(20),
            Value::Null,
            Value::Integer(40),
        ];

        let mut bitmap = Bitmap::with_capacity(4);
        bitmap.set(0);
        bitmap.set(1);
        // index 2 is null
        bitmap.set(3);

        let segment = ColumnSegment::new(1);
        segment
            .write_to_file(&path, &values, Some(&bitmap))
            .unwrap();

        let mut read_segment = ColumnSegment::new(0);
        let (read_values, read_bitmap) = read_segment.read_from_file(&path).unwrap();

        assert_eq!(read_segment.column_id(), 1);
        assert_eq!(read_values.len(), 4);
        assert_eq!(read_values[0], Value::Integer(10));
        assert_eq!(read_values[1], Value::Integer(20));
        assert_eq!(read_values[2], Value::Null);
        assert_eq!(read_values[3], Value::Integer(40));

        let bitmap = read_bitmap.unwrap();
        assert!(bitmap.is_set(0));
        assert!(bitmap.is_set(1));
        assert!(bitmap.is_null(2));
        assert!(bitmap.is_set(3));
    }

    #[test]
    fn test_write_and_read_with_snappy_compression() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("segment_snappy.bin");

        let values: Vec<Value> = (0..1000).map(|i| Value::Integer(i)).collect();

        let mut bitmap = Bitmap::with_capacity(1000);
        for i in 0..1000 {
            if i % 2 == 0 {
                bitmap.set(i);
            }
        }

        let segment = ColumnSegment::with_compression(1, CompressionType::Snappy);
        segment
            .write_to_file(&path, &values, Some(&bitmap))
            .unwrap();

        let mut read_segment = ColumnSegment::new(0);
        let (read_values, read_bitmap) = read_segment.read_from_file(&path).unwrap();

        assert_eq!(read_segment.column_id(), 1);
        assert_eq!(read_values.len(), 1000);
        assert_eq!(read_values[0], Value::Integer(0));
        assert_eq!(read_values[999], Value::Integer(999));
    }

    #[test]
    fn test_write_and_read_with_zstd_compression() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("segment_zstd.bin");

        let values: Vec<Value> = (0..1000)
            .map(|i| Value::Text(format!("value_{}", i)))
            .collect();

        let segment = ColumnSegment::with_compression(1, CompressionType::Zstd);
        segment.write_to_file(&path, &values, None).unwrap();

        let mut read_segment = ColumnSegment::new(0);
        let (read_values, read_bitmap) = read_segment.read_from_file(&path).unwrap();

        assert_eq!(read_segment.column_id(), 1);
        assert_eq!(read_values.len(), 1000);
        assert_eq!(read_values[0], Value::Text("value_0".to_string()));
        assert_eq!(read_values[999], Value::Text("value_999".to_string()));
        assert!(read_bitmap.is_none());
    }

    #[test]
    fn test_serialize_deserialize_metadata() {
        let segment = ColumnSegment::new(42);
        let metadata = segment.serialize_metadata().unwrap();
        let deserialized = ColumnSegment::deserialize_metadata(&metadata).unwrap();

        assert_eq!(deserialized.column_id, 42);
    }
}
