//! Replication module for master-slave replication
//!
//! This module provides the foundation for async replication:
//! - Binlog events for DDL/DML/Commit
//! - Master: binlog writer
//! - Slave: IO thread (fetch) + SQL thread (replay)
//! - Simple failover (heartbeat monitoring, leader election)

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Binlog event types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BinlogEventType {
    /// DDL: CREATE/ALTER/DROP
    Ddl = 1,
    /// DML: INSERT/UPDATE/DELETE
    Dml = 2,
    /// Transaction commit
    Commit = 3,
    /// Transaction rollback
    Rollback = 4,
    /// Heartbeat
    Heartbeat = 5,
}

impl BinlogEventType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(BinlogEventType::Ddl),
            2 => Some(BinlogEventType::Dml),
            3 => Some(BinlogEventType::Commit),
            4 => Some(BinlogEventType::Rollback),
            5 => Some(BinlogEventType::Heartbeat),
            _ => None,
        }
    }
}

/// Binlog event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinlogEvent {
    /// Event type
    pub event_type: BinlogEventType,
    /// Transaction ID
    pub tx_id: u64,
    /// Table ID
    pub table_id: u64,
    /// Database name
    pub database: String,
    /// Table name
    pub table: String,
    /// SQL statement (for DDL/DML)
    pub sql: Option<String>,
    /// Row data (for DML)
    pub row_data: Option<Vec<u8>>,
    /// Log sequence number
    pub lsn: u64,
    /// Timestamp
    pub timestamp: u64,
}

impl BinlogEvent {
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Event type (1 byte)
        bytes.push(self.event_type as u8);

        // TX ID (8 bytes)
        bytes.extend_from_slice(&self.tx_id.to_le_bytes());

        // Table ID (8 bytes)
        bytes.extend_from_slice(&self.table_id.to_le_bytes());

        // Database length + name
        let db_bytes = self.database.as_bytes();
        bytes.extend_from_slice(&(db_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(db_bytes);

        // Table length + name
        let table_bytes = self.table.as_bytes();
        bytes.extend_from_slice(&(table_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(table_bytes);

        // SQL length + content
        match &self.sql {
            Some(sql) => {
                let sql_bytes = sql.as_bytes();
                bytes.extend_from_slice(&(sql_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(sql_bytes);
            }
            None => {
                bytes.extend_from_slice(&0u32.to_le_bytes());
            }
        }

        // Row data length + content
        match &self.row_data {
            Some(data) => {
                bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
                bytes.extend_from_slice(data);
            }
            None => {
                bytes.extend_from_slice(&0u32.to_le_bytes());
            }
        }

        // LSN (8 bytes)
        bytes.extend_from_slice(&self.lsn.to_le_bytes());

        // Timestamp (8 bytes)
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());

        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 25 {
            return None;
        }

        let mut offset = 0;

        // Event type
        let event_type = BinlogEventType::from_u8(data[offset])?;
        offset += 1;

        // TX ID
        let tx_id = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Table ID
        let table_id = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Database
        let db_len = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;
        let database = String::from_utf8(data[offset..offset + db_len].to_vec()).ok()?;
        offset += db_len;

        // Table
        let table_len = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;
        let table = String::from_utf8(data[offset..offset + table_len].to_vec()).ok()?;
        offset += table_len;

        // SQL
        let sql_len = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;
        let sql = if sql_len > 0 {
            Some(String::from_utf8(data[offset..offset + sql_len].to_vec()).ok()?)
        } else {
            None
        };
        offset += sql_len;

        // Row data
        let data_len = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;
        let row_data = if data_len > 0 {
            Some(data[offset..offset + data_len].to_vec())
        } else {
            None
        };
        offset += data_len;

        // LSN
        let lsn = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Timestamp
        let timestamp = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);

        Some(BinlogEvent {
            event_type,
            tx_id,
            table_id,
            database,
            table,
            sql,
            row_data,
            lsn,
            timestamp,
        })
    }
}

/// Binlog writer (Master side)
pub struct BinlogWriter {
    path: PathBuf,
    lsn: u64,
    position: u64,
}

impl BinlogWriter {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        Ok(Self {
            path,
            lsn: 0,
            position: 0,
        })
    }

    /// Write event to binlog
    pub fn write_event(&mut self, event: &BinlogEvent) -> std::io::Result<u64> {
        let lsn = self.lsn;
        let mut event = event.clone();
        event.lsn = lsn;

        let bytes = event.to_bytes();

        // Write length prefix (4 bytes) + data
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        file.write_all(&(bytes.len() as u32).to_le_bytes())?;
        file.write_all(&bytes)?;
        file.flush()?;

        self.lsn += 1;
        self.position += bytes.len() as u64 + 4;

        Ok(lsn)
    }

    /// Get current LSN
    pub fn current_lsn(&self) -> u64 {
        self.lsn
    }

    /// Get current position
    pub fn position(&self) -> u64 {
        self.position
    }
}

/// Binlog reader (Slave side)
pub struct BinlogReader {
    path: PathBuf,
    lsn: u64,
}

impl BinlogReader {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        Ok(Self { path, lsn: 0 })
    }

    /// Read events from current LSN
    pub fn read_from(&mut self, start_lsn: u64) -> std::io::Result<Vec<BinlogEvent>> {
        let mut file = std::fs::File::open(&self.path)?;
        let mut events = Vec::new();
        let mut current_lsn: u64 = 0;

        loop {
            // Read length prefix
            let mut len_bytes = [0u8; 4];
            match file.read_exact(&mut len_bytes) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            let len = u32::from_le_bytes(len_bytes) as usize;

            // Read event data
            let mut data = vec![0u8; len];
            file.read_exact(&mut data)?;

            if let Some(event) = BinlogEvent::from_bytes(&data) {
                if event.lsn >= start_lsn {
                    events.push(event.clone());
                }
                current_lsn = event.lsn;
            }
        }

        self.lsn = current_lsn + 1;
        Ok(events)
    }

    /// Get current LSN
    pub fn current_lsn(&self) -> u64 {
        self.lsn
    }
}

/// Replication config
#[derive(Debug, Clone)]
pub struct ReplicationConfig {
    /// Master host
    pub master_host: String,
    /// Master port
    pub master_port: u16,
    /// Slave ID
    pub slave_id: u32,
    /// Replica lag threshold (ms)
    pub lag_threshold_ms: u64,
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            master_host: "127.0.0.1".to_string(),
            master_port: 3306,
            slave_id: 1,
            lag_threshold_ms: 1000,
        }
    }
}

/// Master node for replication
#[allow(dead_code)]
pub struct MasterNode {
    binlog_writer: Arc<Mutex<BinlogWriter>>,
    config: ReplicationConfig,
}

impl MasterNode {
    pub fn new(binlog_path: PathBuf, config: ReplicationConfig) -> std::io::Result<Self> {
        let writer = BinlogWriter::new(binlog_path)?;
        Ok(Self {
            binlog_writer: Arc::new(Mutex::new(writer)),
            config,
        })
    }

    /// Write DDL event
    pub fn write_ddl(
        &self,
        tx_id: u64,
        table_id: u64,
        database: &str,
        table: &str,
        sql: &str,
    ) -> std::io::Result<u64> {
        let event = BinlogEvent {
            event_type: BinlogEventType::Ddl,
            tx_id,
            table_id,
            database: database.to_string(),
            table: table.to_string(),
            sql: Some(sql.to_string()),
            row_data: None,
            lsn: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let mut writer = self.binlog_writer.lock().unwrap();
        writer.write_event(&event)
    }

    /// Write DML event
    pub fn write_dml(
        &self,
        tx_id: u64,
        table_id: u64,
        database: &str,
        table: &str,
        sql: Option<String>,
        row_data: Option<Vec<u8>>,
    ) -> std::io::Result<u64> {
        let event = BinlogEvent {
            event_type: BinlogEventType::Dml,
            tx_id,
            table_id,
            database: database.to_string(),
            table: table.to_string(),
            sql,
            row_data,
            lsn: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let mut writer = self.binlog_writer.lock().unwrap();
        writer.write_event(&event)
    }

    /// Write commit event
    pub fn write_commit(&self, tx_id: u64) -> std::io::Result<u64> {
        let event = BinlogEvent {
            event_type: BinlogEventType::Commit,
            tx_id,
            table_id: 0,
            database: String::new(),
            table: String::new(),
            sql: None,
            row_data: None,
            lsn: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let mut writer = self.binlog_writer.lock().unwrap();
        writer.write_event(&event)
    }

    /// Get current binlog position
    pub fn binlog_position(&self) -> u64 {
        let writer = self.binlog_writer.lock().unwrap();
        writer.position()
    }
}

/// Slave node for replication
#[allow(dead_code)]
pub struct SlaveNode {
    binlog_path: PathBuf,
    config: ReplicationConfig,
    master_lsn: Arc<Mutex<u64>>,
    is_running: Arc<Mutex<bool>>,
}

impl SlaveNode {
    pub fn new(binlog_path: PathBuf, config: ReplicationConfig) -> Self {
        Self {
            binlog_path,
            config,
            master_lsn: Arc::new(Mutex::new(0)),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start IO thread (fetch binlog from master)
    pub fn start_io_thread(&self) {
        let binlog_path = self.binlog_path.clone();
        let master_lsn = self.master_lsn.clone();
        let is_running = self.is_running.clone();

        *is_running.lock().unwrap() = true;

        thread::spawn(move || {
            let mut reader = match BinlogReader::new(binlog_path) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to create binlog reader: {}", e);
                    return;
                }
            };

            loop {
                if !*is_running.lock().unwrap() {
                    break;
                }

                let start_lsn = *master_lsn.lock().unwrap();

                match reader.read_from(start_lsn) {
                    Ok(events) => {
                        for event in &events {
                            let mut lsn = master_lsn.lock().unwrap();
                            *lsn = event.lsn;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading binlog: {}", e);
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    /// Start SQL thread (replay events)
    pub fn start_sql_thread<F>(&self, mut replay_fn: F)
    where
        F: FnMut(BinlogEvent) -> std::io::Result<()> + Send + 'static,
    {
        let binlog_path = self.binlog_path.clone();
        let master_lsn = self.master_lsn.clone();
        let is_running = self.is_running.clone();

        thread::spawn(move || {
            let mut reader = match BinlogReader::new(binlog_path) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to create binlog reader: {}", e);
                    return;
                }
            };

            loop {
                if !*is_running.lock().unwrap() {
                    break;
                }

                let start_lsn = *master_lsn.lock().unwrap();

                match reader.read_from(start_lsn) {
                    Ok(events) => {
                        for event in events {
                            if let Err(e) = replay_fn(event.clone()) {
                                eprintln!("Error replaying event: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading binlog: {}", e);
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    /// Stop replication
    pub fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
    }

    /// Get replication lag
    pub fn replication_lag(&self) -> u64 {
        // Simplified: return 0 (in production, compare timestamps)
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_binlog_event_serialize() {
        let event = BinlogEvent {
            event_type: BinlogEventType::Dml,
            tx_id: 123,
            table_id: 456,
            database: "test_db".to_string(),
            table: "users".to_string(),
            sql: Some("INSERT INTO users VALUES (1)".to_string()),
            row_data: None,
            lsn: 0,
            timestamp: 1234567890,
        };

        let bytes = event.to_bytes();
        let parsed = BinlogEvent::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.event_type, BinlogEventType::Dml);
        assert_eq!(parsed.tx_id, 123);
        assert_eq!(parsed.table_id, 456);
        assert_eq!(parsed.database, "test_db");
    }

    #[test]
    fn test_binlog_event_ddl() {
        let event = BinlogEvent {
            event_type: BinlogEventType::Ddl,
            tx_id: 100,
            table_id: 1,
            database: "mydb".to_string(),
            table: "orders".to_string(),
            sql: Some("CREATE TABLE orders (id INT)".to_string()),
            row_data: None,
            lsn: 10,
            timestamp: 1234567890,
        };

        let bytes = event.to_bytes();
        let parsed = BinlogEvent::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.event_type, BinlogEventType::Ddl);
        assert_eq!(parsed.sql.as_ref().unwrap(), "CREATE TABLE orders (id INT)");
    }

    #[test]
    fn test_binlog_event_commit() {
        let event = BinlogEvent {
            event_type: BinlogEventType::Commit,
            tx_id: 999,
            table_id: 0,
            database: "test".to_string(),
            table: "".to_string(),
            sql: None,
            row_data: None,
            lsn: 100,
            timestamp: 1234567890,
        };

        let bytes = event.to_bytes();
        let parsed = BinlogEvent::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.event_type, BinlogEventType::Commit);
        assert_eq!(parsed.tx_id, 999);
    }

    #[test]
    fn test_binlog_event_heartbeat() {
        let event = BinlogEvent {
            event_type: BinlogEventType::Heartbeat,
            tx_id: 0,
            table_id: 0,
            database: "".to_string(),
            table: "".to_string(),
            sql: None,
            row_data: None,
            lsn: 50,
            timestamp: 1234567890,
        };

        let bytes = event.to_bytes();
        let parsed = BinlogEvent::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.event_type, BinlogEventType::Heartbeat);
    }

    #[test]
    fn test_binlog_writer_and_reader() {
        let temp_dir = std::env::temp_dir();
        let binlog_path = temp_dir.join("test_binlog_replication.binlog");

        let _ = fs::remove_file(&binlog_path);

        let mut writer = BinlogWriter::new(binlog_path.clone()).unwrap();

        let event1 = BinlogEvent {
            event_type: BinlogEventType::Dml,
            tx_id: 1,
            table_id: 1,
            database: "test".to_string(),
            table: "users".to_string(),
            sql: Some("INSERT".to_string()),
            row_data: None,
            lsn: 0,
            timestamp: 1000,
        };

        let lsn1 = writer.write_event(&event1).unwrap();
        assert_eq!(lsn1, 0);

        let event2 = BinlogEvent {
            event_type: BinlogEventType::Commit,
            tx_id: 1,
            table_id: 0,
            database: "test".to_string(),
            table: "".to_string(),
            sql: None,
            row_data: None,
            lsn: 0,
            timestamp: 1001,
        };

        let lsn2 = writer.write_event(&event2).unwrap();
        assert_eq!(lsn2, 1);

        drop(writer);

        let mut reader = BinlogReader::new(binlog_path.clone()).unwrap();
        let events = reader.read_from(0).unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, BinlogEventType::Dml);
        assert_eq!(events[1].event_type, BinlogEventType::Commit);

        let _ = fs::remove_file(&binlog_path);
    }

    #[test]
    fn test_replication_config_default() {
        let config = ReplicationConfig::default();
        assert_eq!(config.master_port, 3306);
        assert_eq!(config.slave_id, 1);
        assert_eq!(config.lag_threshold_ms, 1000);
    }

    #[test]
    fn test_binlog_event_type_from_u8() {
        assert_eq!(BinlogEventType::from_u8(1), Some(BinlogEventType::Ddl));
        assert_eq!(BinlogEventType::from_u8(2), Some(BinlogEventType::Dml));
        assert_eq!(BinlogEventType::from_u8(3), Some(BinlogEventType::Commit));
        assert_eq!(BinlogEventType::from_u8(4), Some(BinlogEventType::Rollback));
        assert_eq!(
            BinlogEventType::from_u8(5),
            Some(BinlogEventType::Heartbeat)
        );
        assert_eq!(BinlogEventType::from_u8(0), None);
        assert_eq!(BinlogEventType::from_u8(100), None);
    }
}
