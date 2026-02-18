//! Mock TcpStream for network testing
//!
//! Provides mock implementations of TcpStream and TcpListener for testing network functionality

use std::io::{Read, Write, Result as IoResult};
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// Mock TcpStream for testing network code
pub struct MockTcpStream {
    read_buffer: Arc<Mutex<Vec<u8>>>,
    write_buffer: Arc<Mutex<Vec<u8>>>,
    closed: Arc<Mutex<bool>>,
}

impl MockTcpStream {
    /// Create a new MockTcpStream
    pub fn new() -> Self {
        Self {
            read_buffer: Arc::new(Mutex::new(Vec::new())),
            write_buffer: Arc::new(Mutex::new(Vec::new())),
            closed: Arc::new(Mutex::new(false)),
        }
    }

    /// Create with preset read data
    pub fn with_read_data(data: Vec<u8>) -> Self {
        Self {
            read_buffer: Arc::new(Mutex::new(data)),
            write_buffer: Arc::new(Mutex::new(Vec::new())),
            closed: Arc::new(Mutex::new(false)),
        }
    }

    /// Get data written to the stream
    pub fn get_written_data(&self) -> Vec<u8> {
        self.write_buffer.lock().unwrap().clone()
    }

    /// Set data to be read from the stream
    pub fn set_read_data(&self, data: Vec<u8>) {
        *self.read_buffer.lock().unwrap() = data;
    }

    /// Check if stream is closed
    pub fn is_closed(&self) -> bool {
        *self.closed.lock().unwrap()
    }
}

impl Default for MockTcpStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Read for MockTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        if *self.closed.lock().unwrap() {
            return Err(IoError::new(ErrorKind::ConnectionReset, "Stream closed"));
        }
        
        let mut data = self.read_buffer.lock().unwrap();
        if data.is_empty() {
            return Err(IoError::new(ErrorKind::WouldBlock, "No data available"));
        }
        
        let len = std::cmp::min(buf.len(), data.len());
        buf[..len].copy_from_slice(&data[..len]);
        data.drain(..len);
        Ok(len)
    }
}

impl Write for MockTcpStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        if *self.closed.lock().unwrap() {
            return Err(IoError::new(ErrorKind::ConnectionReset, "Stream closed"));
        }
        
        self.write_buffer.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

/// Mock TcpListener for testing server code
pub struct MockTcpListener {
    incoming: Arc<Mutex<Vec<MockTcpStream>>>,
    bind_error: Option<IoError>,
}

impl MockTcpListener {
    /// Create a new MockTcpListener
    pub fn new() -> Self {
        Self {
            incoming: Arc::new(Mutex::new(Vec::new())),
            bind_error: None,
        }
    }

    /// Create with preset connections
    pub fn with_connections(connections: Vec<MockTcpStream>) -> Self {
        Self {
            incoming: Arc::new(Mutex::new(connections)),
            bind_error: None,
        }
    }

    /// Set bind error
    pub fn with_bind_error(error: IoError) -> Self {
        Self {
            incoming: Arc::new(Mutex::new(Vec::new())),
            bind_error: Some(error),
        }
    }

    /// Add a connection to be returned
    pub fn add_connection(&self, stream: MockTcpStream) {
        self.incoming.lock().unwrap().push(stream);
    }

    /// Get local address
    pub fn local_addr(&self) -> IoResult<SocketAddr> {
        Ok("127.0.0.1:3306".parse().unwrap())
    }

    /// Accept incoming connection
    pub fn accept(&self) -> IoResult<(MockTcpStream, SocketAddr)> {
        let mut incoming = self.incoming.lock().unwrap();
        match incoming.pop() {
            Some(stream) => Ok((stream, "127.0.0.1:12345".parse().unwrap())),
            None => Err(IoError::new(ErrorKind::ConnectionRefused, "No connections")),
        }
    }
}

impl Default for MockTcpListener {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator for incoming connections
pub struct Incoming<'a> {
    listener: &'a MockTcpListener,
}

impl<'a> Iterator for Incoming<'a> {
    type Item = IoResult<MockTcpStream>;

    fn next(&mut self) -> Option<Self::Item> {
        self.listener.incoming.lock().unwrap().pop().map(Ok)
    }
}

impl MockTcpListener {
    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }
}

/// Helper to create mock MySQL handshake packet
pub fn create_mock_handshake() -> Vec<u8> {
    let mut packet = Vec::new();
    // Packet header (payload length = 62)
    packet.extend_from_slice(&[0x3e, 0x00, 0x00, 0x00]);
    // Protocol version
    packet.push(0x0a);
    // Server version
    packet.extend_from_slice(b"1.0.0-SQLRustGo\0");
    // Connection ID
    packet.extend_from_slice(&1u32.to_le_bytes());
    // Auth plugin data part 1
    packet.extend_from_slice(&[0x00; 8]);
    packet.push(0x00);
    // Capability flags (lower 2 bytes)
    packet.extend_from_slice(&0x85a2u16.to_le_bytes());
    // Character set
    packet.push(0x21);
    // Status flags
    packet.extend_from_slice(&0x0002u16.to_le_bytes());
    // Capability flags (upper 2 bytes)
    packet.extend_from_slice(&0x0200u16.to_le_bytes());
    // Auth plugin data length
    packet.push(0x00);
    // Reserved
    packet.extend_from_slice(&[0x00; 10]);
    // Auth plugin name
    packet.extend_from_slice(b"mysql_native_password\0");
    packet
}

/// Helper to create mock MySQL OK packet
pub fn create_mock_ok_packet() -> Vec<u8> {
    let mut packet = Vec::new();
    // Packet header
    packet.extend_from_slice(&[0x07, 0x00, 0x00, 0x01]);
    // Affected rows
    packet.push(0x00);
    // Last insert ID
    packet.push(0x00);
    // Status flags
    packet.extend_from_slice(&0x0002u16.to_le_bytes());
    // Warnings
    packet.extend_from_slice(&0x0000u16.to_le_bytes());
    packet
}

/// Helper to create mock MySQL query packet
pub fn create_mock_query_packet(query: &str) -> Vec<u8> {
    let mut packet = Vec::new();
    // Packet header
    let payload_len = query.len() + 1;
    packet.extend_from_slice(&[(payload_len as u8), 0x00, 0x00, 0x00]);
    // COM_QUERY
    packet.push(0x03);
    // Query string
    packet.extend_from_slice(query.as_bytes());
    packet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_tcp_stream_write() {
        let mut stream = MockTcpStream::new();
        let data = b"Hello, World!";
        let written = stream.write(data).unwrap();
        assert_eq!(written, data.len());
        assert_eq!(stream.get_written_data(), data);
    }

    #[test]
    fn test_mock_handshake_packet() {
        let packet = create_mock_handshake();
        assert!(packet.len() > 4);
        assert_eq!(packet[0], 0x3e); // payload length
    }

    #[test]
    fn test_mock_ok_packet() {
        let packet = create_mock_ok_packet();
        assert!(packet.len() > 4);
    }

    #[test]
    fn test_mock_query_packet() {
        let packet = create_mock_query_packet("SELECT 1");
        assert!(packet.len() > 4);
    }

    #[test]
    fn test_mock_tcp_listener_new() {
        let listener = MockTcpListener::new();
        assert!(listener.local_addr().is_ok());
    }

    #[test]
    fn test_mock_tcp_listener_accept_empty() {
        let listener = MockTcpListener::new();
        let result = listener.accept();
        assert!(result.is_err());
    }
}
