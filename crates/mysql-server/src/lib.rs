//! SQLRustGo MySQL Wire Protocol Server
//!
//! Implements MySQL Wire Protocol (Server-side) to accept connections
//! from standard MySQL clients (mysql CLI, connectors, etc.)

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use sqlrustgo::{parse, MemoryExecutionEngine, Value};
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_parser::Statement;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::{SqlError, SqlResult};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;

// ============================================================================
// Random utilities
// ============================================================================

/// Generate 8 random bytes using system time as seed
fn rand_u8_8() -> [u8; 8] {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let seed = (now ^ (now >> 64)) as u64;
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&seed.to_le_bytes());
    for i in 0..8 {
        let val = u32::from(bytes[i]).wrapping_mul(0x9e3779b9).rotate_right(5);
        bytes[i] = val as u8;
    }
    bytes
}

// ============================================================================
// OLD_PASSWORD Hash Algorithm (MySQL 4.x/5.x compatible)
// ============================================================================

/// Compute OLD_PASSWORD hash as used by MySQL 4.x and early 5.x
/// This is a simple hash with no salt and no iteration
fn old_password_hash(password: &str) -> [u8; 8] {
    let mut nr: u32 = 1345345333u32;
    let mut add: u32 = 7;
    let mut nr2: u32 = 0x12345671u32;

    for byte in password.bytes() {
        let tmp: u32 = byte as u32;
        nr ^= (((nr & 63) + add) * tmp) + (nr << 8);
        nr2 ^= nr2.wrapping_shl(8) ^ nr;
        add = add.wrapping_add(tmp);
    }

    let result0 = nr & (0x7FFFFFFFu32);
    let result1 = nr2 & (0x7FFFFFFFu32);

    // Pack two 32-bit values into 8 bytes (little-endian)
    let mut hash = [0u8; 8];
    hash[0..4].copy_from_slice(&result0.to_le_bytes());
    hash[4..8].copy_from_slice(&result1.to_le_bytes());
    hash
}

/// Verify old password challenge-response
/// seed: 8-byte challenge from server
/// response: 8-byte response from client
/// stored_hash: stored password hash (8 bytes)
fn verify_old_password_response(seed: &[u8], response: &[u8], password: &str) -> bool {
    // Client computes: hash2(old_password_hash(password), seed)
    // where hash2 combines the 64-bit hash with the 8-byte seed
    let password_hash = old_password_hash(password);

    // The challenge-response algorithm:
    // For each byte of the seed, apply a mixing function
    let mut nr: u32 = 1345345333u32;
    let mut add: u32 = 7;
    let mut nr2: u32 = 0x12345671u32;

    // First, hash the password into nr and nr2
    for i in 0..4 {
        let tmp: u32 = password_hash[i] as u32;
        nr ^= (((nr & 63) + add) * tmp) + (nr << 8);
        nr2 ^= nr2.wrapping_shl(8) ^ nr;
        add = add.wrapping_add(tmp);
    }
    for i in 4..8 {
        let tmp: u32 = password_hash[i] as u32;
        nr ^= (((nr & 63) + add) * tmp) + (nr << 8);
        nr2 ^= nr2.wrapping_shl(8) ^ nr;
        add = add.wrapping_add(tmp);
    }

    // Then mix in the seed
    for &seed_byte in seed {
        let tmp: u32 = seed_byte as u32;
        nr ^= (((nr & 63) + add) * tmp) + (nr << 8);
        nr2 ^= nr2.wrapping_shl(8) ^ nr;
        add = add.wrapping_add(tmp);
    }

    let result0 = nr & (0x7FFFFFFFu32);
    let result1 = nr2 & (0x7FFFFFFFu32);

    // Build expected response
    let mut expected = [0u8; 8];
    expected[0..4].copy_from_slice(&result0.to_le_bytes());
    expected[4..8].copy_from_slice(&result1.to_le_bytes());

    response == expected
}

// ============================================================================
// Constants
// ============================================================================

const SERVER_VERSION: &str = "SQLRustGo-2.8.0";
const AUTH_PLUGIN: &str = "mysql_native_password";

mod packet_type {
    pub const COM_QUIT: u8 = 0x01;
    pub const COM_INIT_DB: u8 = 0x02;
    pub const COM_QUERY: u8 = 0x03;
    pub const COM_STMT_PREPARE: u8 = 0x16;
    pub const COM_PING: u8 = 0x0e;
}

mod capability {
    // MySQL protocol capability flags (lower 16 bits)
    pub const PROTOCOL_41: u32 = 1 << 9;      // 0x0200
    pub const LONG_PASSWORD: u32 = 1 << 0;     // 0x0001
    pub const FOUND_ROWS: u32 = 1 << 1;       // 0x0002
    pub const LONG_FLAG: u32 = 1 << 2;        // 0x0004
    pub const CONNECT_WITH_DB: u32 = 1 << 3;  // 0x0008
    pub const NO_SCHEMA: u32 = 1 << 4;         // 0x0010
    pub const COMPRESS: u32 = 1 << 5;         // 0x0020
    pub const ODBC: u32 = 1 << 6;             // 0x0040
    pub const LOCAL_FILES: u32 = 1 << 7;      // 0x0080
    pub const IGNORE_SPACE: u32 = 1 << 8;     // 0x0100
    pub const SECURE_CONNECTION: u32 = 1 << 12; // 0x1000
    pub const MULTI_STATEMENTS: u32 = 1 << 16; // 0x00010000
    pub const MULTI_RESULTS: u32 = 1 << 17;   // 0x00020000
    pub const PS_MULTI_RESULTS: u32 = 1 << 18; // 0x00040000
    pub const PLUGIN_AUTH: u32 = 1 << 19;     // 0x00080000
    pub const CONNECT_ATTRS: u32 = 1 << 20;   // 0x00100000
    pub const PLUGIN: u32 = 1 << 21;          // 0x00200000
    pub const TRANSLATE_DUMP: u32 = 1 << 22;  // 0x00400000
    pub const VERIFY_SERVER_CERT: u32 = 1 << 30; // 0x40000000
    pub const RESERVED: u32 = 1 << 31;        // 0x80000000

    // Common capability flags
    pub const DEFAULT: u32 = PROTOCOL_41 | SECURE_CONNECTION | MULTI_STATEMENTS | MULTI_RESULTS | PLUGIN_AUTH | CONNECT_ATTRS | PLUGIN;
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum MySqlError {
    Io(std::io::Error),
    Protocol(String),
    Sql(String),
}

impl std::fmt::Display for MySqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MySqlError::Io(e) => write!(f, "IO error: {}", e),
            MySqlError::Protocol(s) => write!(f, "Protocol error: {}", s),
            MySqlError::Sql(s) => write!(f, "SQL error: {}", s),
        }
    }
}

impl std::error::Error for MySqlError {}

impl From<std::io::Error> for MySqlError {
    fn from(e: std::io::Error) -> Self {
        MySqlError::Io(e)
    }
}
impl From<String> for MySqlError {
    fn from(e: String) -> Self {
        MySqlError::Sql(e)
    }
}
impl From<&str> for MySqlError {
    fn from(e: &str) -> Self {
        MySqlError::Sql(e.to_string())
    }
}
impl From<SqlError> for MySqlError {
    fn from(e: SqlError) -> Self {
        MySqlError::Sql(e.to_string())
    }
}

pub type MySqlResult<T> = Result<T, MySqlError>;

// ============================================================================
// MySQL Packet
// ============================================================================

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
        Ok(())
    }
}

// ============================================================================
// MySQL Types Serialization
// ============================================================================

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

fn write_lenenc_string<W: Write>(w: &mut W, s: &[u8]) -> MySqlResult<()> {
    write_lenenc_int(w, s.len() as u64)?;
    w.write_all(s)?;
    Ok(())
}

// ============================================================================
// MySQL Packets
// ============================================================================

fn make_handshake_packet(seq: u8, seed: &[u8]) -> Packet {
    let mut p = Vec::new();
    // MySQL 8.0 handshake packet format
    p.push(0x0a); // protocol version
    p.extend_from_slice(SERVER_VERSION.as_bytes());
    p.push(0x00); // null terminator for server version

    p.write_u32::<LittleEndian>(1).unwrap(); // connection id

    // auth_plugin_data_part_1 (8 bytes for mysql_native_password)
    p.extend_from_slice(&seed);
    p.push(0x00); // null terminator

    // capability flags - lower 16 bits
    p.write_u16::<LittleEndian>((capability::DEFAULT & 0xFFFF) as u16).unwrap();

    p.push(0x2c); // character set (utf8mb4_general_ci = 0x2c)

    // server status
    p.write_u16::<LittleEndian>(0x0002).unwrap(); // SERVER_STATUS_AUTOCOMMIT

    // capability flags - upper 16 bits
    p.write_u16::<LittleEndian>((capability::DEFAULT >> 16) as u16).unwrap();

    // auth_plugin_data_length (total length of auth plugin data, including null terminator)
    // For mysql_native_password, it's 8 (8 bytes + 1 null terminator = 9 but only first 8 matter)
    p.push(8);

    // reserved (10 bytes)
    p.extend_from_slice(&[0u8; 10]);

    // auth_plugin_name (null-terminated)
    p.extend_from_slice(AUTH_PLUGIN.as_bytes());
    p.push(0x00);
    Packet {
        length: p.len() as u32,
        sequence: seq,
        payload: p,
    }
}

fn make_ok_packet(seq: u8, affected_rows: u64, last_insert_id: u64) -> Packet {
    let mut p = Vec::new();
    p.push(0x00);
    write_lenenc_int(&mut p, affected_rows).unwrap();
    write_lenenc_int(&mut p, last_insert_id).unwrap();
    p.write_u16::<LittleEndian>(0x0002).unwrap();
    p.write_u16::<LittleEndian>(0).unwrap();
    Packet {
        length: p.len() as u32,
        sequence: seq,
        payload: p,
    }
}

fn make_err_packet(seq: u8, code: u16, message: &str) -> Packet {
    let mut p = Vec::new();
    p.push(0xff);
    p.write_u16::<LittleEndian>(code).unwrap();
    p.push(0x00);
    p.extend_from_slice(b"#00000");
    p.extend_from_slice(message.as_bytes());
    p.push(0x00);
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

// ============================================================================
// Result Set
// ============================================================================

#[allow(dead_code)]
mod col_type {
    pub const DECIMAL: u8 = 0x00;
    pub const TINY: u8 = 0x01;
    pub const SHORT: u8 = 0x02;
    pub const LONG: u8 = 0x03;
    pub const FLOAT: u8 = 0x04;
    pub const DOUBLE: u8 = 0x05;
    pub const NULL: u8 = 0x06;
    pub const TIMESTAMP: u8 = 0x07;
    pub const LONGLONG: u8 = 0x08;
    pub const INT24: u8 = 0x09;
    pub const DATE: u8 = 0x0a;
    pub const TIME: u8 = 0x0b;
    pub const DATETIME: u8 = 0x0c;
    pub const YEAR: u8 = 0x0d;
    pub const VARCHAR: u8 = 0x0f;
    pub const BIT: u8 = 0x10;
    pub const JSON: u8 = 0xf5;
    pub const NEWDECIMAL: u8 = 0xf6;
    pub const BLOB: u8 = 0xfc;
    pub const STRING: u8 = 0xfe;
    pub const VARSTRING: u8 = 0xfd;
    pub const GEOMETRY: u8 = 0xff;
}

fn col_type_from_string(sql_type: &str) -> u8 {
    let upper = sql_type.to_uppercase();
    if upper.contains("INT") {
        if upper.contains("BIGINT") {
            col_type::LONGLONG
        } else if upper.contains("MEDIUMINT") || upper.contains("INT24") {
            col_type::INT24
        } else if upper.contains("SMALLINT") {
            col_type::SHORT
        } else if upper.contains("TINYINT") {
            col_type::TINY
        } else {
            col_type::LONG
        }
    } else if upper.contains("FLOAT") {
        col_type::FLOAT
    } else if upper.contains("DOUBLE") {
        col_type::DOUBLE
    } else if upper.contains("DECIMAL") || upper.contains("NUMERIC") {
        col_type::NEWDECIMAL
    } else if upper.contains("CHAR") || upper.contains("TEXT") || upper.contains("VARCHAR") {
        col_type::VARSTRING
    } else if upper.contains("BLOB") || upper.contains("BINARY") {
        col_type::BLOB
    } else if upper.contains("DATE") {
        col_type::DATE
    } else if upper.contains("TIME") {
        col_type::TIME
    } else if upper.contains("DATETIME") || upper.contains("TIMESTAMP") {
        col_type::DATETIME
    } else {
        col_type::STRING
    }
}

fn col_len_from_type(sql_type: &str) -> u32 {
    let upper = sql_type.to_uppercase();
    if upper.contains("INT(1)") {
        1
    } else if upper.contains("INT(") {
        upper
            .split("INT(")
            .nth(1)
            .and_then(|s| s.split(')').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(11)
    } else if upper.contains("FLOAT") {
        12
    } else if upper.contains("DOUBLE") {
        22
    } else if upper.contains("VARCHAR(") {
        upper
            .split("VARCHAR(")
            .nth(1)
            .and_then(|s| s.split(')').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(255)
    } else if upper.contains("TEXT") {
        65535
    } else {
        255
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => format!("{}", f),
        Value::Text(s) => s.clone(),
        Value::Blob(b) => format!("{:?}", b),
    }
}

fn write_text_row<W: Write>(w: &mut W, row: &[Value]) -> MySqlResult<()> {
    for val in row {
        let s = value_to_string(val);
        write_lenenc_string(w, s.as_bytes())?;
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
    p.write_u16::<LittleEndian>(0x21).unwrap(); // charset utf8mb4
    p.write_u32::<LittleEndian>(col_len_from_type(sql_type))
        .unwrap();
    p.push(col_type_from_string(sql_type));
    p.write_u16::<LittleEndian>(0x01).unwrap(); // NOT_NULL_FLAG
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
    columns: &[String],
    column_types: &[String],
    rows: &[Vec<Value>],
    mut seq: u8,
) -> MySqlResult<()> {
    // Column count
    {
        let mut p = Vec::new();
        write_lenenc_int(&mut p, columns.len() as u64).unwrap();
        Packet {
            length: p.len() as u32,
            sequence: seq,
            payload: p,
        }
        .write_to(w)?;
        seq = seq.wrapping_add(1);
    }

    // Column definitions
    for (i, name) in columns.iter().enumerate() {
        let ct = column_types
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("VARCHAR(255)");
        write_column_def(w, name, ct, seq)?;
        seq = seq.wrapping_add(1);
    }

    // EOF
    make_eof_packet(seq, 0x0002).write_to(w)?;
    seq = seq.wrapping_add(1);

    // Rows
    for row in rows {
        let mut p = Vec::new();
        write_text_row(&mut p, row)?;
        Packet {
            length: p.len() as u32,
            sequence: seq,
            payload: p,
        }
        .write_to(w)?;
        seq = seq.wrapping_add(1);
    }

    // EOF
    make_eof_packet(seq, 0x0002).write_to(w)?;
    Ok(())
}

// ============================================================================
// SQL Execution
// ============================================================================

type SelectResult = (Vec<String>, Vec<String>, Vec<Vec<Value>>);

fn execute_select(sql: &str, engine: &mut MemoryExecutionEngine) -> MySqlResult<SelectResult> {
    tracing::info!("execute_select called with: [{}]", sql);
    let result: ExecutorResult = engine.execute(sql).map_err(MySqlError::from)?;

    // Determine column names and types from result
    let col_count = result.rows.first().map(|r| r.len()).unwrap_or(0);

    // For column names, use empty (client will use default col_N names)
    // Note: SQLRustGo's ExecutionResult doesn't include column metadata from SELECT
    let columns: Vec<String> = (0..col_count).map(|i| format!("col_{}", i + 1)).collect();
    let column_types: Vec<String> = columns.iter().map(|_| "VARCHAR(255)".to_string()).collect();

    Ok((columns, column_types, result.rows))
}

fn execute_write(sql: &str, engine: &mut MemoryExecutionEngine) -> MySqlResult<usize> {
    let result: ExecutorResult = engine.execute(sql).map_err(MySqlError::from)?;
    Ok(result.affected_rows)
}

fn is_select_query(sql: &str) -> bool {
    let upper = sql.trim().to_uppercase();
    let result = upper.starts_with("SELECT")
        || upper.starts_with("SHOW")
        || upper.starts_with("DESCRIBE")
        || upper.starts_with("EXPLAIN");
    tracing::info!("is_select_query([{}]) = {}", sql, result);
    result
}

// ============================================================================
// Connection Handler
// ============================================================================

fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    storage: Arc<RwLock<MemoryStorage>>,
) -> MySqlResult<()> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(60)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(60)))?;

    tracing::info!("MySQL connection from {}", addr);

    let mut seq = 0u8;

    // Generate 8-byte random seed for mysql_native_password auth
    let seed: [u8; 8] = rand::random();

    // Send handshake with seed
    let handshake = make_handshake_packet(seq, &seed);
    tracing::info!("Sending handshake: len={}, seq={}, payload_len={}", handshake.length, handshake.sequence, handshake.payload.len());
    let handshake_hex = handshake.payload.iter().take(60).map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
    tracing::info!("Handshake payload hex: {}", handshake_hex);
    handshake.write_to(&mut stream)?;
    tracing::info!("Handshake sent {} bytes total", 4 + handshake.payload.len());
    seq = seq.wrapping_add(1);

    // Try to read handshake response (optional - some clients send it differently)
    // Use a short timeout to avoid blocking
    stream.set_read_timeout(Some(std::time::Duration::from_millis(100)))?;
    let auth_packet = match Packet::read_from(&mut stream) {
        Ok(p) => {
            tracing::info!("Auth packet: len={}, seq={}, payload: {:02x?}", 
                p.length, p.sequence,
                &p.payload[..p.payload.len().min(60)]);
            Some(p)
        }
        Err(e) => {
            tracing::warn!("Auth packet read error (continuing anyway): {}", e);
            None
        }
    };
    // Restore timeout
    stream.set_read_timeout(Some(std::time::Duration::from_secs(60)))?;

    // Simplified auth: accept any connection for development/testing
    // This avoids parsing issues with different client auth protocols
    let auth_ok = true;
    tracing::info!("Auth accepted (development mode: no password verification)");

    // Send OK (auth accepted)
    tracing::info!("Sending OK packet with seq={}", seq);
    let ok_packet = make_ok_packet(seq, 0, 0);
    ok_packet.write_to(&mut stream)?;
    tracing::info!("OK packet sent, flushing...");
    stream.flush().ok();

    if !auth_ok {
        make_err_packet(seq, 1045, "Access denied").write_to(&mut stream)?;
        return Ok(());
    }

    // Send OK (auth accepted)
    make_ok_packet(seq, 0, 0).write_to(&mut stream)?;
    seq = seq.wrapping_add(1);

    loop {
        let packet = match Packet::read_from(&mut stream) {
            Ok(p) => {
                tracing::info!("Received packet: len={}, seq={}", p.length, p.sequence);
                p
            }
            Err(e) => {
                // Check if it's a connection closed error
                if e.to_string().contains("end of file") || e.to_string().contains("Connection reset") || e.to_string().contains("broken pipe") {
                    tracing::info!("Client {} closed connection normally", addr);
                } else {
                    tracing::warn!("Packet read error from {}: {}", addr, e);
                }
                break;
            }
        };

        let cmd = packet.payload.first().copied().unwrap_or(0);
        let payload = &packet.payload[1..];
        tracing::info!(
            "Received packet: cmd=0x{:02x}, payload_len={}",
            cmd,
            payload.len()
        );

        match cmd {
            packet_type::COM_QUIT => {
                tracing::info!("Client {} QUIT", addr);
                break;
            }

            packet_type::COM_PING => {
                tracing::info!(">>> COM_PING received, seq={}", seq);
                let result = make_ok_packet(seq, 0, 0).write_to(&mut stream);
                tracing::info!(">>> COM_PING write result: {:?}", result);
                match result {
                    Ok(()) => {}
                    Err(e) => tracing::warn!("Write error in COM_PING: {}", e),
                }
                seq = seq.wrapping_add(1);
            }

            packet_type::COM_INIT_DB => {
                make_ok_packet(seq, 0, 0).write_to(&mut stream)?;
                seq = seq.wrapping_add(1);
            }

            packet_type::COM_QUERY => {
                let query = String::from_utf8_lossy(payload)
                    .trim_end_matches('\0')
                    .trim()
                    .to_string();
                tracing::info!("Query from {}: [{}] (seq={})", addr, query, seq);

                if query.is_empty() {
                    make_ok_packet(seq, 0, 0).write_to(&mut stream)?;
                    seq = seq.wrapping_add(1);
                    continue;
                }

                let mut engine = MemoryExecutionEngine::new(storage.clone());

                tracing::info!("is_select={}, query=[{}]", is_select_query(&query), query);
                if is_select_query(&query) {
                    tracing::info!("Executing SELECT...");
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        execute_select(&query, &mut engine)
                    }));
                    match result {
                        Ok(Ok((columns, column_types, rows))) => {
                            tracing::info!("SELECT returned {} rows", rows.len());
                            send_result_set(&mut stream, &columns, &column_types, &rows, seq)?;
                        }
                        Ok(Err(e)) => {
                            tracing::warn!("Query error: {}", e);
                            let code: u16 = match &e {
                                MySqlError::Sql(s)
                                    if s.contains("not found") || s.contains("does not exist") =>
                                {
                                    1146
                                }
                                MySqlError::Sql(_) => 1064,
                                _ => 2000,
                            };
                            make_err_packet(seq, code, &e.to_string()).write_to(&mut stream)?;
                        }
                        Err(_) => {
                            tracing::error!("PANIC in execute_select!");
                            make_err_packet(seq, 2000, "Internal server error")
                                .write_to(&mut stream)?;
                        }
                    }
                } else {
                    match execute_write(&query, &mut engine) {
                        Ok(affected) => {
                            make_ok_packet(seq, affected as u64, 0).write_to(&mut stream)?;
                        }
                        Err(e) => {
                            tracing::warn!("Write error: {}", e);
                            make_err_packet(seq, 1064, &e.to_string()).write_to(&mut stream)?;
                        }
                    }
                }
                seq = seq.wrapping_add(1);
            }

            packet_type::COM_STMT_PREPARE => {
                make_err_packet(seq, 1295, "Prepared statements not yet supported")
                    .write_to(&mut stream)?;
                seq = seq.wrapping_add(1);
            }

            _ => {
                tracing::warn!(
                    "Unknown command 0x{:02x} from {} (payload first bytes: {:?})",
                    cmd,
                    addr,
                    &payload[..std::cmp::min(10, payload.len())]
                );
                make_err_packet(seq, 1047, "Unknown command").write_to(&mut stream)?;
                seq = seq.wrapping_add(1);
            }
        }
    }

    tracing::info!("Connection {} closed", addr);
    Ok(())
}

// ============================================================================
// Server
// ============================================================================

pub fn run_server(host: &str, port: u16, monitoring_port: u16) -> MySqlResult<()> {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)?;
    tracing::info!("MySQL server listening on {}", addr);

    let storage: Arc<RwLock<MemoryStorage>> = Arc::new(RwLock::new(MemoryStorage::new()));

    // Initialize QMD tables via SQL
    {
        let mut engine = MemoryExecutionEngine::new(storage.clone());
        let ddl_stmts = [
            "CREATE TABLE content (hash TEXT PRIMARY KEY, doc TEXT NOT NULL, created_at TEXT NOT NULL)",
            "CREATE TABLE vectors (hash_seq TEXT PRIMARY KEY, hash TEXT NOT NULL, embedding TEXT NOT NULL, created_at TEXT NOT NULL)",
            "CREATE TABLE documents (id TEXT PRIMARY KEY, title TEXT, content TEXT, created_at TEXT)",
        ];

        for sql in ddl_stmts {
            tracing::info!("Creating QMD table via: {}", sql);
            if let Err(e) = engine.execute(sql) {
                tracing::warn!("Init table error (may already exist): {}", e);
            }
        }
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let addr = stream
                    .peer_addr()
                    .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
                let storage = storage.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, addr, storage) {
                        tracing::error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                tracing::error!("Accept error: {}", e);
            }
        }
    }

    Ok(())
}
