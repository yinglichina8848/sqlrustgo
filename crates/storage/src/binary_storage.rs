//! Binary Table Storage
//!
//! Fast binary format for table storage, replacing JSON.

use crate::engine::{ColumnDefinition, TableData, TableInfo};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

/// Binary storage for tables
pub struct BinaryTableStorage {
    data_dir: PathBuf,
}

impl BinaryTableStorage {
    pub fn new(data_dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }

    fn table_path(&self, table: &str) -> PathBuf {
        self.data_dir.join(format!("{}.bin", table))
    }

    /// Save table in binary format
    pub fn save(&self, table: &str, data: &TableData) -> std::io::Result<()> {
        let path = self.table_path(table);
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        // Header: magic + version
        w.write_all(b"BINT")?;
        w.write_all(&1u32.to_le_bytes())?;

        // Column count & info
        w.write_all(&(data.info.columns.len() as u32).to_le_bytes())?;
        for col in &data.info.columns {
            let code = match col.data_type.to_uppercase().as_str() {
                t if t.contains("INT") => 1u8,
                t if t.contains("FLOAT") || t.contains("REAL") => 2u8,
                _ => 3u8, // TEXT
            };
            w.write_all(&[code])?;
        }

        // Row count
        w.write_all(&(data.rows.len() as u64).to_le_bytes())?;

        // Write rows
        for row in &data.rows {
            for val in row {
                match val {
                    sqlrustgo_types::Value::Integer(i) => {
                        w.write_all(&[1])?;
                        w.write_all(&i.to_le_bytes())?;
                    }
                    sqlrustgo_types::Value::Float(f) => {
                        w.write_all(&[2])?;
                        w.write_all(&f.to_le_bytes())?;
                    }
                    sqlrustgo_types::Value::Text(s) => {
                        w.write_all(&[3])?;
                        let bytes = s.as_bytes();
                        w.write_all(&(bytes.len() as u32).to_le_bytes())?;
                        w.write_all(bytes)?;
                    }
                    sqlrustgo_types::Value::Null => {
                        w.write_all(&[0])?;
                    }
                    _ => {
                        w.write_all(&[0])?;
                    }
                }
            }
        }

        w.flush()
    }

    /// Load table from binary format (optimized)
    pub fn load(&self, table: &str) -> std::io::Result<TableData> {
        let path = self.table_path(table);
        let file = File::open(path)?;
        
        // Use buffered reader with large buffer for faster reading
        use std::io::BufReader;
        let mut reader = BufReader::with_capacity(8 * 1024 * 1024, file); // 8MB buffer
        
        // Verify header
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"BINT" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not a binary table file",
            ));
        }

        let mut ver = [0u8; 4];
        reader.read_exact(&mut ver)?;

        // Column info
        let mut col_count = [0u8; 4];
        reader.read_exact(&mut col_count)?;
        let col_count = u32::from_le_bytes(col_count) as usize;

        let mut col_types = Vec::with_capacity(col_count);
        for _ in 0..col_count {
            let mut tc = [0u8; 1];
            reader.read_exact(&mut tc)?;
            col_types.push(tc[0]);
        }

        // Row count
        let mut row_count = [0u8; 8];
        reader.read_exact(&mut row_count)?;
        let row_count = u64::from_le_bytes(row_count) as usize;

        // Build columns
        let columns: Vec<ColumnDefinition> = (0..col_count)
            .map(|i| ColumnDefinition {
                name: format!("col_{}", i),
                data_type: match col_types[i] {
                    1 => "INTEGER".to_string(),
                    2 => "REAL".to_string(),
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

        // Read rows
        let mut rows = Vec::with_capacity(row_count);
        for _ in 0..row_count {
            let mut row = Vec::with_capacity(col_count);
            for _ in 0..col_types.len() {
                // Read value type code
                let mut vtc = [0u8; 1];
                reader.read_exact(&mut vtc)?;
                match vtc[0] {
                    0 => row.push(sqlrustgo_types::Value::Null),
                    1 => {
                        let mut buf = [0u8; 8];
                        reader.read_exact(&mut buf)?;
                        row.push(sqlrustgo_types::Value::Integer(i64::from_le_bytes(buf)));
                    }
                    2 => {
                        let mut buf = [0u8; 8];
                        reader.read_exact(&mut buf)?;
                        row.push(sqlrustgo_types::Value::Float(f64::from_le_bytes(buf)));
                    }
                    3 => {
                        let mut len = [0u8; 4];
                        reader.read_exact(&mut len)?;
                        let len = u32::from_le_bytes(len) as usize;
                        let mut s = vec![0u8; len];
                        reader.read_exact(&mut s)?;
                        row.push(sqlrustgo_types::Value::Text(String::from_utf8_lossy(&s).to_string()));
                    }
                    _ => row.push(sqlrustgo_types::Value::Null),
                }
            }
            rows.push(row);
        }

        Ok(TableData { info, rows })
    }

    pub fn exists(&self, table: &str) -> bool {
        self.table_path(table).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_save_load() {
        let tmp = std::env::temp_dir().join("bin_test");
        let storage = BinaryTableStorage::new(tmp.clone()).unwrap();

        let cols = vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: false,
            is_primary_key: false,
            references: None,
            auto_increment: false,
        }];
        let rows = vec![vec![sqlrustgo_types::Value::Integer(42)]];

        let data = TableData {
            info: TableInfo {
                name: "test".to_string(),
                columns: cols,
            },
            rows,
        };

        storage.save("test", &data).unwrap();
        assert!(storage.exists("test"));

        let loaded = storage.load("test").unwrap();
        assert_eq!(loaded.rows.len(), 1);
        
        if let sqlrustgo_types::Value::Integer(i) = loaded.rows[0][0] {
            assert_eq!(i, 42);
        } else {
            panic!("Expected Integer");
        }

        std::fs::remove_dir_all(tmp).ok();
    }
}
