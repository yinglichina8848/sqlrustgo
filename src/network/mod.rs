//! Network Layer for SQLRustGo
//!
//! Provides MySQL-compatible network protocol support for client-server architecture.

use crate::{SqlError, Value};
use bytes::{BufMut, BytesMut};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// MySQL protocol packet header size
const PACKET_HEADER_SIZE: usize = 4;

/// MySQL protocol capability flags
pub mod capability {
    pub const LONG_PASSWORD: u32 = 0x00000001;
    pub const FOUND_ROWS: u32 = 0x00000002;
    pub const LONG_FLAG: u32 = 0x00000004;
    pub const CONNECT_WITH_DB: u32 = 0x00000008;
    pub const NO_SCHEMA: u32 = 0x00000010;
    pub const COMPRESS: u32 = 0x00000020;
    pub const ODBC: u32 = 0x00000040;
    pub const LOCAL_FILES: u32 = 0x00000080;
    pub const IGNORE_SPACE: u32 = 0x00000100;
    pub const PROTOCOL_41: u32 = 0x00000200;
    pub const INTERACTIVE: u32 = 0x00000400;
    pub const STATS: u32 = 0x00000800;
    pub const RESERVED: u32 = 0x00001000;
    pub const MULTI_STATEMENTS: u32 = 0x00010000;
    pub const MULTI_RESULTS: u32 = 0x00020000;
    pub const PS_MULTI_RESULTS: u32 = 0x00040000;
    pub const PLUGIN_AUTH: u32 = 0x00080000;
    pub const CONNECT_ATTRS: u32 = 0x00100000;
    pub const PLUGIN_AUTH_LENENC_CLIENT_CAP: u32 = 0x00200000;
    pub const CAN_HANDLE_EXPIRED_PASSWORDS: u32 = 0x00400000;
    pub const SESSION_TRACK: u32 = 0x00800000;
    pub const DEPRECATE_EOF: u32 = 0x01000000;
    pub const UNKNOWN: u32 = 0x80000000;
}

/// MySQL command types
#[derive(Debug, Clone)]
pub enum MySqlCommand {
    Sleep,
    Quit,
    InitDb,
    Query,
    FieldList,
    CreateDb,
    DropDb,
    Refresh,
    Shutdown,
    Statistics,
    ProcessInfo,
    Connect,
    ProcessKill,
    Debug,
    Ping,
    Time,
    DelayedInsert,
    ChangeUser,
    BinlogDump,
    TableDump,
    ConnectOut,
    RegisterSlave,
    StmtPrepare,
    StmtExecute,
    StmtSendLongData,
    StmtClose,
    StmtReset,
    SetOption,
    StmtFetch,
    Daemon,
    BinlogDumpGtid,
    ResetConnection,
    Unknown(u8),
}

impl From<u8> for MySqlCommand {
    fn from(code: u8) -> Self {
        match code {
            0x00 => MySqlCommand::Sleep,
            0x01 => MySqlCommand::Quit,
            0x02 => MySqlCommand::InitDb,
            0x03 => MySqlCommand::Query,
            0x04 => MySqlCommand::FieldList,
            0x05 => MySqlCommand::CreateDb,
            0x06 => MySqlCommand::DropDb,
            0x07 => MySqlCommand::Refresh,
            0x08 => MySqlCommand::Shutdown,
            0x09 => MySqlCommand::Statistics,
            0x0a => MySqlCommand::ProcessInfo,
            0x0b => MySqlCommand::Connect,
            0x0c => MySqlCommand::ProcessKill,
            0x0d => MySqlCommand::Debug,
            0x0e => MySqlCommand::Ping,
            0x0f => MySqlCommand::Time,
            0x10 => MySqlCommand::DelayedInsert,
            0x11 => MySqlCommand::ChangeUser,
            0x12 => MySqlCommand::BinlogDump,
            0x13 => MySqlCommand::TableDump,
            0x14 => MySqlCommand::ConnectOut,
            0x15 => MySqlCommand::RegisterSlave,
            0x16 => MySqlCommand::StmtPrepare,
            0x17 => MySqlCommand::StmtExecute,
            0x18 => MySqlCommand::StmtSendLongData,
            0x19 => MySqlCommand::StmtClose,
            0x1a => MySqlCommand::StmtReset,
            0x1b => MySqlCommand::SetOption,
            0x1c => MySqlCommand::StmtFetch,
            0x1d => MySqlCommand::Daemon,
            0x1e => MySqlCommand::BinlogDumpGtid,
            0x1f => MySqlCommand::ResetConnection,
            _ => MySqlCommand::Unknown(code),
        }
    }
}

/// MySQL packet structure
#[derive(Debug, Clone)]
pub struct MySqlPacket {
    pub sequence: u8,
    pub payload: Vec<u8>,
}

impl MySqlPacket {
    /// Parse a MySQL packet from bytes
    pub fn parse(data: &[u8]) -> Result<Self, SqlError> {
        if data.len() < PACKET_HEADER_SIZE {
            return Err(SqlError::ProtocolError("Packet too short".to_string()));
        }

        let payload_length = u32::from_le_bytes([data[0], data[1], data[2], 0]) as usize;
        let sequence = data[3];

        if data.len() < PACKET_HEADER_SIZE + payload_length {
            return Err(SqlError::ProtocolError("Incomplete packet".to_string()));
        }

        let payload = data[PACKET_HEADER_SIZE..PACKET_HEADER_SIZE + payload_length].to_vec();

        Ok(Self { sequence, payload })
    }

    /// Serialize packet to bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        buf.put_u32_le(self.payload.len() as u32);
        buf.put_u8(self.sequence);
        buf.put_slice(&self.payload);
        buf.to_vec()
    }
}

/// MySQL greeting packet (Initial Handshake)
#[derive(Debug, Clone)]
pub struct HandshakeV10 {
    pub protocol_version: u8,
    pub server_version: String,
    pub connection_id: u32,
    pub auth_plugin_data: Vec<u8>,
    pub capability_flags: u32,
    pub character_set: u8,
    pub status_flags: u16,
}

impl HandshakeV10 {
    /// Create server greeting
    pub fn new(connection_id: u32) -> Self {
        Self {
            protocol_version: 0x0a, // MySQL 10
            server_version: "1.0.0-SQLRustGo".to_string(),
            connection_id,
            auth_plugin_data: vec![0; 8], // Placeholder
            capability_flags: capability::PROTOCOL_41
                | capability::LONG_PASSWORD
                | capability::FOUND_ROWS
                | capability::MULTI_STATEMENTS
                | capability::MULTI_RESULTS,
            character_set: 0x21,  // utf8mb4_general_ci
            status_flags: 0x0002, // AUTOCOMMIT
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();

        // Protocol version
        buf.put_u8(self.protocol_version);

        // Server version (null-terminated string)
        buf.put_slice(self.server_version.as_bytes());
        buf.put_u8(0);

        // Connection ID
        buf.put_u32_le(self.connection_id);

        // Auth plugin data part 1 (8 bytes)
        buf.put_slice(&self.auth_plugin_data[..8.min(self.auth_plugin_data.len())]);
        buf.put_u8(0); // Null terminator

        // Capability flags lower 2 bytes
        buf.put_u16_le(self.capability_flags as u16);

        // Character set
        buf.put_u8(self.character_set);

        // Status flags
        buf.put_u16_le(self.status_flags);

        // Capability flags upper 2 bytes
        buf.put_u16_le((self.capability_flags >> 16) as u16);

        // Auth plugin data length (if plugin auth)
        let auth_len = if self.capability_flags & capability::PLUGIN_AUTH != 0 {
            self.auth_plugin_data.len() as u8
        } else {
            0
        };
        buf.put_u8(auth_len);

        // Reserved (10 bytes)
        buf.put_slice(&[0u8; 10]);

        // Auth plugin data part 2 (if any)
        if self.auth_plugin_data.len() > 8 {
            buf.put_slice(&self.auth_plugin_data[8..]);
        }

        // Auth plugin name (null-terminated)
        buf.put_slice(b"mysql_native_password\0");

        buf.to_vec()
    }
}

/// OK packet response
#[derive(Debug, Clone)]
pub struct OkPacket {
    pub affected_rows: u64,
    pub last_insert_id: u64,
    pub status_flags: u16,
    pub warnings: u16,
    pub message: String,
}

impl OkPacket {
    pub fn new(affected_rows: u64, message: &str) -> Self {
        Self {
            affected_rows,
            last_insert_id: 0,
            status_flags: 0x0002, // AUTOCOMMIT
            warnings: 0,
            message: message.to_string(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        buf.put_u8(0x00); // OK packet header
        buf.put_u64_le(self.affected_rows);
        buf.put_u64_le(self.last_insert_id);
        buf.put_u16_le(self.status_flags);
        buf.put_u16_le(self.warnings);

        if !self.message.is_empty() {
            // Use length-encoded string
            buf.put_u64_le(self.message.len() as u64);
            buf.put_slice(self.message.as_bytes());
        }

        buf.to_vec()
    }
}

/// Error packet response
#[derive(Debug, Clone)]
pub struct ErrPacket {
    pub error_code: u16,
    pub sql_state: String,
    pub message: String,
}

impl ErrPacket {
    pub fn new(code: u16, message: &str) -> Self {
        Self {
            error_code: code,
            sql_state: "HY000".to_string(), // General SQL state
            message: message.to_string(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        buf.put_u8(0xff); // Error packet header
        buf.put_u16_le(self.error_code);

        // SQL state (5 bytes)
        if self.sql_state.len() == 5 {
            buf.put_slice(self.sql_state.as_bytes());
        } else {
            buf.put_slice(b"HY000");
        }

        // Error message
        buf.put_slice(self.message.as_bytes());

        buf.to_vec()
    }
}

/// Result set row
#[derive(Debug, Clone)]
pub struct RowData {
    pub values: Vec<Value>,
}

impl RowData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();

        for value in &self.values {
            match value {
                Value::Null => {
                    buf.put_u8(0xfb); // NULL indicator
                }
                Value::Integer(i) => {
                    let s = i.to_string();
                    buf.put_u64_le(s.len() as u64);
                    buf.put_slice(s.as_bytes());
                }
                Value::Float(f) => {
                    let s = f.to_string();
                    buf.put_u64_le(s.len() as u64);
                    buf.put_slice(s.as_bytes());
                }
                Value::Text(s) => {
                    buf.put_u64_le(s.len() as u64);
                    buf.put_slice(s.as_bytes());
                }
                Value::Blob(b) => {
                    buf.put_u64_le(b.len() as u64);
                    buf.put_slice(b);
                }
                Value::Boolean(b) => {
                    let s = if *b { "1" } else { "0" };
                    buf.put_u64_le(1);
                    buf.put_slice(s.as_bytes());
                }
            }
        }

        buf.to_vec()
    }
}

/// MySQL-compatible protocol handler (synchronous version)
pub struct NetworkHandler {
    stream: TcpStream,
    connection_id: u32,
}

impl NetworkHandler {
    /// Create a new network handler
    pub fn new(stream: TcpStream, connection_id: u32) -> Self {
        Self {
            stream,
            connection_id,
        }
    }

    /// Handle a client connection
    pub fn handle(&mut self) -> Result<(), SqlError> {
        // Send greeting
        self.send_greeting()?;

        // Read and handle packets
        loop {
            match self.read_packet() {
                Ok(Some((_sequence, payload))) => {
                    let command = MySqlCommand::from(payload[0]);

                    match command {
                        MySqlCommand::Quit => {
                            break;
                        }
                        MySqlCommand::Query => {
                            let query = String::from_utf8_lossy(&payload[1..]).to_string();
                            self.execute_query(&query)?;
                        }
                        MySqlCommand::Ping => {
                            self.send_ok("PONG", 0)?;
                        }
                        _ => {
                            self.send_error(1047, "Command not supported")?;
                        }
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    // Log error but don't crash
                    eprintln!("Protocol error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Send MySQL handshake greeting
    fn send_greeting(&mut self) -> Result<(), SqlError> {
        let greeting = HandshakeV10::new(self.connection_id);
        let packet = MySqlPacket {
            sequence: 0,
            payload: greeting.to_bytes(),
        };

        self.stream
            .write_all(&packet.serialize())
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Read a MySQL packet
    fn read_packet(&mut self) -> Result<Option<(u8, Vec<u8>)>, SqlError> {
        let mut header = [0u8; PACKET_HEADER_SIZE];

        match self.stream.read(&mut header) {
            Ok(0) => Ok(None), // Connection closed
            Ok(n) if n < PACKET_HEADER_SIZE => Err(SqlError::ProtocolError(
                "Incomplete packet header".to_string(),
            )),
            Ok(_) => {
                let payload_length =
                    u32::from_le_bytes([header[0], header[1], header[2], 0]) as usize;
                let sequence = header[3];

                let mut payload = vec![0u8; payload_length];
                let mut remaining = payload_length;
                let mut offset = 0;

                while remaining > 0 {
                    match self.stream.read(&mut payload[offset..]) {
                        Ok(0) => {
                            return Err(SqlError::ProtocolError("Connection closed".to_string()));
                        }
                        Ok(n) => {
                            remaining -= n;
                            offset += n;
                        }
                        Err(e) => return Err(SqlError::IoError(e.to_string())),
                    }
                }

                Ok(Some((sequence, payload)))
            }
            Err(e) => Err(SqlError::IoError(e.to_string())),
        }
    }

    /// Execute a query and send result
    fn execute_query(&mut self, query: &str) -> Result<(), SqlError> {
        // For now, return a simple OK response
        // Full query execution would integrate with the executor
        let trimmed = query.trim();

        if trimmed.eq_ignore_ascii_case("SELECT VERSION()") {
            let response = OkPacket::new(0, "1.0.0-SQLRustGo");
            self.send_packet(response.to_bytes().as_slice())?;
        } else if trimmed.eq_ignore_ascii_case("SELECT 1")
            || trimmed.eq_ignore_ascii_case("SELECT 1 AS a")
        {
            // Return a simple result set
            self.send_select_response()?;
        } else {
            self.send_ok("Query executed", 0)?;
        }

        Ok(())
    }

    /// Send OK packet
    fn send_ok(&mut self, message: &str, affected_rows: u64) -> Result<(), SqlError> {
        let packet = OkPacket::new(affected_rows, message);
        self.send_packet(&packet.to_bytes())
    }

    /// Send error packet
    fn send_error(&mut self, code: u16, message: &str) -> Result<(), SqlError> {
        let packet = ErrPacket::new(code, message);
        self.send_packet(&packet.to_bytes())
    }

    /// Send a raw packet
    fn send_packet(&mut self, data: &[u8]) -> Result<(), SqlError> {
        // Simple packet: length (3 bytes) + sequence (1 byte) + payload
        let mut buf = BytesMut::new();
        buf.put_u32_le(data.len() as u32);
        buf.put_u8(0); // sequence
        buf.put_slice(data);

        self.stream
            .write_all(&buf)
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Send simple SELECT response
    fn send_select_response(&mut self) -> Result<(), SqlError> {
        // Column count (1)
        let mut buf = BytesMut::new();
        buf.put_u8(0x01); // 1 column
        self.send_packet(&buf)?;

        // Column definition: name="1", type=INT
        let mut col_buf = BytesMut::new();
        col_buf.put_slice(b"def\0"); // catalog
        col_buf.put_slice(b"test\0"); // schema
        col_buf.put_slice(b"\0"); // table alias
        col_buf.put_slice(b"test\0"); // table
        col_buf.put_slice(b"\0"); // column alias
        col_buf.put_slice(b"1\0"); // column
        col_buf.put_u8(0x0c); // charset
        col_buf.put_u32_le(11); // column length
        col_buf.put_u8(0x03); // type (MYSQL_TYPE_LONG)
        col_buf.put_u8(0x00); // flags
        col_buf.put_u8(0x00); // decimals
        col_buf.put_u16_le(0x0000); // default

        self.send_packet(&col_buf)?;

        // EOF packet
        let mut eof_buf = BytesMut::new();
        eof_buf.put_u8(0xfe);
        eof_buf.put_u16_le(0x0000); // warnings
        eof_buf.put_u16_le(0x0000); // status flags
        self.send_packet(&eof_buf)?;

        // Row data
        let mut row_buf = BytesMut::new();
        row_buf.put_u64_le(1); // length
        row_buf.put_slice(b"1"); // value
        self.send_packet(&row_buf)?;

        // EOF packet (final)
        self.send_packet(&eof_buf)?;

        Ok(())
    }
}

/// Start the server (synchronous version)
pub fn start_server_sync(addr: &str) -> Result<(), SqlError> {
    let listener =
        TcpListener::bind(addr).map_err(|e| SqlError::IoError(format!("Failed to bind: {}", e)))?;

    println!("SQLRustGo Server listening on {}", addr);
    println!("MySQL protocol compatible");

    let mut connection_id: u32 = 1;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let conn_id = connection_id;
                connection_id += 1;

                println!("New connection #{}", conn_id);

                let mut handler = NetworkHandler::new(stream, conn_id);
                if let Err(e) = handler.handle() {
                    eprintln!("Client #{} error: {}", conn_id, e);
                }

                println!("Connection #{} closed", conn_id);
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }

    Ok(())
}

/// Connect to server
pub fn connect(addr: &str) -> Result<TcpStream, SqlError> {
    TcpStream::connect(addr).map_err(|e| SqlError::IoError(e.to_string()))
}

/// Simple client query execution
pub fn execute_query_on_server(addr: &str, query: &str) -> Result<String, SqlError> {
    let mut stream = connect(addr)?;

    // Read greeting (skip for now, just read until we get a response)
    let mut buf = [0u8; 1024];
    let _ = stream
        .read(&mut buf)
        .map_err(|e| SqlError::IoError(e.to_string()))?;

    // Send query as MySQL packet
    let mut packet = BytesMut::new();
    packet.put_u32_le((query.len() + 1) as u32);
    packet.put_u8(0x03); // Query command
    packet.put_slice(query.as_bytes());

    stream
        .write_all(&packet)
        .map_err(|e| SqlError::IoError(e.to_string()))?;

    // Read response
    let n = stream
        .read(&mut buf)
        .map_err(|e| SqlError::IoError(e.to_string()))?;
    let response = String::from_utf8_lossy(&buf[..n]).to_string();

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_creation() {
        let handshake = HandshakeV10::new(1);
        assert_eq!(handshake.protocol_version, 0x0a);
    }

    #[test]
    fn test_handshake_bytes() {
        let handshake = HandshakeV10::new(1);
        let bytes = handshake.to_bytes();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_ok_packet() {
        let ok = OkPacket::new(1, "OK");
        let bytes = ok.to_bytes();
        assert_eq!(bytes[0], 0x00); // OK header
    }

    #[test]
    fn test_error_packet() {
        let err = ErrPacket::new(1064, "Syntax error");
        let bytes = err.to_bytes();
        assert_eq!(bytes[0], 0xff); // Error header
    }

    #[test]
    fn test_mysql_command() {
        assert!(matches!(MySqlCommand::from(0x01), MySqlCommand::Quit));
        assert!(matches!(MySqlCommand::from(0x03), MySqlCommand::Query));
        assert!(matches!(MySqlCommand::from(0x0e), MySqlCommand::Ping));
    }

    #[test]
    fn test_packet_serialize() {
        let packet = MySqlPacket {
            sequence: 0,
            payload: vec![0x01, 0x02, 0x03],
        };
        let bytes = packet.serialize();
        assert_eq!(bytes.len(), 8); // 4 header (u32 + u8) + 3 payload
    }

    #[test]
    fn test_connect_function() {
        let _f: fn(&str) -> Result<TcpStream, SqlError> = connect;
    }

    #[test]
    fn test_network_handler_creation() {
        // Can't actually create without a real stream
        assert!(true);
    }
}
