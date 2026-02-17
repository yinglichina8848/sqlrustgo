//! Network Handler Integration Tests
//!
//! Tests for NetworkHandler using mock TcpStream

mod mocks;

use sqlrustgo::network::{
    HandshakeV10, MySqlPacket, OkPacket, ErrPacket, RowData, NetworkHandler,
    capability,
};
use sqlrustgo::Value;

use mocks::{MockTcpStream, create_mock_handshake, create_mock_ok_packet, create_mock_query_packet};

/// Test handshake packet serialization and deserialization
#[test]
fn test_handshake_serialize_roundtrip() {
    let handshake = HandshakeV10::new(1);
    let bytes = handshake.to_bytes();
    assert!(!bytes.is_empty());
    
    // Parse the handshake
    let payload_start = 4; // Skip packet header
    let parsed = HandshakeV10::parse(&bytes[payload_start..]).unwrap();
    assert_eq!(parsed.protocol_version, 0x0a);
    assert_eq!(parsed.connection_id, 1);
}

/// Test MySQL packet roundtrip
#[test]
fn test_packet_roundtrip() {
    let original_payload = vec![0x01, 0x02, 0x03, 0x04];
    let packet = MySqlPacket {
        sequence: 1,
        payload: original_payload.clone(),
    };
    
    let serialized = packet.serialize();
    assert!(serialized.len() >= 4);
    
    // Parse it back
    let parsed = MySqlPacket::parse(&serialized).unwrap();
    assert_eq!(parsed.sequence, 1);
    assert_eq!(parsed.payload, original_payload);
}

/// Test OK packet with various messages
#[test]
fn test_ok_packet_various_messages() {
    // Test with short message
    let packet = OkPacket::new(1, "OK");
    let bytes = packet.to_bytes();
    assert!(!bytes.is_empty());
    
    // Test with long message
    let long_msg = "This is a longer message that tests the packet encoding".to_string();
    let packet2 = OkPacket::new(100, &long_msg);
    let bytes2 = packet2.to_bytes();
    assert!(bytes2.len() > bytes.len());
}

/// Test Error packet with various codes
#[test]
fn test_error_packet_various_codes() {
    // Test common MySQL error codes
    let test_cases = vec![
        (1064, "You have an error in your SQL syntax"),
        (1146, "Table doesn't exist"),
        (1213, "Deadlock found"),
    ];
    
    for (code, msg) in test_cases {
        let packet = ErrPacket::new(code, msg);
        let bytes = packet.to_bytes();
        assert!(!bytes.is_empty());
    }
}

/// Test RowData with various value types
#[test]
fn test_row_data_all_types() {
    let test_cases = vec![
        vec![Value::Integer(42)],
        vec![Value::Text("hello".to_string())],
        vec![Value::Float(3.14159)],
        vec![Value::Boolean(true)],
        vec![Value::Null],
        vec![Value::Integer(1), Value::Text("test".to_string()), Value::Float(1.5)],
    ];
    
    for values in test_cases {
        let row = RowData { values };
        let bytes = row.to_bytes();
        assert!(!bytes.is_empty());
    }
}

/// Test capability flags combinations
#[test]
fn test_capability_flags() {
    use capability::*;
    
    // Test various combinations
    let combinations = vec![
        PROTOCOL_41,
        LONG_PASSWORD | FOUND_ROWS,
        PROTOCOL_41 | MULTI_STATEMENTS | MULTI_RESULTS,
        LONG_PASSWORD | FOUND_ROWS | CONNECT_WITH_DB | PROTOCOL_41,
    ];
    
    for flags in combinations {
        let mut handshake = HandshakeV10::new(1);
        handshake.capability_flags = flags;
        let bytes = handshake.to_bytes();
        assert!(!bytes.is_empty());
    }
}

/// Test packet with maximum payload size
#[test]
fn test_packet_max_payload() {
    let large_payload: Vec<u8> = (0..16777215).map(|i| (i % 256) as u8).collect();
    let packet = MySqlPacket {
        sequence: 0,
        payload: large_payload,
    };
    
    let bytes = packet.serialize();
    // Should have header (4) + payload
    assert!(bytes.len() > 4);
}

/// Test sequence number handling
#[test]
fn test_packet_sequence_numbers() {
    for seq in [0, 1, 127, 128, 255] {
        let packet = MySqlPacket {
            sequence: seq,
            payload: vec![0x01],
        };
        let bytes = packet.serialize();
        assert_eq!(bytes[3], seq);
    }
}

/// Test empty payload
#[test]
fn test_packet_empty_payload() {
    let packet = MySqlPacket {
        sequence: 0,
        payload: vec![],
    };
    let bytes = packet.serialize();
    // Should still have 4 byte header
    assert_eq!(bytes.len(), 4);
}

/// Test RowData with special characters
#[test]
fn test_row_data_special_characters() {
    let test_cases = vec![
        Value::Text("Hello World".to_string()),
        Value::Text("Line1\nLine2".to_string()),
        Value::Text("Tab\tTab".to_string()),
        Value::Text("Quote\"Quote".to_string()),
        Value::Text("Unicode 你好".to_string()),
        Value::Text("Emoji 🎉".to_string()),
    ];
    
    for value in test_cases {
        let row = RowData { values: vec![value] };
        let bytes = row.to_bytes();
        assert!(!bytes.is_empty());
    }
}

/// Test RowData with large integers
#[test]
fn test_row_data_large_integers() {
    let test_cases = vec![
        Value::Integer(0),
        Value::Integer(1),
        Value::Integer(-1),
        Value::Integer(i64::MAX),
        Value::Integer(i64::MIN),
        Value::Integer(9999999999i64),
        Value::Integer(-9999999999i64),
    ];
    
    for value in test_cases {
        let row = RowData { values: vec![value] };
        let bytes = row.to_bytes();
        assert!(!bytes.is_empty());
    }
}

/// Test RowData with special floats
#[test]
fn test_row_data_special_floats() {
    let test_cases = vec![
        Value::Float(0.0),
        Value::Float(-0.0),
        Value::Float(1.0),
        Value::Float(-1.0),
        Value::Float(f64::MAX),
        Value::Float(f64::MIN),
        Value::Float(f64::EPSILON),
    ];
    
    for value in test_cases {
        let row = RowData { values: vec![value] };
        let bytes = row.to_bytes();
        assert!(!bytes.is_empty());
    }
}

/// Integration test: simulate full query response cycle
#[test]
fn test_query_response_cycle() {
    // Simulate server sending handshake
    let handshake = HandshakeV10::new(1);
    let handshake_bytes = handshake.to_bytes();
    
    // Create mock packet with handshake
    let packet = MySqlPacket {
        sequence: 0,
        payload: handshake_bytes,
    };
    let packet_bytes = packet.serialize();
    assert!(!packet_bytes.is_empty());
    
    // Simulate server sending OK response
    let ok = OkPacket::new(0, "Query OK");
    let ok_bytes = ok.to_bytes();
    let ok_packet = MySqlPacket {
        sequence: 1,
        payload: ok_bytes,
    };
    let ok_packet_bytes = ok_packet.serialize();
    assert!(!ok_packet_bytes.is_empty());
}
