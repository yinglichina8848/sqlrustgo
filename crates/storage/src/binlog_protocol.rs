//! Binlog Replication Protocol
//!
//! Defines the network protocol for master-slave replication:
//! - Message types for binlog events, heartbeats, and acknowledgments
//! - Serialization/deserialization for network transmission
//! - Protocol version and handshake

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReplicationMessage {
    HandshakeRequest {
        slave_id: u32,
        host: String,
        port: u16,
        replication_protocol_version: u32,
    },
    HandshakeResponse {
        server_id: u32,
        server_version: String,
        binlog_file: String,
        binlog_pos: u64,
    },
    BinlogData {
        file: String,
        pos: u64,
        events: Vec<BinlogEventData>,
    },
    Heartbeat {
        lsn: u64,
        timestamp: u64,
    },
    HeartbeatAck {
        lsn: u64,
    },
    BinlogPosRequest {
        file: String,
        pos: u64,
    },
    BinlogPosResponse {
        file: String,
        pos: u64,
    },
    Error {
        code: u16,
        message: String,
    },
    EOF,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinlogEventData {
    pub event_type: u8,
    pub tx_id: u64,
    pub table_id: u64,
    pub database: String,
    pub table: String,
    pub sql: Option<String>,
    pub row_data: Option<Vec<u8>>,
    pub lsn: u64,
    pub timestamp: u64,
}

impl ReplicationMessage {
    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn deserialize(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }
}

pub struct BinlogProtocol {
    version: u32,
}

impl BinlogProtocol {
    pub const VERSION: u32 = 1;
    pub const DEFAULT_MASTER_PORT: u16 = 3333;
    pub const HEARTBEAT_INTERVAL_MS: u64 = 1000;
    pub const CONNECTION_TIMEOUT_MS: u64 = 5000;
    pub const MAX_RETRY_ATTEMPTS: u32 = 3;
    pub const RETRY_BASE_DELAY_MS: u64 = 1000;
    pub const MAX_RETRY_DELAY_MS: u64 = 30000;
}

impl Default for BinlogProtocol {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
        }
    }
}

pub struct PacketReader;

impl PacketReader {
    pub fn read_packet(stream: &mut std::net::TcpStream) -> std::io::Result<Option<Vec<u8>>> {
        let mut len_buf = [0u8; 4];
        match stream.read_exact(&mut len_buf) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data)?;

        Ok(Some(data))
    }
}

pub struct PacketWriter;

impl PacketWriter {
    pub fn write_packet(stream: &mut std::net::TcpStream, data: &[u8]) -> std::io::Result<()> {
        let len = data.len() as u32;
        stream.write_all(&len.to_le_bytes())?;
        stream.write_all(data)?;
        stream.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_request_serialization() {
        let msg = ReplicationMessage::HandshakeRequest {
            slave_id: 2,
            host: "127.0.0.1".to_string(),
            port: 3306,
            replication_protocol_version: 1,
        };

        let bytes = msg.serialize();
        let parsed = ReplicationMessage::deserialize(&bytes).unwrap();

        match parsed {
            ReplicationMessage::HandshakeRequest {
                slave_id,
                host,
                port,
                replication_protocol_version,
            } => {
                assert_eq!(slave_id, 2);
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 3306);
                assert_eq!(replication_protocol_version, 1);
            }
            _ => panic!("Expected HandshakeRequest"),
        }
    }

    #[test]
    fn test_heartbeat_serialization() {
        let msg = ReplicationMessage::Heartbeat {
            lsn: 12345,
            timestamp: 1234567890,
        };

        let bytes = msg.serialize();
        let parsed = ReplicationMessage::deserialize(&bytes).unwrap();

        match parsed {
            ReplicationMessage::Heartbeat { lsn, timestamp } => {
                assert_eq!(lsn, 12345);
                assert_eq!(timestamp, 1234567890);
            }
            _ => panic!("Expected Heartbeat"),
        }
    }

    #[test]
    fn test_binlog_data_serialization() {
        let event = BinlogEventData {
            event_type: 2,
            tx_id: 100,
            table_id: 1,
            database: "test_db".to_string(),
            table: "users".to_string(),
            sql: Some("INSERT INTO users VALUES (1)".to_string()),
            row_data: None,
            lsn: 500,
            timestamp: 1234567890,
        };

        let msg = ReplicationMessage::BinlogData {
            file: "binlog.000001".to_string(),
            pos: 100,
            events: vec![event],
        };

        let bytes = msg.serialize();
        let parsed = ReplicationMessage::deserialize(&bytes).unwrap();

        match parsed {
            ReplicationMessage::BinlogData { file, pos, events } => {
                assert_eq!(file, "binlog.000001");
                assert_eq!(pos, 100);
                assert_eq!(events.len(), 1);
            }
            _ => panic!("Expected BinlogData"),
        }
    }
}
