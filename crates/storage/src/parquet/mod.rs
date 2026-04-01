//! Parquet import/export support for SQLRustGo
//!
//! Provides ParquetReader and ParquetWriter for COPY FROM/TO Parquet support.
//!
//! # Data Flow
//!
//! - Parquet → Arrow Array → Record → ColumnarStorage (import)
//! - ColumnarStorage → Record → Arrow Array → Parquet (export)

pub mod reader;
pub mod writer;

pub use reader::ParquetReader;
pub use writer::ParquetWriter;

use crate::engine::{Record, SqlResult};

/// Import records from a Parquet file
pub fn import_from_parquet(path: &str, columns: &[String]) -> SqlResult<Vec<Record>> {
    reader::read_parquet_file(path, columns)
}

/// Export records to a Parquet file
pub fn export_to_parquet(path: &str, records: &[Record], column_names: &[String]) -> SqlResult<()> {
    writer::write_parquet_file(path, records, column_names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{ColumnDefinition, TableInfo};
    use tempfile::tempdir;

    #[test]
    fn test_parquet_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.parquet");

        // Create test records
        let records = vec![
            vec![
                sqlrustgo_types::Value::Integer(1),
                sqlrustgo_types::Value::Text("hello".to_string()),
            ],
            vec![
                sqlrustgo_types::Value::Integer(2),
                sqlrustgo_types::Value::Text("world".to_string()),
            ],
            vec![
                sqlrustgo_types::Value::Null,
                sqlrustgo_types::Value::Text("null_test".to_string()),
            ],
        ];

        let columns = vec!["id".to_string(), "name".to_string()];

        // Write to Parquet
        writer::write_parquet_file(path.to_str().unwrap(), &records, &columns).unwrap();

        // Read from Parquet
        let read_records = reader::read_parquet_file(path.to_str().unwrap(), &columns).unwrap();

        assert_eq!(records.len(), read_records.len());
        for (original, read) in records.iter().zip(read_records.iter()) {
            assert_eq!(original.len(), read.len());
            for (o, r) in original.iter().zip(read.iter()) {
                assert_eq!(o, r);
            }
        }
    }
}
