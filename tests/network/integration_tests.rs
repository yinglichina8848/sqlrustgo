//! Network Integration Tests
//!
//! These tests verify complete client-server interaction cycles
//! without requiring real network connections.

use sqlrustgo::network::mocks::{MockTcpStream, mock_query_packet};
use sqlrustgo::network::{NetworkHandler, MySqlCommand};
use std::io::{Read, Write};

/// Helper to create a connected mock client-server pair
fn create_mock_connection() -> (NetworkHandler<MockTcpStream>, MockTcpStream) {
    let stream = MockTcpStream::new();
    let handler = NetworkHandler::new(stream.clone(), 1);
    (handler, stream)
}

/// Helper to read response from mock stream
fn read_response(stream: &mut MockTcpStream) -> Vec<u8> {
    let written = stream.get_written().to_vec();
    stream.clear();
    written
}

#[cfg(test)]
mod client_server_tests {
    use super::*;

    #[test]
    fn test_query_response_cycle() {
        let (mut handler, mut stream) = create_mock_connection();
        
        // Client sends query
        let query = b"SELECT 1";
        stream.write(mock_query_packet("SELECT 1").as_slice()).unwrap();
        
        // Handler processes query
        let result = handler.handle();
        assert!(result.is_ok());
        
        // Verify response written
        let response = read_response(&mut stream);
        assert!(!response.is_empty());
    }

    #[test]
    fn test_multiple_queries() {
        let (mut handler, mut stream) = create_mock_connection();
        
        // First query
        stream.write(mock_query_packet("SELECT 1").as_slice()).unwrap();
        let _ = handler.handle();
        let response1 = read_response(&mut stream);
        
        // Second query
        stream.write(mock_query_packet("SELECT 2").as_slice()).unwrap();
        let _ = handler.handle();
        let response2 = read_response(&mut stream);
        
        // Both queries should produce responses
        assert!(!response1.is_empty());
        assert!(!response2.is_empty());
    }

    #[test]
    fn test_ping_command() {
        let (mut handler, mut stream) = create_mock_connection();
        
        // Send PING command (0x0e)
        let mut ping = vec![0x01, 0x00, 0x00, 0x00, 0x0e]; // packet header + PING
        stream.write(&ping).unwrap();
        
        let result = handler.handle();
        assert!(result.is_ok());
        
        let response = read_response(&mut stream);
        assert!(!response.is_empty());
    }

    #[test]
    fn test_quit_command() {
        let (mut handler, mut stream) = create_mock_connection();
        
        // Send QUIT command (0x01)
        let mut quit = vec![0x01, 0x00, 0x00, 0x00, 0x01]; // packet header + QUIT
        stream.write(&quit).unwrap();
        
        // Handler should handle quit gracefully
        let result = handler.handle();
        assert!(result.is_ok());
    }

    #[test]
    fn test_select_version() {
        let (mut handler, mut stream) = create_mock_connection();
        
        stream.write(mock_query_packet("SELECT VERSION()").as_slice()).unwrap();
        let _ = handler.handle();
        
        let response = read_response(&mut stream);
        assert!(!response.is_empty());
    }

    #[test]
    fn test_select_1_as_a() {
        let (mut handler, mut stream) = create_mock_connection();
        
        stream.write(mock_query_packet("SELECT 1 AS a").as_slice()).unwrap();
        let _ = handler.handle();
        
        let response = read_response(&mut stream);
        assert!(!response.is_empty());
    }

    #[test]
    fn test_empty_query() {
        let (mut handler, mut stream) = create_mock_connection();
        
        // Empty query should be handled
        let mut empty = vec![0x01, 0x00, 0x00, 0x00, 0x03]; // packet header + empty query
        stream.write(&empty).unwrap();
        
        let result = handler.handle();
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod packet_parsing_tests {
    use super::*;

    #[test]
    fn test_query_packet_parsing() {
        let packet = mock_query_packet("SELECT 1");
        
        // Verify packet structure
        assert_eq!(packet[0], 9); // length: "SELECT 1".len() + 1 = 8 + 1 = 9
        assert_eq!(packet[4], 0x03); // COM_QUERY command
        assert_eq!(&packet[5..], b"SELECT 1");
    }

    #[test]
    fn test_packet_header_format() {
        let query = "SELECT * FROM users WHERE id = 1";
        let packet = mock_query_packet(query);
        
        // Header is first 4 bytes: [length_low, length_mid, length_high, sequence]
        let len = packet.len() - 4; // actual payload length
        assert_eq!(packet[0], (len & 0xff) as u8);
        assert_eq!(packet[1], ((len >> 8) & 0xff) as u8);
        assert_eq!(packet[2], ((len >> 16) & 0xff) as u8);
    }
}
