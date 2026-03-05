//! Write-Ahead Log (WAL) with group commit and buffering optimization

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{Condvar, Mutex};

/// WAL record types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalRecord {
    Begin {
        tx_id: u64,
    },
    Commit {
        tx_id: u64,
    },
    Rollback {
        tx_id: u64,
    },
    /// Batch commit marker
    BatchCommit {
        tx_ids: Vec<u64>,
        timestamp: u64,
    },
}

/// WAL configuration
#[derive(Debug, Clone, Copy)]
pub struct WalConfig {
    /// Maximum batch size for group commit
    pub batch_size: usize,
    /// Maximum wait time before flushing (milliseconds)
    pub flush_timeout_ms: u64,
    /// Buffer size
    pub buffer_size: usize,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            flush_timeout_ms: 5,
            buffer_size: 64,
        }
    }
}

/// WAL with group commit and buffering
#[allow(dead_code)]
pub struct WriteAheadLog {
    file: Mutex<File>,
    path: String,
    config: WalConfig,
    /// Write buffer
    buffer: Mutex<VecDeque<WalRecord>>,
    /// Pending batch for group commit
    pending_batch: Mutex<Vec<u64>>,
    /// Condition variable for signaling
    cv: Condvar,
    /// Stats
    stats: Mutex<WalStats>,
}

/// WAL statistics
#[derive(Debug, Default)]
pub struct WalStats {
    pub flush_count: u64,
    pub batch_commits: u64,
    pub total_records: u64,
    pub total_bytes: u64,
}

impl WalStats {
    pub fn avg_batch_size(&self) -> f64 {
        if self.batch_commits == 0 {
            return 0.0;
        }
        self.total_records as f64 / self.batch_commits as f64
    }
}

impl WriteAheadLog {
    /// Create or open WAL with config
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        Self::with_config(path, WalConfig::default())
    }

    /// Create WAL with custom config
    pub fn with_config(path: &str, config: WalConfig) -> Result<Self, std::io::Error> {
        #[allow(clippy::suspicious_open_options)]
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        Ok(Self {
            file: Mutex::new(file),
            path: path.to_string(),
            config,
            buffer: Mutex::new(VecDeque::with_capacity(config.buffer_size)),
            pending_batch: Mutex::new(Vec::with_capacity(config.batch_size)),
            cv: Condvar::new(),
            stats: Mutex::new(WalStats::default()),
        })
    }

    /// Append a record to the WAL (buffered)
    pub fn append(&self, record: &WalRecord) -> Result<(), std::io::Error> {
        let mut buffer = self.buffer.lock().unwrap();

        // Track stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_records += 1;
        }

        // For commit records, add to pending batch
        if let WalRecord::Commit { tx_id } = record {
            let mut batch = self.pending_batch.lock().unwrap();
            batch.push(*tx_id);
        }

        // Add to buffer
        buffer.push_back(record.clone());

        // Check if we should flush
        let should_flush = {
            let batch = self.pending_batch.lock().unwrap();
            batch.len() >= self.config.batch_size
        };

        if should_flush {
            drop(buffer);
            self.flush()?;
        }

        Ok(())
    }

    /// Force flush the buffer
    pub fn flush(&self) -> Result<(), std::io::Error> {
        // Take all records from buffer
        let records = {
            let mut buffer = self.buffer.lock().unwrap();
            let records: Vec<WalRecord> = buffer.drain(..).collect();
            records
        };

        if records.is_empty() {
            return Ok(());
        }

        // Check for batch commit opportunity
        let batch_tx_ids: Vec<u64> = records
            .iter()
            .filter_map(|r| {
                if let WalRecord::Commit { tx_id } = r {
                    Some(*tx_id)
                } else {
                    None
                }
            })
            .collect();

        let mut file = self.file.lock().unwrap();

        // Write all records
        for record in &records {
            self.write_record(&mut file, record)?;
        }

        // If we have multiple commits, write batch commit marker
        if batch_tx_ids.len() > 1 {
            let batch_record = WalRecord::BatchCommit {
                tx_ids: batch_tx_ids.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };
            self.write_record(&mut file, &batch_record)?;
        }

        file.flush()?;

        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.flush_count += 1;
            if batch_tx_ids.len() > 1 {
                stats.batch_commits += 1;
            }
        }

        Ok(())
    }

    /// Write a single record
    fn write_record(&self, file: &mut File, record: &WalRecord) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(record)
            .map_err(|_| std::io::Error::other("serialization failed"))?;

        let len_bytes = (json.len() as u32).to_le_bytes();
        file.write_all(&len_bytes)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;

        // Update byte stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_bytes += (4 + json.len()) as u64;
        }

        Ok(())
    }

    /// Synchronous append (bypass buffer)
    pub fn append_sync(&self, record: &WalRecord) -> Result<(), std::io::Error> {
        let mut file = self.file.lock().unwrap();
        self.write_record(&mut file, record)?;
        file.flush()?;
        Ok(())
    }

    /// Read all records from log
    pub fn read_all(&self) -> Result<Vec<WalRecord>, std::io::Error> {
        let mut file = self.file.lock().unwrap();
        let mut records = Vec::new();

        if file.seek(SeekFrom::Start(0)).is_err() {
            return Ok(records);
        }

        loop {
            let mut len_buf = [0u8; 4];
            match file.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(_) => break,
            }

            let len = u32::from_le_bytes(len_buf) as usize;
            let mut data = vec![0u8; len];
            if file.read_exact(&mut data).is_err() {
                break;
            }

            let mut newline = [0u8; 1];
            let _ = file.read(&mut newline);

            if let Ok(record) = serde_json::from_slice(&data) {
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Truncate log (after checkpoint)
    pub fn truncate(&self) -> Result<(), std::io::Error> {
        // Flush first
        self.flush()?;

        let mut file = self.file.lock().unwrap();
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> WalStats {
        let stats = self.stats.lock().unwrap();
        WalStats {
            flush_count: stats.flush_count,
            batch_commits: stats.batch_commits,
            total_records: stats.total_records,
            total_bytes: stats.total_bytes,
        }
    }

    /// Get current buffer size
    pub fn buffer_len(&self) -> usize {
        let buffer = self.buffer.lock().unwrap();
        buffer.len()
    }

    /// Get pending batch size
    pub fn pending_count(&self) -> usize {
        let batch = self.pending_batch.lock().unwrap();
        batch.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn cleanup(path: &str) {
        fs::remove_file(path).ok();
    }

    fn test_wal_append() {
        let path = "/tmp/wal_test_append.log";
        cleanup(path);

        let wal = WriteAheadLog::new(path).unwrap();
        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();

        let records = wal.read_all().unwrap();
        assert!(!records.is_empty());

        cleanup(path);
    }

    #[test]
    fn test_wal_group_commit() {
        let path = "/tmp/wal_test_group.log";
        cleanup(path);

        let config = WalConfig {
            batch_size: 3,
            flush_timeout_ms: 100,
            buffer_size: 10,
        };
        let wal = WriteAheadLog::with_config(path, config).unwrap();

        // Append multiple commits
        for i in 1..=5 {
            wal.append(&WalRecord::Begin { tx_id: i }).unwrap();
            wal.append(&WalRecord::Commit { tx_id: i }).unwrap();
        }

        // Force flush
        wal.flush().unwrap();

        let stats = wal.stats();
        assert!(stats.flush_count > 0);

        cleanup(path);
    }

    #[test]
    fn test_wal_stats() {
        let path = "/tmp/wal_test_stats.log";
        cleanup(path);

        let wal = WriteAheadLog::new(path).unwrap();

        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        wal.append(&WalRecord::Commit { tx_id: 1 }).unwrap();

        let stats = wal.stats();
        assert!(stats.total_records > 0);

        cleanup(path);
    }

    #[test]
    fn test_wal_buffer() {
        let path = "/tmp/wal_test_buffer.log";
        cleanup(path);

        let wal = WriteAheadLog::new(path).unwrap();

        wal.append(&WalRecord::Begin { tx_id: 1 }).unwrap();
        assert_eq!(wal.buffer_len(), 1);

        wal.flush().unwrap();
        assert_eq!(wal.buffer_len(), 0);

        cleanup(path);
    }

    #[test]
    fn test_wal_sync() {
        let path = "/tmp/wal_test_sync.log";
        cleanup(path);

        let wal = WriteAheadLog::new(path).unwrap();

        wal.append_sync(&WalRecord::Begin { tx_id: 1 }).unwrap();
        wal.append_sync(&WalRecord::Commit { tx_id: 1 }).unwrap();

        let records = wal.read_all().unwrap();
        assert_eq!(records.len(), 2);

        cleanup(path);
    }
}
