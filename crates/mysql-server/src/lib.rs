//! SQLRustGo MySQL Wire Protocol Server
//!
//! Supports mysql_native_password auth + TLS (mariadb-connector-c 3.4+ compatible)

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rcgen::{CertificateParams, KeyPair};
use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::{SqlError, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;

const SERVER_VERSION: &str = "8.0.33-SQLRustGo";
#[allow(dead_code)]
const AUTH_PLUGIN: &str = "mysql_native_password";
const SCRAMBLE_LENGTH: usize = 20;
const SKIP_AUTH: bool = true;

mod packet_type {
    pub const COM_QUIT: u8 = 0x01;
    pub const COM_INIT_DB: u8 = 0x02;
    pub const COM_QUERY: u8 = 0x03;
    pub const COM_STMT_PREPARE: u8 = 0x16;
    pub const COM_PING: u8 = 0x0e;
}

mod capability {
    pub const LONG_PASSWORD: u32 = 0x00000001;
    pub const FOUND_ROWS: u32 = 0x00000002;
    pub const LONG_FLAG: u32 = 0x00000004;
    pub const CONNECT_WITH_DB: u32 = 0x00000008;
    pub const PROTOCOL_41: u32 = 0x00000200;
    pub const TRANSACTIONS: u32 = 0x00002000;
    pub const SECURE_CONNECTION: u32 = 0x00008000;
    pub const MULTI_STATEMENTS: u32 = 0x00010000;
    pub const MULTI_RESULTS: u32 = 0x00020000;
    pub const PLUGIN_AUTH: u32 = 0x00080000;
    pub const PLUGIN_AUTH_LENENC_CLIENT_DATA: u32 = 0x00200000;
    pub const SSL: u32 = 0x00000800;
    pub const DEPRECATE_EOF: u32 = 0x01000000;

    pub const SERVER_DEFAULT: u32 = LONG_PASSWORD
        | FOUND_ROWS
        | LONG_FLAG
        | CONNECT_WITH_DB
        | PROTOCOL_41
        | TRANSACTIONS
        | SECURE_CONNECTION
        | MULTI_STATEMENTS
        | MULTI_RESULTS
        | PLUGIN_AUTH_LENENC_CLIENT_DATA
        | SSL;
}

#[derive(Debug)]
pub enum MySqlError {
    Io(std::io::Error),
    Protocol(String),
    Sql(String),
}

impl std::fmt::Display for MySqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MySqlError::Io(e) => write!(f, "IO: {}", e),
            MySqlError::Protocol(s) => write!(f, "Protocol: {}", s),
            MySqlError::Sql(s) => write!(f, "SQL: {}", s),
        }
    }
}
impl std::error::Error for MySqlError {}
impl From<std::io::Error> for MySqlError {
    fn from(e: std::io::Error) -> Self {
        MySqlError::Io(e)
    }
}
impl From<SqlError> for MySqlError {
    fn from(e: SqlError) -> Self {
        MySqlError::Sql(e.to_string())
    }
}
pub type MySqlResult<T> = Result<T, MySqlError>;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test Packet serialization - roundtrip
    #[test]
    fn test_packet_roundtrip() {
        let pkt = Packet {
            length: 5,
            sequence: 3,
            payload: vec![1, 2, 3, 4, 5],
        };
        let mut buf = Vec::new();
        pkt.write_to(&mut buf).unwrap();

        // Verify header: 3 bytes length + 1 byte sequence
        assert_eq!(buf.len(), 4 + 5);
        assert_eq!(buf[0], 5); // length byte 0
        assert_eq!(buf[1], 0); // length byte 1
        assert_eq!(buf[2], 0); // length byte 2
        assert_eq!(buf[3], 3); // sequence

        let mut reader = std::io::Cursor::new(&buf);
        let read_pkt = Packet::read_from(&mut reader).unwrap();
        assert_eq!(read_pkt.length, pkt.length);
        assert_eq!(read_pkt.sequence, pkt.sequence);
        assert_eq!(read_pkt.payload, pkt.payload);
    }

    // Test Packet with empty payload
    #[test]
    fn test_packet_empty_payload() {
        let pkt = Packet {
            length: 0,
            sequence: 0,
            payload: vec![],
        };
        let mut buf = Vec::new();
        pkt.write_to(&mut buf).unwrap();
        let mut reader = std::io::Cursor::new(&buf);
        let read_pkt = Packet::read_from(&mut reader).unwrap();
        assert_eq!(read_pkt.length, 0);
        assert!(read_pkt.payload.is_empty());
    }

    // Test length-encoded integer branches
    #[test]
    fn test_write_lenenc_int_1byte() {
        // Branch: v < 251 (1 byte)
        let mut buf = Vec::new();
        write_lenenc_int(&mut buf, 100).unwrap();
        assert_eq!(buf, vec![100]);
    }

    #[test]
    fn test_write_lenenc_int_2bytes() {
        // Branch: 251 <= v < 0x10000 (3 bytes: 0xfc + u16)
        let mut buf = Vec::new();
        write_lenenc_int(&mut buf, 300).unwrap();
        assert_eq!(buf[0], 0xfc);
        assert_eq!(u16::from_le_bytes([buf[1], buf[2]]), 300);
    }

    #[test]
    fn test_write_lenenc_int_3bytes() {
        // Branch: 0x10000 <= v < 0x1000000 (4 bytes: 0xfd + u24)
        let mut buf = Vec::new();
        write_lenenc_int(&mut buf, 0x10000).unwrap();
        assert_eq!(buf[0], 0xfd);
        assert_eq!(u32::from_le_bytes([buf[1], buf[2], buf[3], 0]), 0x10000);
    }

    #[test]
    fn test_write_lenenc_int_8bytes() {
        // Branch: v >= 0x1000000 (9 bytes: 0xfe + u64)
        let mut buf = Vec::new();
        write_lenenc_int(&mut buf, 0x1000000).unwrap();
        assert_eq!(buf[0], 0xfe);
        assert_eq!(u64::from_le_bytes([
            buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8],
        ]), 0x1000000);
    }

    // Test read_lenenc_int branches
    #[test]
    fn test_read_lenenc_int_1byte() {
        // Branch: 0..=0xfa
        let mut reader = std::io::Cursor::new(&[100u8]);
        let val = read_lenenc_int(&mut reader).unwrap();
        assert_eq!(val, 100);
    }

    #[test]
    fn test_read_lenenc_int_2bytes() {
        // Branch: 0xfc
        let mut reader = std::io::Cursor::new(&[0xfc, 0x2c, 0x01]); // 300 in little-endian
        let val = read_lenenc_int(&mut reader).unwrap();
        assert_eq!(val, 300);
    }

    #[test]
    fn test_read_lenenc_int_3bytes() {
        // Branch: 0xfd
        let mut reader = std::io::Cursor::new(&[0xfd, 0x00, 0x00, 0x01]); // 0x10000 in little-endian
        let val = read_lenenc_int(&mut reader).unwrap();
        assert_eq!(val, 0x10000);
    }

    #[test]
    fn test_read_lenenc_int_8bytes() {
        // Branch: 0xfe with u64 value
        let mut reader = std::io::Cursor::new(&[0xfe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00]); // 0x10000000000 in little-endian
        let val = read_lenenc_int(&mut reader).unwrap();
        assert_eq!(val, 0x10000000000);
    }

    #[test]
    fn test_read_lenenc_int_0xfb_error() {
        // Branch: 0xfb = NULL
        let mut reader = std::io::Cursor::new(&[0xfbu8]);
        let result = read_lenenc_int(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_lenenc_int_0xff_error() {
        // Branch: 0xff = invalid
        let mut reader = std::io::Cursor::new(&[0xffu8]);
        let result = read_lenenc_int(&mut reader);
        assert!(result.is_err());
    }

    // Test length-encoded string
    #[test]
    fn test_write_lenenc_string() {
        let mut buf = Vec::new();
        write_lenenc_string(&mut buf, b"hello").unwrap();
        assert_eq!(buf[0], 5); // length prefix
        assert_eq!(&buf[1..], b"hello");
    }

    // Test make_ok_packet structure
    #[test]
    fn test_make_ok_packet() {
        let pkt = make_ok_packet(1, 5, 10, 0x0002, 0);
        assert_eq!(pkt.sequence, 1);
        assert_eq!(pkt.payload[0], 0x00); // OK packet type
        // Verify it can be written without error
        let mut buf = Vec::new();
        pkt.write_to(&mut buf).unwrap();
    }

    // Test make_err_packet structure
    #[test]
    fn test_make_err_packet() {
        let pkt = make_err_packet(1, 1146, "42S02", "Table not found");
        assert_eq!(pkt.sequence, 1);
        assert_eq!(pkt.payload[0], 0xff); // ERR packet type
        assert_eq!(u16::from_le_bytes([pkt.payload[1], pkt.payload[2]]), 1146);
        // Verify it can be written without error
        let mut buf = Vec::new();
        pkt.write_to(&mut buf).unwrap();
    }

    // Test make_eof_packet structure
    #[test]
    fn test_make_eof_packet() {
        let pkt = make_eof_packet(2, 0x0002);
        assert_eq!(pkt.sequence, 2);
        assert_eq!(pkt.payload[0], 0xfe); // EOF packet type
        // Verify it can be written without error
        let mut buf = Vec::new();
        pkt.write_to(&mut buf).unwrap();
    }

    // Test make_handshake_packet structure
    #[test]
    fn test_make_handshake_packet() {
        let scramble = [1u8; 20];
        let pkt = make_handshake_packet(0, &scramble);
        assert_eq!(pkt.sequence, 0);
        assert_eq!(pkt.payload[0], 0x0a); // Protocol version 10
        // Verify it can be written without error
        let mut buf = Vec::new();
        pkt.write_to(&mut buf).unwrap();
    }

    // Test is_select function branches
    #[test]
    fn test_is_select() {
        assert!(is_select("SELECT * FROM t"));
        assert!(is_select("SELECT"));
        assert!(is_select("SELECT a FROM t WHERE id = 1"));
        assert!(is_select("  SELECT * FROM t"));
        assert!(is_select("SHOW TABLES"));
        assert!(is_select("SHOW"));
        assert!(is_select("DESCRIBE t"));
        assert!(is_select("EXPLAIN SELECT * FROM t"));
        assert!(!is_select("INSERT INTO t VALUES(1)"));
        assert!(!is_select("UPDATE t SET a = 1"));
        assert!(!is_select("DELETE FROM t"));
        assert!(!is_select("DROP TABLE t"));
    }

    // Test is_transaction_cmd function branches
    #[test]
    fn test_is_transaction_cmd() {
        assert!(is_transaction_cmd("BEGIN"));
        assert!(is_transaction_cmd("COMMIT"));
        assert!(is_transaction_cmd("ROLLBACK"));
        assert!(is_transaction_cmd("START TRANSACTION"));
        assert!(is_transaction_cmd("begin"));
        assert!(is_transaction_cmd("BEGIN "));
        assert!(is_transaction_cmd("  BEGIN"));
        assert!(!is_transaction_cmd("BEGINWORK"));
        assert!(!is_transaction_cmd("BEGIN TRANSACTION")); // different from START TRANSACTION
        assert!(!is_transaction_cmd("SELECT * FROM t"));
        assert!(!is_transaction_cmd(""));
    }

    // Test parse_handshake_response - too short
    #[test]
    fn test_parse_handshake_response_too_short() {
        let pkt = Packet {
            length: 10,
            sequence: 0,
            payload: vec![0; 10],
        };
        let result = parse_handshake_response(&pkt);
        assert!(result.is_err());
    }

    // Test parse_handshake_response - minimal valid response
    #[test]
    fn test_parse_handshake_response_minimal() {
        let mut payload = vec![0u8; 64];
        // capability_flags (4 bytes) at offset 0-3
        payload[0] = 0x01; // LONG_PASSWORD
        payload[1] = 0x00;
        payload[2] = 0x00;
        payload[3] = 0x00;
        // username (null-terminated) starting at offset 32
        payload[32] = b't';
        payload[33] = b'e';
        payload[34] = b's';
        payload[35] = b't';
        payload[36] = 0x00; // null terminator

        let pkt = Packet {
            length: payload.len() as u32,
            sequence: 1,
            payload,
        };
        let result = parse_handshake_response(&pkt);
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.username, "test");
        assert_eq!(resp.capability_flags, 0x01);
    }

    // Test MySqlError Display
    #[test]
    fn test_mysql_error_display() {
        let err = MySqlError::Protocol("test error".to_string());
        assert_eq!(format!("{}", err), "Protocol: test error");

        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io error");
        let err = MySqlError::Io(io_err);
        assert_eq!(format!("{}", err), "IO: io error");

        let err = MySqlError::Sql("sql error".to_string());
        assert_eq!(format!("{}", err), "SQL: sql error");
    }
}

#[derive(Debug)]
pub struct Packet {
    pub length: u32,
    pub sequence: u8,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn read_from<R: Read>(r: &mut R) -> MySqlResult<Self> {
        let length = r.read_u24::<LittleEndian>()?;
        let sequence = r.read_u8()?;
        let mut payload = vec![0u8; length as usize];
        r.read_exact(&mut payload)?;
        Ok(Self {
            length,
            sequence,
            payload,
        })
    }
    pub fn write_to<W: Write>(&self, w: &mut W) -> MySqlResult<()> {
        w.write_u24::<LittleEndian>(self.length)?;
        w.write_u8(self.sequence)?;
        w.write_all(&self.payload)?;
        w.flush()?;
        Ok(())
    }
}

fn write_lenenc_int<W: Write>(w: &mut W, v: u64) -> MySqlResult<()> {
    if v < 251 {
        w.write_u8(v as u8)?;
    } else if v < 0x10000 {
        w.write_u8(0xfc)?;
        w.write_u16::<LittleEndian>(v as u16)?;
    } else if v < 0x1000000 {
        w.write_u8(0xfd)?;
        w.write_u24::<LittleEndian>(v as u32)?;
    } else {
        w.write_u8(0xfe)?;
        w.write_u64::<LittleEndian>(v)?;
    }
    Ok(())
}

fn read_lenenc_int<R: Read>(r: &mut R) -> MySqlResult<u64> {
    let first = r.read_u8()?;
    match first {
        0..=0xfa => Ok(first as u64),
        0xfb => Err(MySqlError::Protocol("NULL lenenc".into())),
        0xfc => Ok(r.read_u16::<LittleEndian>()? as u64),
        0xfd => Ok(r.read_u24::<LittleEndian>()? as u64),
        0xfe => Ok(r.read_u64::<LittleEndian>()?),
        0xff => Err(MySqlError::Protocol("invalid lenenc".into())),
    }
}

fn write_lenenc_string<W: Write>(w: &mut W, s: &[u8]) -> MySqlResult<()> {
    write_lenenc_int(w, s.len() as u64)?;
    w.write_all(s)?;
    Ok(())
}

fn make_handshake_packet(seq: u8, scramble: &[u8; SCRAMBLE_LENGTH]) -> Packet {
    let mut p = Vec::new();
    p.push(0x0a); // protocol 10
    p.extend_from_slice(SERVER_VERSION.as_bytes());
    p.push(0x00);
    p.write_u32::<LittleEndian>(1).unwrap(); // connection_id
    p.extend_from_slice(&scramble[0..8]);
    p.push(0x00); // scramble part1 + filler
    p.write_u16::<LittleEndian>((capability::SERVER_DEFAULT & 0xFFFF) as u16)
        .unwrap();
    p.push(0xff); // charset utf8mb4
    p.write_u16::<LittleEndian>(0x0002).unwrap(); // status AUTOCOMMIT
    p.write_u16::<LittleEndian>(((capability::SERVER_DEFAULT >> 16) & 0xFFFF) as u16)
        .unwrap();
    p.push(SCRAMBLE_LENGTH as u8); // auth_plugin_data_len
    p.extend_from_slice(&[0u8; 10]); // reserved
    p.extend_from_slice(&scramble[8..20]);
    p.push(0x00); // scramble part2 + null
    Packet {
        length: p.len() as u32,
        sequence: seq,
        payload: p,
    }
}

fn make_ok_packet(seq: u8, affected: u64, last_id: u64, status: u16, warnings: u16) -> Packet {
    let mut p = Vec::new();
    p.push(0x00);
    write_lenenc_int(&mut p, affected).unwrap();
    write_lenenc_int(&mut p, last_id).unwrap();
    p.write_u16::<LittleEndian>(status).unwrap();
    p.write_u16::<LittleEndian>(warnings).unwrap();
    Packet {
        length: p.len() as u32,
        sequence: seq,
        payload: p,
    }
}

fn make_err_packet(seq: u8, code: u16, state: &str, msg: &str) -> Packet {
    let mut p = Vec::new();
    p.push(0xff);
    p.write_u16::<LittleEndian>(code).unwrap();
    p.push(0x23);
    p.extend_from_slice(state.as_bytes());
    p.extend_from_slice(msg.as_bytes());
    Packet {
        length: p.len() as u32,
        sequence: seq,
        payload: p,
    }
}

fn make_eof_packet(seq: u8, status: u16) -> Packet {
    let mut p = Vec::new();
    p.push(0xfe);
    p.write_u16::<LittleEndian>(0).unwrap();
    p.write_u16::<LittleEndian>(status).unwrap();
    Packet {
        length: p.len() as u32,
        sequence: seq,
        payload: p,
    }
}

struct HandshakeResponse {
    capability_flags: u32,
    username: String,
    auth_response: Vec<u8>,
    database: Option<String>,
    auth_plugin_name: Option<String>,
}

fn parse_handshake_response(packet: &Packet) -> MySqlResult<HandshakeResponse> {
    let p = &packet.payload;
    if p.len() < 32 {
        return Err(MySqlError::Protocol(format!("Too short: {}", p.len())));
    }
    let cap =
        u16::from_le_bytes([p[0], p[1]]) as u32 | ((u16::from_le_bytes([p[2], p[3]]) as u32) << 16);
    let rest = &p[32..];
    tracing::info!(
        "Handshake response: cap=0x{:08x}, rest_len={}, rest_hex={:02x?}",
        cap,
        rest.len(),
        &rest[..std::cmp::min(32, rest.len())]
    );
    let uname_end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
    let username = String::from_utf8_lossy(&rest[..uname_end]).to_string();
    let mut pos = uname_end + 1;
    let auth = if cap & capability::PLUGIN_AUTH_LENENC_CLIENT_DATA != 0 {
        if pos >= rest.len() {
            vec![]
        } else {
            let mut cur = std::io::Cursor::new(&rest[pos..]);
            match read_lenenc_int(&mut cur) {
                Ok(len) => {
                    let consumed = cur.position() as usize;
                    pos += consumed;
                    if pos + len as usize > rest.len() {
                        rest[pos..].to_vec()
                    } else {
                        let d = rest[pos..pos + len as usize].to_vec();
                        pos += len as usize;
                        d
                    }
                }
                Err(_) => {
                    vec![]
                }
            }
        }
    } else if cap & capability::SECURE_CONNECTION != 0 {
        if pos >= rest.len() {
            vec![]
        } else {
            let len = rest[pos] as usize;
            pos += 1;
            if pos + len > rest.len() {
                rest[pos..].to_vec()
            } else {
                rest[pos..pos + len].to_vec()
            }
        }
    } else {
        let end = rest[pos..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(rest.len() - pos);
        let d = rest[pos..pos + end].to_vec();
        pos += end + 1;
        d
    };
    let db = if cap & capability::CONNECT_WITH_DB != 0 && pos < rest.len() {
        let end = rest[pos..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(rest.len() - pos);
        let d = String::from_utf8_lossy(&rest[pos..pos + end]).to_string();
        pos += end + 1;
        Some(d)
    } else {
        None
    };
    let plugin = if cap & capability::PLUGIN_AUTH != 0 && pos < rest.len() {
        let end = rest[pos..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(rest.len() - pos);
        Some(String::from_utf8_lossy(&rest[pos..pos + end]).to_string())
    } else {
        None
    };
    Ok(HandshakeResponse {
        capability_flags: cap,
        username,
        auth_response: auth,
        database: db,
        auth_plugin_name: plugin,
    })
}

#[allow(dead_code)]
mod col_type {
    pub const TINY: u8 = 0x01;
    pub const SHORT: u8 = 0x02;
    pub const LONG: u8 = 0x03;
    pub const FLOAT: u8 = 0x04;
    pub const DOUBLE: u8 = 0x05;
    pub const LONGLONG: u8 = 0x08;
    pub const INT24: u8 = 0x09;
    pub const VARCHAR: u8 = 0x0f;
    pub const NEWDECIMAL: u8 = 0xf6;
    pub const VARSTRING: u8 = 0xfd;
    pub const STRING: u8 = 0xfe;
    pub const BLOB: u8 = 0xfc;
}

fn col_type_from_string(t: &str) -> u8 {
    let u = t.to_uppercase();
    if u.contains("BIGINT") {
        col_type::LONGLONG
    } else if u.contains("MEDIUMINT") {
        col_type::INT24
    } else if u.contains("SMALLINT") {
        col_type::SHORT
    } else if u.contains("TINYINT") {
        col_type::TINY
    } else if u.contains("INT") {
        col_type::LONG
    } else if u.contains("FLOAT") {
        col_type::FLOAT
    } else if u.contains("DOUBLE") {
        col_type::DOUBLE
    } else if u.contains("DECIMAL") {
        col_type::NEWDECIMAL
    } else if u.contains("BLOB") || u.contains("BINARY") {
        col_type::BLOB
    } else {
        col_type::VARSTRING
    }
}

fn col_len_from_type(t: &str) -> u32 {
    let u = t.to_uppercase();
    if u.contains("INT(1)") {
        1
    } else if u.contains("INT(") {
        11
    } else if u.contains("FLOAT") {
        12
    } else if u.contains("DOUBLE") {
        22
    } else if u.contains("VARCHAR(") {
        255
    } else if u.contains("TEXT") {
        65535
    } else {
        255
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => "NULL".into(),
        Value::Boolean(b) => if *b { "1" } else { "0" }.into(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => format!("{}", f),
        Value::Text(s) => s.clone(),
        Value::Blob(b) => format!("{:?}", b),
    }
}

fn write_text_row<W: Write>(w: &mut W, row: &[Value]) -> MySqlResult<()> {
    for v in row {
        match v {
            Value::Null => {
                w.write_u8(0xfb)?;
            }
            _ => {
                write_lenenc_string(w, value_to_string(v).as_bytes())?;
            }
        }
    }
    Ok(())
}

fn write_column_def<W: Write>(w: &mut W, name: &str, sql_type: &str, seq: u8) -> MySqlResult<()> {
    let mut p = Vec::new();
    write_lenenc_string(&mut p, b"def").unwrap();
    write_lenenc_string(&mut p, b"").unwrap();
    write_lenenc_string(&mut p, b"").unwrap();
    write_lenenc_string(&mut p, b"").unwrap();
    write_lenenc_string(&mut p, name.as_bytes()).unwrap();
    write_lenenc_string(&mut p, name.as_bytes()).unwrap();
    p.push(0x0c);
    p.write_u16::<LittleEndian>(0x21).unwrap();
    p.write_u32::<LittleEndian>(col_len_from_type(sql_type))
        .unwrap();
    p.push(col_type_from_string(sql_type));
    p.write_u16::<LittleEndian>(0x01).unwrap();
    p.push(0x00);
    p.write_u16::<LittleEndian>(0).unwrap();
    Packet {
        length: p.len() as u32,
        sequence: seq,
        payload: p,
    }
    .write_to(w)?;
    Ok(())
}

fn send_result_set<W: Write>(
    w: &mut W,
    cols: &[String],
    ctypes: &[String],
    rows: &[Vec<Value>],
    mut seq: u8,
    cap: u32,
) -> MySqlResult<()> {
    tracing::info!(
        "send_result_set: {} cols, {} rows, start_seq={}",
        cols.len(),
        rows.len(),
        seq
    );
    {
        let mut p = Vec::new();
        write_lenenc_int(&mut p, cols.len() as u64).unwrap();
        Packet {
            length: p.len() as u32,
            sequence: seq,
            payload: p,
        }
        .write_to(w)?;
        seq = seq.wrapping_add(1);
    }
    for (i, n) in cols.iter().enumerate() {
        write_column_def(
            w,
            n,
            ctypes.get(i).map(|s| s.as_str()).unwrap_or("VARCHAR(255)"),
            seq,
        )?;
        seq = seq.wrapping_add(1);
    }
    if cap & capability::DEPRECATE_EOF == 0 {
        make_eof_packet(seq, 0x0002).write_to(w)?;
        seq = seq.wrapping_add(1);
    }
    for (ri, r) in rows.iter().enumerate() {
        let mut p = Vec::new();
        write_text_row(&mut p, r)?;
        tracing::debug!("Row {}: {} bytes, seq={}", ri, p.len(), seq);
        Packet {
            length: p.len() as u32,
            sequence: seq,
            payload: p,
        }
        .write_to(w)?;
        seq = seq.wrapping_add(1);
    }
    if cap & capability::DEPRECATE_EOF == 0 {
        make_eof_packet(seq, 0x0002).write_to(w)?;
    } else {
        make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(w)?;
    }
    tracing::info!("send_result_set done: final_seq={}", seq);
    Ok(())
}

#[allow(clippy::type_complexity)]
fn execute_select(
    sql: &str,
    engine: &mut MemoryExecutionEngine,
) -> MySqlResult<(Vec<String>, Vec<String>, Vec<Vec<Value>>)> {
    let r = engine.execute(sql).map_err(MySqlError::from)?;
    let n = r.rows.first().map(|row| row.len()).unwrap_or(0);
    let cols: Vec<String> = if n > 0 {
        (0..n).map(|i| format!("col_{}", i + 1)).collect()
    } else {
        vec!["result".to_string()]
    };
    let ctypes: Vec<String> = cols.iter().map(|_| "VARCHAR(255)".into()).collect();
    Ok((cols, ctypes, r.rows))
}

fn execute_write(sql: &str, engine: &mut MemoryExecutionEngine) -> MySqlResult<usize> {
    Ok(engine.execute(sql).map_err(MySqlError::from)?.affected_rows)
}

fn is_select(sql: &str) -> bool {
    let u = sql.trim().to_uppercase();
    u.starts_with("SELECT")
        || u.starts_with("SHOW")
        || u.starts_with("DESCRIBE")
        || u.starts_with("EXPLAIN")
}

fn is_transaction_cmd(sql: &str) -> bool {
    let u = sql.trim().to_uppercase();
    u == "BEGIN" || u == "COMMIT" || u == "ROLLBACK" || u == "START TRANSACTION"
}

fn generate_self_signed_cert() -> (Vec<u8>, Vec<u8>) {
    let key_pair = KeyPair::generate().unwrap();
    let key_der = key_pair.serialize_der();
    let params = CertificateParams::new(vec!["localhost".into(), "127.0.0.1".into()]).unwrap();
    let cert = params.self_signed(&key_pair).unwrap();
    let cert_der = cert.der().as_ref().to_vec();
    (cert_der, key_der)
}

fn make_tls_config() -> rustls::ServerConfig {
    let (cert_der, key_der) = generate_self_signed_cert();
    let cert = rustls::pki_types::CertificateDer::from(cert_der);
    let key = rustls::pki_types::PrivateKeyDer::try_from(key_der).unwrap();
    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap()
}

fn do_command_loop<S: Read + Write>(
    stream: &mut S,
    addr: SocketAddr,
    storage: Arc<RwLock<MemoryStorage>>,
    cap: u32,
    _seq: u8,
) -> MySqlResult<()> {
    #[allow(unused_assignments)]
    let mut seq: u8 = 0;
    loop {
        let pkt = match Packet::read_from(stream) {
            Ok(p) => p,
            Err(e) => {
                tracing::debug!("Disconnected: {}", e);
                break;
            }
        };
        let cmd = pkt.payload.first().copied().unwrap_or(0);
        let payload = &pkt.payload[1..];
        seq = pkt.sequence.wrapping_add(1);
        match cmd {
            packet_type::COM_QUIT => break,
            packet_type::COM_PING => {
                make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
            }
            packet_type::COM_INIT_DB => {
                make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
            }
            packet_type::COM_QUERY => {
                let q = String::from_utf8_lossy(payload)
                    .trim_end_matches('\0')
                    .trim()
                    .to_string();
                tracing::info!("Query [{}]: {}", addr, q);
                if q.is_empty() {
                    make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
                    continue;
                }
                if is_transaction_cmd(&q) {
                    make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
                    continue;
                }
                tracing::debug!("Query about to execute: {}", q);
                let mut eng = MemoryExecutionEngine::new(storage.clone());
                if is_select(&q) {
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        execute_select(&q, &mut eng)
                    })) {
                        Ok(Ok((c, t, r))) => send_result_set(stream, &c, &t, &r, seq, cap)?,
                        Ok(Err(e)) => {
                            let code = match &e {
                                MySqlError::Sql(s) if s.contains("not found") => 1146,
                                MySqlError::Sql(_) => 1064,
                                _ => 2000,
                            };
                            make_err_packet(seq, code, "42000", &e.to_string()).write_to(stream)?;
                        }
                        Err(_) => make_err_packet(seq, 2000, "HY000", "Internal error")
                            .write_to(stream)?,
                    }
                } else {
                    match execute_write(&q, &mut eng) {
                        Ok(a) => make_ok_packet(seq, a as u64, 0, 0x0002, 0).write_to(stream)?,
                        Err(e) => {
                            make_err_packet(seq, 1064, "42000", &e.to_string()).write_to(stream)?
                        }
                    }
                }
            }
            packet_type::COM_STMT_PREPARE => {
                make_err_packet(seq, 1295, "HY000", "Prepared statements not supported")
                    .write_to(stream)?;
            }
            _ => {
                make_err_packet(seq, 1047, "HY000", "Unknown command").write_to(stream)?;
            }
        }
    }
    Ok(())
}

fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    storage: Arc<RwLock<MemoryStorage>>,
    tls_config: Arc<rustls::ServerConfig>,
) {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(600)))
        .ok();
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(60)))
        .ok();
    stream.set_nodelay(true).ok();
    tracing::info!("Connection from {}", addr);

    let scramble1: [u8; 8] = rand::random();
    let scramble2: [u8; 12] = rand::random();
    let mut scramble = [0u8; 20];
    scramble[..8].copy_from_slice(&scramble1);
    scramble[8..].copy_from_slice(&scramble2);

    if let Err(e) = make_handshake_packet(0, &scramble).write_to(&mut &stream) {
        tracing::error!("Handshake send: {}", e);
        return;
    }

    let pkt = match Packet::read_from(&mut &stream) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Handshake read: {}", e);
            return;
        }
    };

    // SSL Request?
    if pkt.length == 32 {
        let cap = u32::from_le_bytes([
            pkt.payload[0],
            pkt.payload[1],
            pkt.payload[2],
            pkt.payload[3],
        ]);
        if cap & capability::SSL != 0 {
            tracing::info!("SSL upgrade for {}", addr);
            let mut conn = match rustls::ServerConnection::new(tls_config) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("TLS: {}", e);
                    return;
                }
            };
            // Complete TLS handshake
            conn.complete_io(&mut stream).unwrap();
            let mut tls = rustls::Stream::new(&mut conn, &mut stream);
            // Read handshake response over TLS
            let tls_pkt = match Packet::read_from(&mut tls) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("TLS read: {}", e);
                    return;
                }
            };
            tracing::info!(
                "TLS packet: len={}, seq={}, first_bytes={:02x?}",
                tls_pkt.length,
                tls_pkt.sequence,
                &tls_pkt.payload[..std::cmp::min(16, tls_pkt.payload.len())]
            );
            let resp = match parse_handshake_response(&tls_pkt) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("TLS parse: {}", e);
                    return;
                }
            };
            tracing::info!(
                "TLS user={}, db={:?}, plugin={:?}, auth_resp_len={}",
                resp.username,
                resp.database,
                resp.auth_plugin_name,
                resp.auth_response.len()
            );
            tracing::info!("Auth check: is_empty={}, len=20? {}", resp.auth_response.is_empty(), resp.auth_response.len() == 20);
            if SKIP_AUTH {
                tracing::info!("SKIP_AUTH enabled, accepting connection");
            } else if !(resp.auth_response.is_empty() || resp.auth_response.len() == 20) {
                tracing::warn!("Auth response rejected: len={}", resp.auth_response.len());
                make_err_packet(3, 1045, "28000", "Access denied")
                    .write_to(&mut tls)
                    .ok();
                return;
            }
            tracing::info!("Auth accepted, sending OK packet, seq=3");
            make_ok_packet(3, 0, 0, 0x0002, 0).write_to(&mut tls).ok();
            tracing::info!("Starting command loop, seq=4");
            let _ = do_command_loop(&mut tls, addr, storage, resp.capability_flags, 4);
            return;
        }
    }

    // Non-SSL
    let resp = match parse_handshake_response(&pkt) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Parse: {}", e);
            return;
        }
    };
    tracing::info!(
        "user={}, db={:?}, plugin={:?}, cap=0x{:08x}, auth_resp_len={}",
        resp.username,
        resp.database,
        resp.auth_plugin_name,
        resp.capability_flags,
        resp.auth_response.len()
    );
    if SKIP_AUTH {
        tracing::info!("SKIP_AUTH enabled, accepting connection");
    } else if !(resp.auth_response.is_empty() || resp.auth_response.len() == 20) {
        tracing::warn!("Auth response rejected: len={}", resp.auth_response.len());
        make_err_packet(2, 1045, "28000", "Access denied")
            .write_to(&mut &stream)
            .ok();
        return;
    }
    tracing::info!("Auth accepted, sending OK packet, seq=2");
    make_ok_packet(2, 0, 0, 0x0002, 0)
        .write_to(&mut &stream)
        .ok();
    tracing::info!("Starting command loop, seq=3");
    let _ = do_command_loop(&mut &stream, addr, storage, resp.capability_flags, 3);
}

pub fn run_server(host: &str, port: u16) -> MySqlResult<()> {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)?;
    tracing::info!("MySQL server listening on {}", addr);
    let tls_config = Arc::new(make_tls_config());
    tracing::info!("TLS ready (self-signed cert)");
    
    // Create a SINGLE shared storage for ALL connections
    let storage: Arc<RwLock<MemoryStorage>> = Arc::new(RwLock::new(MemoryStorage::new()));
    {
        let mut eng = MemoryExecutionEngine::new(storage.clone());
        for sql in ["CREATE TABLE content (hash TEXT PRIMARY KEY, doc TEXT NOT NULL, created_at TEXT NOT NULL)",
            "CREATE TABLE vectors (hash_seq TEXT PRIMARY KEY, hash TEXT NOT NULL, embedding TEXT NOT NULL, created_at TEXT NOT NULL)",
            "CREATE TABLE documents (id TEXT PRIMARY KEY, title TEXT, content TEXT, created_at TEXT)"] {
            if let Err(e) = eng.execute(sql) { tracing::warn!("Init: {}", e); }
        }
    }
    for s in listener.incoming() {
        match s {
            Ok(stream) => {
                let addr = stream
                    .peer_addr()
                    .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
                let st = storage.clone();
                let tc = tls_config.clone();
                thread::spawn(move || handle_connection(stream, addr, st, tc));
            }
            Err(e) => tracing::error!("Accept: {}", e),
        }
    }
    Ok(())
}
