//! MySQL server integration tests - test Packet I/O and MySqlError.

use sqlrustgo_mysql_server::{make_ok_packet, split_sql_statements, MySqlError, Packet};

#[test]
fn test_ok_packet_with_more_results() {
    let pkt = make_ok_packet(1, 5, 10, 0x0002 | 0x0008, 0);
    assert_eq!(pkt.sequence, 1);
    assert_eq!(pkt.payload[0], 0x00);
}

#[test]
fn test_ok_packet_without_more_results() {
    let pkt = make_ok_packet(1, 5, 10, 0x0002, 0);
    assert_eq!(pkt.sequence, 1);
    assert_eq!(pkt.payload[0], 0x00);
}

#[test]
fn test_split_sql_statements_single() {
    let sql = "SELECT 1";
    let results = split_sql_statements(sql);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "SELECT 1");
}

#[test]
fn test_split_sql_statements_multiple() {
    let sql = "SELECT 1; SELECT 2; SELECT 3";
    let results = split_sql_statements(sql);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], "SELECT 1");
    assert_eq!(results[1], "SELECT 2");
    assert_eq!(results[2], "SELECT 3");
}

#[test]
fn test_split_sql_statements_with_insert() {
    let sql = "INSERT INTO t VALUES(1); SELECT * FROM t";
    let results = split_sql_statements(sql);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], "INSERT INTO t VALUES(1)");
    assert_eq!(results[1], "SELECT * FROM t");
}

#[test]
fn test_split_sql_statements_with_whitespace() {
    let sql = "SELECT 1 ; SELECT 2 ";
    let results = split_sql_statements(sql);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], "SELECT 1");
    assert_eq!(results[1], "SELECT 2");
}

#[test]
fn test_split_sql_statements_empty() {
    let sql = "";
    let results = split_sql_statements(sql);
    assert!(results.is_empty());
}

#[test]
fn test_split_sql_statements_trims_whitespace() {
    let sql = "  SELECT 1  ;  SELECT 2  ";
    let results = split_sql_statements(sql);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], "SELECT 1");
    assert_eq!(results[1], "SELECT 2");
}

// ============ MySqlError Tests ============

#[test]
fn test_mysql_error_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = MySqlError::Io(io_err);
    let display = format!("{}", err);
    assert!(display.contains("IO error"));
}

#[test]
fn test_mysql_error_protocol() {
    let err = MySqlError::Protocol("bad handshake".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Protocol error"));
    assert!(display.contains("bad handshake"));
}

#[test]
fn test_mysql_error_sql() {
    let err = MySqlError::Sql("syntax error".to_string());
    let display = format!("{}", err);
    assert!(display.contains("SQL error"));
    assert!(display.contains("syntax error"));
}

#[test]
fn test_mysql_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let err: MySqlError = MySqlError::from(io_err);
    let display = format!("{}", err);
    assert!(display.contains("IO error"));
}

#[test]
fn test_mysql_error_from_string() {
    let err: MySqlError = MySqlError::Protocol("test string error".to_string());
    let display = format!("{}", err);
    assert!(display.contains("test string error"));
}

#[test]
fn test_mysql_error_from_str() {
    let err: MySqlError = MySqlError::Protocol("test str error".to_string());
    let display = format!("{}", err);
    assert!(display.contains("test str error"));
}

#[test]
fn test_mysql_error_debug() {
    let err = MySqlError::Protocol("debug test".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("Protocol"));
}

// ============ Packet Tests ============

#[test]
fn test_packet_struct() {
    let pkt = Packet {
        length: 10,
        sequence: 5,
        payload: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    };
    assert_eq!(pkt.length, 10);
    assert_eq!(pkt.sequence, 5);
    assert_eq!(pkt.payload.len(), 10);
}

#[test]
fn test_packet_write_and_read_roundtrip() {
    let original = Packet {
        length: 3,
        sequence: 1,
        payload: vec![0x01, 0x02, 0x03],
    };

    let mut buf = Vec::new();
    original.write_to(&mut buf).unwrap();

    // Read it back
    let mut cursor = std::io::Cursor::new(buf);
    let read = Packet::read_from(&mut cursor).unwrap();

    assert_eq!(read.length, original.length);
    assert_eq!(read.sequence, original.sequence);
    assert_eq!(read.payload, original.payload);
}

#[test]
fn test_packet_roundtrip_larger() {
    let original = Packet {
        length: 255,
        sequence: 3,
        payload: (0..255).collect(),
    };

    let mut buf = Vec::new();
    original.write_to(&mut buf).unwrap();

    let mut cursor = std::io::Cursor::new(buf);
    let read = Packet::read_from(&mut cursor).unwrap();

    assert_eq!(read.length, original.length);
    assert_eq!(read.sequence, original.sequence);
    assert_eq!(read.payload, original.payload);
}

#[test]
fn test_packet_roundtrip_empty() {
    let original = Packet {
        length: 0,
        sequence: 0,
        payload: vec![],
    };

    let mut buf = Vec::new();
    original.write_to(&mut buf).unwrap();

    let mut cursor = std::io::Cursor::new(buf);
    let read = Packet::read_from(&mut cursor).unwrap();

    assert_eq!(read.length, 0);
    assert!(read.payload.is_empty());
}

#[test]
fn test_packet_sequence_numbers() {
    for seq in [0u8, 1, 127, 255] {
        let pkt = Packet {
            length: 5,
            sequence: seq,
            payload: vec![1, 2, 3, 4, 5],
        };
        let mut buf = Vec::new();
        pkt.write_to(&mut buf).unwrap();
        let mut cursor = std::io::Cursor::new(buf);
        let read = Packet::read_from(&mut cursor).unwrap();
        assert_eq!(read.sequence, seq);
    }
}

#[test]
fn test_packet_payload_various_bytes() {
    // Test various byte values including boundaries
    let payload: Vec<u8> = vec![0x00, 0x7F, 0x80, 0xFF, b'\n', b'\t', 0x00];
    let pkt = Packet {
        length: payload.len() as u32,
        sequence: 0,
        payload: payload.clone(),
    };

    let mut buf = Vec::new();
    pkt.write_to(&mut buf).unwrap();

    let mut cursor = std::io::Cursor::new(buf);
    let read = Packet::read_from(&mut cursor).unwrap();
    assert_eq!(read.payload, payload);
}

// ============ Transaction Command Tests ============

use sqlrustgo_mysql_server::{parse_transaction_command, SessionState, TransactionCommand};

#[test]
fn test_transaction_cmd_begin() {
    assert_eq!(
        parse_transaction_command("BEGIN"),
        TransactionCommand::Begin
    );
    assert_eq!(
        parse_transaction_command("begin"),
        TransactionCommand::Begin
    );
    assert_eq!(
        parse_transaction_command("  BEGIN  "),
        TransactionCommand::Begin
    );
    assert_eq!(
        parse_transaction_command("START TRANSACTION"),
        TransactionCommand::Begin
    );
    assert_eq!(
        parse_transaction_command("start transaction"),
        TransactionCommand::Begin
    );
}

#[test]
fn test_transaction_cmd_commit() {
    assert_eq!(
        parse_transaction_command("COMMIT"),
        TransactionCommand::Commit
    );
    assert_eq!(
        parse_transaction_command("commit"),
        TransactionCommand::Commit
    );
    assert_eq!(
        parse_transaction_command("  COMMIT  "),
        TransactionCommand::Commit
    );
}

#[test]
fn test_transaction_cmd_rollback() {
    assert_eq!(
        parse_transaction_command("ROLLBACK"),
        TransactionCommand::Rollback
    );
    assert_eq!(
        parse_transaction_command("rollback"),
        TransactionCommand::Rollback
    );
    assert_eq!(
        parse_transaction_command("  ROLLBACK  "),
        TransactionCommand::Rollback
    );
}

#[test]
fn test_transaction_cmd_non_transaction() {
    assert_eq!(
        parse_transaction_command("SELECT 1"),
        TransactionCommand::None
    );
    assert_eq!(
        parse_transaction_command("INSERT INTO t VALUES(1)"),
        TransactionCommand::None
    );
    assert_eq!(parse_transaction_command(""), TransactionCommand::None);
}

#[test]
fn test_session_state_transaction_flow() {
    let mut session = SessionState::default();
    assert!(!session.transaction_active);

    match parse_transaction_command("BEGIN") {
        TransactionCommand::Begin => session.transaction_active = true,
        _ => panic!("expected BEGIN"),
    }
    assert!(session.transaction_active);

    match parse_transaction_command("COMMIT") {
        TransactionCommand::Commit => session.transaction_active = false,
        _ => panic!("expected COMMIT"),
    }
    assert!(!session.transaction_active);

    match parse_transaction_command("BEGIN") {
        TransactionCommand::Begin => session.transaction_active = true,
        _ => panic!("expected BEGIN"),
    }
    assert!(session.transaction_active);

    match parse_transaction_command("ROLLBACK") {
        TransactionCommand::Rollback => session.transaction_active = false,
        _ => panic!("expected ROLLBACK"),
    }
    assert!(!session.transaction_active);
}
