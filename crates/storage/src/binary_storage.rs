//! Binary Storage Format
//!
//! Stores table data in compact binary format for fast I/O.
//! Format: [magic:4][version:4][row_count:8][columns:4][data...]

use crate::engine::{ColumnDefinition, TableData, TableInfo};
use sqlrustgo_types::Value;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

const MAGIC: u32 = 0x42494E41; // "BINA"
const VERSION: u32 = 1;

/// Binary table storage format
pub struct BinaryStorage {
    data_dir: PathBuf,
}

impl BinaryStorage {
    pub fn new(data_dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }

    /// Get binary file path for a table
    fn bin_path(&self, table: &str) -> PathBuf {
        self.data_dir.join(format!("{}.bin", table))
    }

    /// Save table in binary format
    pub fn save_binary(&self, table: &str, data: &TableData) -> std::io::Result<()> {
        let path = self.bin_path(table);
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        // Header
        writer.write_all(&MAGIC.to_le_bytes())?;
        writer.write_all(&VERSION.to_le_bytes())?;
        
        // Column count and info
        let col_count = data.info.columns.len() as u32;
        writer.write_all(&col_count.to_le_bytes())?;
        
        // Column types (encoded: I=Integer, F=Float, S=String, N=Null)
        for col in &data.info.columns {
            let type_code: u8 = match col.data_type.to_uppercase().as_str() {
                t if t.contains("INT") => b'I',
                t if t.contains("FLOAT") || t.contains("DOUBLE") || t.contains("REAL") => b'F',
                t if t.contains("TEXT") || t.contains("CHAR") || t.contains("VARCHAR") => b'S',
                _ => b'S',
            };
            writer.write_all(&[type_code])?;
        }

        // Row count
        let row_count = data.rows.len() as u64;
        writer.write_all(&row_count.to_le_bytes())?;

        // Write rows in binary
        for row in &data.rows {
            for value in row {
                match value {
                    Value::Integer(i) => {
                        writer.write_all(&[b'I'])?;
                        writer.write_all(&i.to_le_bytes())?;
                    }
                    Value::Float(f) => {
                        writer.write_all(&[b'F'])?;
                        writer.write_all(&f.to_le_bytes())?;
                    }
                    Value::Text(s) => {
                        writer.write_all(&[b'S'])?;
                        let bytes = s.as_bytes();
                        let len = bytes.len() as u32;
                        writer.write_all(&len.to_le_bytes())?;
                        writer.write_all(bytes)?;
                    }
                    _ => {
                        writer.write_all(&[b'N'])?;
                    }
                }
            }
        }

        writer.flush()?;
        Ok(())
    }

    /// Load table from binary format
    pub fn load_binary(&self, table: &str) -> std::io::Result<TableData> {
        let path = self.bin_path(table);
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);

        // Read and verify header
        let mut magic_bytes = [0u8; 4];
        reader.read_exact(&mut magic_bytes)?;
        let magic = u32::from_le_bytes(magic_bytes);
        if magic != MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid binary file format",
            ));
        }

        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let _version = u32::from_le_bytes(version_bytes);

        // Column info
        let mut col_count_bytes = [0u8; 4];
        reader.read_exact(&mut col_count_bytes)?;
        let col_count = u32::from_le_bytes(col_count_bytes) as usize;

        let mut col_types = Vec::with_capacity(col_count);
        for _ in 0..col_count {
            let mut type_byte = [0u8; 1];
            reader.read_exact(&mut type_byte)?;
            col_types.push(type_byte[0]);
        }

        // Build table info
        let columns: Vec<ColumnDefinition> = (0..col_count)
            .map(|i| ColumnDefinition {
                name: format!("col_{}", i),
                data_type: match col_types[i] {
                    b'I' => "INTEGER".to_string(),
                    b'F' => "REAL".to_string(),
                    _ => "TEXT".to_string(),
                },
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
            })
            .collect();

        let info = TableInfo {
            name: table.to_string(),
            columns,
        };

        // Row count
        let mut row_count_bytes = [0u8; 8];
        reader.read_exact(&mut row_count_bytes)?;
        let row_count = u64::from_le_bytes(row_count_bytes) as usize;

        // Read rows
        let mut rows = Vec::with_capacity(row_count);
        for _ in 0..row_count {
            let mut row = Vec::with_capacity(col_count);
            for &type_code in &col_types {
                match type_code {
                    b'I' => {
                        let mut bytes = [0u8; 8];
                        reader.read_exact(&mut bytes)?;
                        row.push(Value::Integer(i64::from_le_bytes(bytes)));
                    }
                    b'F' => {
                        let mut bytes = [0u8; 8];
                        reader.read_exact(&mut bytes)?;
                        row.push(Value::Float(f64::from_le_bytes(bytes)));
                    }
                    b'S' => {
                        let mut len_bytes = [0u8; 4];
                        reader.read_exact(&mut len_bytes)?;
                        let len = u32::from_le_bytes(len_bytes) as usize;
                        let mut str_bytes = vec![0u8; len];
                        reader.read_exact(&mut str_bytes)?;
                        row.push(Value::Text(
                            String::from_utf8(str_bytes).unwrap_or_default(),
                        ));
                    }
                    _ => {
                        row.push(Value::Null);
                    }
                }
            }
            rows.push(row);
        }

        Ok(TableData { info, rows })
    }

    /// Check if binary file exists
    pub fn binary_exists(&self, table: &str) -> bool {
        self.bin_path(table).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_types::Value;

    #[test]
    fn test_binary_save_load() {
        let temp_dir = std::env::temp_dir().join("binary_test");
        let storage = BinaryStorage::new(temp_dir.clone()).unwrap();

        let data = TableData {
            info: TableInfo {
                name: "test".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        references: None,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "name".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: true,
                        is_unique: false,
                        is_primary_key: false,
                        references: None,
                        auto_increment: false,
                    },
                ],
            },
            rows: vec![
                vec![Value::Integer(1), Value::Text("Alice".to_string())],
                vec![Value::Integer(2), Value::Text("Bob".to_string())],
            ],
        };

        storage.save_binary("test", &data).unwrap();
        assert!(storage.binary_exists("test"));

        let loaded = storage.load_binary("test").unwrap();
        assert_eq!(loaded.rows.len(), 2);
        assert_eq!(loaded.info.columns.len(), 2);

        // Clean up
        std::fs::remove_dir_all(temp_dir).ok();
    }
}
