//! Async Server Module - Asynchronous server implementation using Tokio
//!
//! Provides async TCP server with connection pooling and session management.

use crate::types::{SqlError, Value};
use bytes::{BufMut, BytesMut};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, RwLockWriteGuard};

/// Async server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub database: String,
    pub max_connections: usize,
    pub connection_timeout_secs: u64,
    pub verbose: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:3306".to_string(),
            database: "./data".to_string(),
            max_connections: 100,
            connection_timeout_secs: 30,
            verbose: false,
        }
    }
}

/// Connection pool for managing server connections
pub struct ConnectionPool {
    connections: RwLock<HashMap<u32, Connection>>,
    max_size: usize,
    next_id: RwLock<u32>,
}

impl ConnectionPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            max_size,
            next_id: RwLock::new(1),
        }
    }

    pub async fn add_connection(&self, conn: Connection) -> Result<u32, SqlError> {
        let mut connections = self.connections.write().await;
        if connections.len() >= self.max_size {
            return Err(SqlError::ProtocolError("Connection pool full".to_string()));
        }

        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;

        connections.insert(id, conn);
        Ok(id)
    }

    pub async fn remove_connection(&self, id: u32) {
        let mut connections = self.connections.write().await;
        connections.remove(&id);
    }

    pub async fn get_connection(&self, id: u32) -> Option<Connection> {
        let connections = self.connections.read().await;
        connections.get(&id).cloned()
    }

    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    pub async fn clear(&self) {
        let mut connections = self.connections.write().await;
        connections.clear();
    }
}

/// A connection in the pool
#[derive(Debug, Clone)]
pub struct Connection {
    pub id: u32,
    pub address: SocketAddr,
    pub created_at: std::time::Instant,
    pub last_activity: std::time::Instant,
    pub query_count: u64,
    pub is_active: bool,
}

impl Connection {
    pub fn new(id: u32, address: SocketAddr) -> Self {
        let now = std::time::Instant::now();
        Self {
            id,
            address,
            created_at: now,
            last_activity: now,
            query_count: 0,
            is_active: true,
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = std::time::Instant::now();
        self.query_count += 1;
    }
}

/// Session manager for handling client sessions
pub struct SessionManager {
    sessions: RwLock<HashMap<u32, Session>>,
    default_session_id: RwLock<u32>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            default_session_id: RwLock::new(0),
        }
    }

    pub async fn create_session(&self, connection_id: u32) -> u32 {
        let mut sessions = self.sessions.write().await;
        let mut next_id = self.default_session_id.write().await;
        let session_id = *next_id;
        *next_id += 1;

        sessions.insert(session_id, Session::new(session_id, connection_id));
        session_id
    }

    pub async fn get_session(&self, session_id: u32) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).cloned()
    }

    pub async fn remove_session(&self, session_id: u32) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(&session_id);
    }

    pub async fn update_session(&self, session_id: u32, session: Session) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session);
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A client session
#[derive(Debug, Clone)]
pub struct Session {
    pub id: u32,
    pub connection_id: u32,
    pub current_database: Option<String>,
    pub variables: HashMap<String, String>,
    pub created_at: std::time::Instant,
}

impl Session {
    pub fn new(id: u32, connection_id: u32) -> Self {
        Self {
            id,
            connection_id,
            current_database: None,
            variables: HashMap::new(),
            created_at: std::time::Instant::now(),
        }
    }

    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    pub fn get_variable(&self, key: &str) -> Option<String> {
        self.variables.get(key).cloned()
    }
}

/// Start the async server
pub async fn start_server_async(config: ServerConfig) -> Result<(), SqlError> {
    let listener = TcpListener::bind(&config.address)
        .await
        .map_err(|e| SqlError::IoError(format!("Failed to bind to {}: {}", config.address, e)))?;

    println!("Async server listening on {}", config.address);
    println!("Max connections: {}", config.max_connections);

    let pool = Arc::new(ConnectionPool::new(config.max_connections));
    let session_manager = Arc::new(SessionManager::new());

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let config = config.clone();
                let pool = pool.clone();
                let session_manager = session_manager.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_async_connection(stream, addr, &config, &pool, &session_manager).await {
                        if config.verbose {
                            eprintln!("Connection error: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                if config.verbose {
                    eprintln!("Accept error: {}", e);
                }
            }
        }
    }
}

/// Handle an async connection
async fn handle_async_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    config: &ServerConfig,
    pool: &Arc<ConnectionPool>,
    session_manager: &Arc<SessionManager>,
) -> Result<(), SqlError> {
    let connection_id = pool
        .add_connection(Connection::new(0, addr))
        .await
        .map_err(|e| SqlError::IoError(e.to_string()))?;

    if config.verbose {
        println!("New async connection #{} from {}", connection_id, addr);
    }

    // Send greeting
    send_greeting_async(&mut stream, connection_id).await?;

    // Create session
    let session_id = session_manager.create_session(connection_id).await;

    // Read and handle packets
    let mut sequence: u8 = 0;
    loop {
        match read_packet_async(&mut stream).await {
            Ok(Some((seq, payload))) => {
                sequence = seq;

                if payload.is_empty() {
                    continue;
                }

                let command = crate::network::MySqlCommand::from(payload[0]);

                match command {
                    crate::network::MySqlCommand::Quit => {
                        break;
                    }
                    crate::network::MySqlCommand::Query => {
                        let query = String::from_utf8_lossy(&payload[1..]).to_string();
                        if config.verbose {
                            println!("[{}] Query: {}", connection_id, query);
                        }
                        execute_query_async(&mut stream, &query, &mut sequence).await?;
                    }
                    crate::network::MySqlCommand::Ping => {
                        send_ok_async(&mut stream, "PONG", 0, &mut sequence).await?;
                    }
                    _ => {
                        send_error_async(&mut stream, 1047, "Command not supported", &mut sequence).await?;
                    }
                }

                // Update connection activity
                if let Some(mut conn) = pool.get_connection(connection_id).await {
                    conn.update_activity();
                }
            }
            Ok(None) => break,
            Err(e) => {
                return Err(e);
            }
        }
    }

    // Clean up
    pool.remove_connection(connection_id).await;
    session_manager.remove_session(session_id).await;

    if config.verbose {
        println!("Connection #{} closed", connection_id);
    }

    Ok(())
}

/// Send MySQL handshake greeting (async)
async fn send_greeting_async(
    stream: &mut TcpStream,
    connection_id: u32,
) -> Result<(), SqlError> {
    let greeting = crate::network::HandshakeV10::new(connection_id);
    let packet_data = greeting.to_bytes();

    let mut buf = BytesMut::new();
    buf.put_u8((packet_data.len() & 0xFF) as u8);
    buf.put_u8(((packet_data.len() >> 8) & 0xFF) as u8);
    buf.put_u8(((packet_data.len() >> 16) & 0xFF) as u8);
    buf.put_u8(0);
    buf.put_slice(&packet_data);

    stream
        .write_all(&buf)
        .await
        .map_err(|e| SqlError::IoError(e.to_string()))?;
    stream
        .flush()
        .await
        .map_err(|e| SqlError::IoError(e.to_string()))?;

    Ok(())
}

/// Read a MySQL packet (async)
async fn read_packet_async(stream: &mut TcpStream) -> Result<Option<(u8, Vec<u8>)>, SqlError> {
    const PACKET_HEADER_SIZE: usize = 4;

    let mut header = [0u8; PACKET_HEADER_SIZE];

    match stream.read(&mut header).await {
        Ok(0) => Ok(None),
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
                match stream.read(&mut payload[offset..]).await {
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

/// Execute a query (async)
async fn execute_query_async(
    stream: &mut TcpStream,
    query: &str,
    sequence: &mut u8,
) -> Result<(), SqlError> {
    use crate::{parse, ExecutionEngine};

    let trimmed = query.trim();

    match parse(trimmed) {
        Ok(statement) => {
            let mut engine = ExecutionEngine::new();
            match engine.execute(statement) {
                Ok(result) => {
                    if result.rows.is_empty() {
                        let message = format!("{} row(s) affected", result.rows_affected);
                        send_ok_async(stream, &message, result.rows_affected, sequence).await?;
                    } else {
                        send_result_set_async(stream, &result, sequence).await?;
                    }
                }
                Err(e) => {
                    send_error_async(stream, 1, &e.to_string(), sequence).await?;
                }
            }
        }
        Err(e) => {
            send_error_async(stream, 1064, &format!("Parse error: {}", e), sequence).await?;
        }
    }

    Ok(())
}

/// Send OK packet (async)
async fn send_ok_async(
    stream: &mut TcpStream,
    message: &str,
    affected_rows: u64,
    sequence: &mut u8,
) -> Result<(), SqlError> {
    let packet = crate::network::OkPacket::new(affected_rows, message);
    send_packet_async(stream, &packet.to_bytes(), sequence).await
}

/// Send error packet (async)
async fn send_error_async(
    stream: &mut TcpStream,
    code: u16,
    message: &str,
    sequence: &mut u8,
) -> Result<(), SqlError> {
    let packet = crate::network::ErrPacket::new(code, message);
    send_packet_async(stream, &packet.to_bytes(), sequence).await
}

/// Send result set (async)
async fn send_result_set_async(
    stream: &mut TcpStream,
    result: &crate::ExecutionResult,
    sequence: &mut u8,
) -> Result<(), SqlError> {
    let columns: Vec<&str> = if result.columns.is_empty() {
        vec!["column_0"]
    } else {
        result.columns.iter().map(|s| s.as_str()).collect()
    };

    // Column count
    {
        let mut buf = BytesMut::new();
        buf.put_u8(0x01);
        buf.put_u64_le(columns.len() as u64);
        send_packet_async(stream, &buf, sequence).await?;
    }

    // Column definitions
    for col in &columns {
        let mut col_buf = BytesMut::new();
        col_buf.put_slice(b"def\0");
        col_buf.put_slice(b"test\0");
        col_buf.put_slice(b"\0");
        col_buf.put_slice(b"test\0");
        col_buf.put_slice(b"\0");
        col_buf.put_slice(col.as_bytes());
        col_buf.put_u8(0);
        col_buf.put_u8(0x0c);
        col_buf.put_u32_le(256);
        col_buf.put_u8(0xfd);
        col_buf.put_u8(0x00);
        col_buf.put_u8(0x00);
        col_buf.put_u16_le(0x0000);
        send_packet_async(stream, &col_buf, sequence).await?;
    }

    // EOF (columns end)
    {
        let mut eof_buf = BytesMut::new();
        eof_buf.put_u8(0xfe);
        eof_buf.put_u16_le(0x0000);
        eof_buf.put_u16_le(0x0002);
        send_packet_async(stream, &eof_buf, sequence).await?;
    }

    // Row data
    for row in &result.rows {
        let mut row_buf = BytesMut::new();
        for value in row {
            match value {
                Value::Null => {
                    row_buf.put_u8(0xfb);
                }
                Value::Integer(i) => {
                    let s = i.to_string();
                    row_buf.put_u64_le(s.len() as u64);
                    row_buf.put_slice(s.as_bytes());
                }
                Value::Float(f) => {
                    let s = f.to_string();
                    row_buf.put_u64_le(s.len() as u64);
                    row_buf.put_slice(s.as_bytes());
                }
                Value::Text(s) => {
                    row_buf.put_u64_le(s.len() as u64);
                    row_buf.put_slice(s.as_bytes());
                }
                Value::Blob(b) => {
                    row_buf.put_u64_le(b.len() as u64);
                    row_buf.put_slice(b);
                }
                Value::Boolean(b) => {
                    let s = if *b { "1" } else { "0" };
                    row_buf.put_u64_le(1);
                    row_buf.put_slice(s.as_bytes());
                }
            }
        }
        send_packet_async(stream, &row_buf, sequence).await?;
    }

    // EOF (rows end)
    {
        let mut eof_buf = BytesMut::new();
        eof_buf.put_u8(0xfe);
        eof_buf.put_u16_le(0x0000);
        eof_buf.put_u16_le(0x000a);
        send_packet_async(stream, &eof_buf, sequence).await?;
    }

    Ok(())
}

/// Send a packet (async)
async fn send_packet_async(
    stream: &mut TcpStream,
    data: &[u8],
    sequence: &mut u8,
) -> Result<(), SqlError> {
    let mut buf = BytesMut::new();
    buf.put_u8((data.len() & 0xFF) as u8);
    buf.put_u8(((data.len() >> 8) & 0xFF) as u8);
    buf.put_u8(((data.len() >> 16) & 0xFF) as u8);
    buf.put_u8(*sequence);
    *sequence = sequence.wrapping_add(1);
    buf.put_slice(data);

    stream
        .write_all(&buf)
        .await
        .map_err(|e| SqlError::IoError(e.to_string()))?;
    stream
        .flush()
        .await
        .map_err(|e| SqlError::IoError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.address, "127.0.0.1:3306");
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_connection_new() {
        use std::net::SocketAddr;
        let addr: SocketAddr = "127.0.0.1:3306".parse().unwrap();
        let conn = Connection::new(1, addr);
        assert_eq!(conn.id, 1);
        assert!(conn.is_active);
    }

    #[test]
    fn test_connection_update_activity() {
        use std::net::SocketAddr;
        let addr: SocketAddr = "127.0.0.1:3306".parse().unwrap();
        let mut conn = Connection::new(1, addr);
        let initial_count = conn.query_count;
        conn.update_activity();
        assert_eq!(conn.query_count, initial_count + 1);
    }

    #[test]
    fn test_session_new() {
        let session = Session::new(1, 100);
        assert_eq!(session.id, 1);
        assert_eq!(session.connection_id, 100);
        assert!(session.current_database.is_none());
    }

    #[test]
    fn test_session_variables() {
        let mut session = Session::new(1, 100);
        session.set_variable("autocommit", "1");
        assert_eq!(session.get_variable("autocommit"), Some("1".to_string()));
    }

    #[tokio::test]
    async fn test_connection_pool() {
        use std::net::SocketAddr;
        let pool = ConnectionPool::new(2);
        let addr: SocketAddr = "127.0.0.1:3306".parse().unwrap();

        let id = pool.add_connection(Connection::new(1, addr)).await;
        assert!(id.is_ok());

        let count = pool.connection_count().await;
        assert_eq!(count, 1);

        pool.remove_connection(1).await;
        let count = pool.connection_count().await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_session_manager() {
        let manager = SessionManager::new();

        let session_id = manager.create_session(1).await;
        assert_eq!(session_id, 0);

        let session = manager.get_session(session_id).await;
        assert!(session.is_some());

        manager.remove_session(session_id).await;
        let session = manager.get_session(session_id).await;
        assert!(session.is_none());
    }
}
