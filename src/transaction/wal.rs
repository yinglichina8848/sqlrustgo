//! Write-Ahead Log (WAL) for transaction management
//! Simple JSON-based logging for durability

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Mutex;

/// WAL record types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalRecord {
    Begin { tx_id: u64 },
    Commit { tx_id: u64 },
    Rollback { tx_id: u64 },
}

/// Write-Ahead Log
#[allow(dead_code)]
pub struct WriteAheadLog {
    file: Mutex<File>,
    path: String,
}

impl WriteAheadLog {
    /// Create or open WAL
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            file: Mutex::new(file),
            path: path.to_string(),
        })
    }

    /// Append a record to the log
    pub fn append(&self, record: &WalRecord) -> Result<(), std::io::Error> {
        let mut file = self.file.lock().unwrap();

        // Serialize to JSON
        let json = serde_json::to_string(record)
            .map_err(|_| std::io::Error::other("serialization failed"))?;

        // Write length prefix + newline
        let len_bytes = (json.len() as u32).to_le_bytes();
        file.write_all(&len_bytes)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        file.flush()?;

        Ok(())
    }

    /// Read all records from log
    pub fn read_all(&self) -> Result<Vec<WalRecord>, std::io::Error> {
        let mut file = self.file.lock().unwrap();
        let mut records = Vec::new();

        // Seek to start
        if file.seek(SeekFrom::Start(0)).is_err() {
            return Ok(records);
        }

        loop {
            let mut len_buf = [0u8; 4];
            match file.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(_) => break, // EOF
            }

            let len = u32::from_le_bytes(len_buf) as usize;
            let mut data = vec![0u8; len];
            if file.read_exact(&mut data).is_err() {
                break;
            }

            // Read newline
            let mut newline = [0u8; 1];
            let _ = file.read(&mut newline);

            if let Ok(record) = serde_json::from_slice(&data) {
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Truncate log (after successful checkpoint)
    pub fn truncate(&self) -> Result<(), std::io::Error> {
        let mut file = self.file.lock().unwrap();
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wal_append() {
        let path = "/tmp/wal_test_append.log";
        std::fs::remove_file(path).ok();

        let wal = WriteAheadLog::new(path).unwrap();
        let record = WalRecord::Begin { tx_id: 1 };
        wal.append(&record).unwrap();

        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 1);

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_wal_commit() {
        let path = "/tmp/wal_test_commit.log";
        std::fs::remove_file(path).ok();

        let wal = WriteAheadLog::new(path).unwrap();

        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        wal.append(&WalRecord::Commit { tx_id: 1 }).unwrap();

        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 2);

        std::fs::remove_file(path).ok();
    }
}
