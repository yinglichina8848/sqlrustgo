//! SQLRustGo MySQL Wire Protocol Server
//!
//! Implements MySQL Wire Protocol (Server-side) to accept connections
//! from standard MySQL clients (mysql CLI, connectors, etc.)

#![allow(
    clippy::unnecessary_cast,
    clippy::identity_op,
    clippy::needless_bool,
    clippy::len_zero,
    clippy::comparison_to_empty,
    clippy::needless_range_loop,
    clippy::explicit_counter_loop,
    clippy::zero_prefixed_literal,
    dead_code
)]

pub mod http_server;
pub mod monitoring;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use monitoring::{create_monitor, SharedMonitor};
use sqlrustgo::{parse, Value};
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_parser::Statement;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::{SqlError, SqlResult};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;
use rcgen::{DistinguishedName, DnType, KeyPair, PKCS_ECDSA_P256_SHA256};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;

// ============================================================================
// OLD_PASSWORD Hash Algorithm (MySQL 4.x/5.x compatible)
// ============================================================================

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

fn rand_u8_20() -> [u8; 20] {
    let part1 = rand_u8_8();
    let part2 = rand_u8_8();
    let part3: [u8; 4] = [
        rand_u8_8()[0],
        rand_u8_8()[1],
        rand_u8_8()[2],
        rand_u8_8()[3],
    ];
    let mut combined = [0u8; 20];
    combined[0..8].copy_from_slice(&part1);
    combined[8..16].copy_from_slice(&part2);
    combined[16..20].copy_from_slice(&part3);
    combined
}

fn generate_tls_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, "localhost");
    distinguished_name.push(DnType::OrganizationName, "SQLRustGo");

    let key_pair = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256)?;
    let cert_params = rcgen::CertificateParams::default();
    let certificate = cert_params.self_signed(&key_pair)?;

    let cert_pem = certificate.pem();
    let key_pem = key_pair.serialize_pem();

    let cert = CertificateDer::from_pem_slice(cert_pem.as_bytes())?;
    let key = PrivateKeyDer::from_pem_slice(key_pem.as_bytes())?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;

    Ok(config)
}

struct TlsStream<'a> {
    conn: &'a mut rustls::ServerConnection,
    stream: &'a mut TcpStream,
}

impl<'a> TlsStream<'a> {
    fn new(conn: &'a mut rustls::ServerConnection, stream: &'a mut TcpStream) -> Self {
        Self { conn, stream }
    }
}

impl<'a> Read for TlsStream<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            match self.conn.reader().read(buf) {
                Ok(0) => {
                    if self.conn.is_handshaking() {
                        match self.conn.complete_io(self.stream) {
                            Ok((_, _)) => continue,
                            Err(e) => return Err(e),
                        }
                    }
                    return Ok(0);
                }
                Ok(n) => return Ok(n),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    match self.conn.complete_io(self.stream) {
                        Ok((_, _)) => continue,
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl<'a> Write for TlsStream<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut writer = self.conn.writer();
        writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        while self.conn.wants_write() {
            self.conn.complete_io(self.stream)?;
        }
        Ok(())
    }
}

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

    let mut hash = [0u8; 8];
    hash[0..4].copy_from_slice(&result0.to_le_bytes());
    hash[4..8].copy_from_slice(&result1.to_le_bytes());
    hash
}

fn verify_old_password_response(seed: &[u8], response: &[u8], password: &str) -> bool {
    let password_hash = old_password_hash(password);

    let mut nr: u32 = 1345345333u32;
    let mut add: u32 = 7;
    let mut nr2: u32 = 0x12345671u32;

    for i in 0..4 {
        let tmp = u32::from(password_hash[i]);
        nr ^= (((nr & 63) + add) * tmp) + (nr << 8);
        nr2 ^= nr2.wrapping_shl(8) ^ nr;
        add = add.wrapping_add(tmp);
    }
    for i in 4..8 {
        let tmp = u32::from(password_hash[i]);
        nr ^= (((nr & 63) + add) * tmp) + (nr << 8);
        nr2 ^= nr2.wrapping_shl(8) ^ nr;
        add = add.wrapping_add(tmp);
    }

    for &seed_byte in seed {
        let tmp: u32 = seed_byte as u32;
        nr ^= (((nr & 63) + add) * tmp) + (nr << 8);
        nr2 ^= nr2.wrapping_shl(8) ^ nr;
        add = add.wrapping_add(tmp);
    }

    let result0 = nr & (0x7FFFFFFFu32);
    let result1 = nr2 & (0x7FFFFFFFu32);

    let mut expected = [0u8; 8];
    expected[0..4].copy_from_slice(&result0.to_le_bytes());
    expected[4..8].copy_from_slice(&result1.to_le_bytes());

    response == expected
}

// Execution Engine Stub
#[allow(dead_code)]
struct ExecutionEngine {
    storage: Arc<RwLock<MemoryStorage>>,
}

impl ExecutionEngine {
    fn new(storage: Arc<RwLock<MemoryStorage>>) -> Self {
        Self { storage }
    }

    fn execute(&mut self, stmt: Statement) -> SqlResult<ExecutorResult> {
        tracing::info!("ExecutionEngine.execute called for statement: {:?}", stmt);
        Ok(ExecutorResult::new(vec![], 0))
    }
}

// ============================================================================
// Constants
// ============================================================================

const SERVER_VERSION: &str = "5.7.40";
const AUTH_PLUGIN: &str = "mysql_native_password";

mod packet_type {
    pub const COM_QUIT: u8 = 0x01;
    pub const COM_INIT_DB: u8 = 0x02;
    pub const COM_QUERY: u8 = 0x03;
    pub const COM_STMT_PREPARE: u8 = 0x16;
    pub const COM_PING: u8 = 0x0e;
}

mod capability {
    pub const LONG_PASSWORD: u32 = 1 << 0;
    pub const FOUND_ROWS: u32 = 1 << 1;
    pub const LONG_FLAG: u32 = 1 << 2;
    pub const CONNECT_WITH_DB: u32 = 1 << 3;
    pub const PROTOCOL_41: u32 = 1 << 9;
    pub const TRANSACTIONS: u32 = 1 << 13;
    pub const SECURE_CONNECTION: u32 = 1 << 27;
    pub const MULTI_STATEMENTS: u32 = 1 << 16;
    pub const MULTI_RESULTS: u32 = 1 << 17;
    pub const PLUGIN_AUTH: u32 = 1 << 19;
    pub const DEFAULT: u32 = LONG_PASSWORD
        | FOUND_ROWS
        | LONG_FLAG
        | CONNECT_WITH_DB
        | PROTOCOL_41
        | TRANSACTIONS
        | MULTI_STATEMENTS
        | MULTI_RESULTS
        | SECURE_CONNECTION
        | PLUGIN_AUTH;
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
        let len = self.length as u32;
        w.write_all(&[
            ((len >> 0) & 0xff) as u8,
            ((len >> 8) & 0xff) as u8,
            ((len >> 16) & 0xff) as u8,
        ])?;
        w.write_all(&[self.sequence])?;
        w.write_all(&self.payload)?;
        w.flush()?;
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

    p.push(0x0a);
    p.extend_from_slice(SERVER_VERSION.as_bytes());
    p.push(0x00);
    p.write_u32::<LittleEndian>(1).unwrap();
    p.extend_from_slice(&seed[0..8]);
    p.push(0x00);

    let cap_lower = (capability::DEFAULT & 0xFFFF) as u16;
    p.write_u16::<LittleEndian>(cap_lower).unwrap();

    p.push(0x2c);

    p.write_u16::<LittleEndian>(0x0002).unwrap();

    let cap_upper = ((capability::DEFAULT >> 16) & 0xFFFF) as u16;
    p.write_u16::<LittleEndian>(cap_upper).unwrap();

    p.push(21);
    p.extend_from_slice(&[0u8; 10]);

    p.extend_from_slice(&seed[8..20]);
    p.push(0x00);

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
    p.push(0x23);
    p.extend_from_slice(b"HY000");
    p.extend_from_slice(message.as_bytes());
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
) -> MySqlResult<u8> {
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
    Ok(seq.wrapping_add(1))
}

// ============================================================================
// SQL Execution
// ============================================================================

type SelectResult = (Vec<String>, Vec<String>, Vec<Vec<Value>>);

fn execute_select(sql: &str, engine: &mut ExecutionEngine) -> MySqlResult<SelectResult> {
    tracing::info!("execute_select called with: [{}]", sql);
    let statement: Statement = parse(sql).map_err(MySqlError::Sql)?;
    tracing::info!("parse done, executing...");
    let result: ExecutorResult = engine.execute(statement).map_err(MySqlError::from)?;

    // Determine column names and types from result
    let col_count = result.rows.first().map(|r| r.len()).unwrap_or(0);

    // For column names, use empty (client will use default col_N names)
    // Note: SQLRustGo's ExecutionResult doesn't include column metadata from SELECT
    let columns: Vec<String> = (0..col_count).map(|i| format!("col_{}", i + 1)).collect();
    let column_types: Vec<String> = columns.iter().map(|_| "VARCHAR(255)".to_string()).collect();

    Ok((columns, column_types, result.rows))
}

fn execute_write(sql: &str, engine: &mut ExecutionEngine) -> MySqlResult<usize> {
    let statement: Statement = parse(sql).map_err(MySqlError::Sql)?;
    let result: ExecutorResult = engine.execute(statement).map_err(MySqlError::from)?;
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

fn get_query_type(sql: &str) -> String {
    let upper = sql.trim().to_uppercase();
    if upper.starts_with("SELECT") {
        "SELECT".to_string()
    } else if upper.starts_with("INSERT") {
        "INSERT".to_string()
    } else if upper.starts_with("UPDATE") {
        "UPDATE".to_string()
    } else if upper.starts_with("DELETE") {
        "DELETE".to_string()
    } else if upper.starts_with("CREATE") {
        "CREATE".to_string()
    } else if upper.starts_with("DROP") {
        "DROP".to_string()
    } else if upper.starts_with("ALTER") {
        "ALTER".to_string()
    } else if upper.starts_with("SHOW") {
        "SHOW".to_string()
    } else {
        "OTHER".to_string()
    }
}


// ============================================================================
// Sysbench Compatibility SQL Handlers
// ============================================================================

fn extract_like_value(query: &str) -> String {
    let upper = query.to_uppercase();
    if let Some(pos) = upper.find("LIKE") {
        let rest = &query[pos + 4..].trim();
        let rest = rest.trim_start_matches('\'').trim_start_matches('"');
        let rest = rest.trim_end_matches('\'').trim_end_matches('"');
        let rest = rest.trim_end_matches('%').trim();
        return rest.to_string();
    }
    String::new()
}

fn extract_session_var(query: &str) -> String {
    let q = query.trim();
    if let Some(pos) = q.find("@@") {
        let rest = &q[pos + 2..];
        let var = rest.trim().trim_end_matches(';').trim();
        return var.to_string();
    }
    String::new()
}

fn send_variable_result_set<W: Write>(
    w: &mut W,
    var_name: &str,
    var_value: &str,
    mut seq: u8,
) -> MySqlResult<u8> {
    {
        let mut p = Vec::new();
        write_lenenc_int(&mut p, 2).unwrap();
        Packet {
            length: p.len() as u32,
            sequence: seq,
            payload: p,
        }
        .write_to(w)?;
        seq = seq.wrapping_add(1);
    }

    write_column_def(w, "Variable_name", "VARCHAR(255)", seq)?;
    seq = seq.wrapping_add(1);
    write_column_def(w, "Value", "VARCHAR(255)", seq)?;
    seq = seq.wrapping_add(1);

    make_eof_packet(seq, 0x0002).write_to(w)?;
    seq = seq.wrapping_add(1);

    {
        let mut p = Vec::new();
        write_lenenc_string(&mut p, var_name.as_bytes()).unwrap();
        write_lenenc_string(&mut p, var_value.as_bytes()).unwrap();
        Packet {
            length: p.len() as u32,
            sequence: seq,
            payload: p,
        }
        .write_to(w)?;
        seq = seq.wrapping_add(1);
    }

    make_eof_packet(seq, 0x0002).write_to(w)?;
    Ok(seq.wrapping_add(1))
}

fn send_session_var_result_set<W: Write>(
    w: &mut W,
    var_name: &str,
    var_value: &str,
    mut seq: u8,
) -> MySqlResult<u8> {
    {
        let mut p = Vec::new();
        write_lenenc_int(&mut p, 1).unwrap();
        Packet {
            length: p.len() as u32,
            sequence: seq,
            payload: p,
        }
        .write_to(w)?;
        seq = seq.wrapping_add(1);
    }

    write_column_def(w, var_name, "INTEGER", seq)?;
    seq = seq.wrapping_add(1);

    make_eof_packet(seq, 0x0002).write_to(w)?;
    seq = seq.wrapping_add(1);

    {
        let mut p = Vec::new();
        write_lenenc_string(&mut p, var_value.as_bytes()).unwrap();
        Packet {
            length: p.len() as u32,
            sequence: seq,
            payload: p,
        }
        .write_to(w)?;
        seq = seq.wrapping_add(1);
    }

    make_eof_packet(seq, 0x0002).write_to(w)?;
    Ok(seq.wrapping_add(1))
}

fn handle_special_query<W: Write>(query: &str, w: &mut W, seq: u8) -> Option<MySqlResult<u8>> {
    let upper = query.trim().to_uppercase();

    if upper.starts_with("SHOW VARIABLES") {
        let like_val = extract_like_value(query);
        let (var_name, var_value) = match like_val.as_str() {
            "tx_isolation" | "transaction_isolation" => {
                ("tx_isolation", "REPEATABLE-READ")
            }
            "character_set_client" => {
                ("character_set_client", "utf8")
            }
            "character_set_connection" => {
                ("character_set_connection", "utf8")
            }
            "character_set_results" => {
                ("character_set_results", "utf8")
            }
            "character_set_server" => {
                ("character_set_server", "utf8")
            }
            "collation_connection" => {
                ("collation_connection", "utf8_general_ci")
            }
            "collation_server" => {
                ("collation_server", "utf8_general_ci")
            }
            "max_allowed_packet" => {
                ("max_allowed_packet", "4194304")
            }
            "net_buffer_length" => {
                ("net_buffer_length", "16384")
            }
            "version" => {
                ("version", SERVER_VERSION)
            }
            _ => {
                return Some(send_variable_result_set(w, &like_val, "", seq));
            }
        };
        return Some(send_variable_result_set(w, var_name, var_value, seq));
    }

    if upper.starts_with("SELECT @@") {
        let var = extract_session_var(query);
        let value = match var.to_uppercase().as_str() {
            "AUTOCOMMIT" => "1",
            "TX_ISOLATION" | "TRANSACTION_ISOLATION" => "REPEATABLE-READ",
            "CHARACTER_SET_CLIENT" => "utf8",
            "CHARACTER_SET_CONNECTION" => "utf8",
            "CHARACTER_SET_RESULTS" => "utf8",
            "CHARACTER_SET_SERVER" => "utf8",
            "COLLATION_CONNECTION" => "utf8_general_ci",
            "COLLATION_SERVER" => "utf8_general_ci",
            "MAX_ALLOWED_PACKET" => "4194304",
            "VERSION" => SERVER_VERSION,
            _ => "0",
        };
        return Some(send_session_var_result_set(w, &format!("@@{}", var), value, seq));
    }

    None
}

// ============================================================================
// Connection Handler
// ============================================================================

fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    storage: Arc<RwLock<MemoryStorage>>,
    monitor: SharedMonitor,
) -> MySqlResult<()> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(60)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(60)))?;

    if let Err(e) = stream.set_nodelay(true) {
        tracing::warn!("Failed to set TCP_NODELAY: {}", e);
    }

    tracing::info!("MySQL connection from {}", addr);
    monitor.record_connection_opened();

    let seed = rand_u8_20();

    let handshake = make_handshake_packet(0, &seed);
    tracing::info!(
        "[S->C] handshake packet: len={}, seq={}",
        handshake.length,
        handshake.sequence
    );
    handshake.write_to(&mut stream)?;

    let mut initial_header = [0u8; 5];
    let n = match stream.read(&mut initial_header) {
        Ok(0) => {
            tracing::info!("Connection closed by client");
            return Ok(());
        }
        Ok(n) => n,
        Err(e) => {
            tracing::error!("Failed to read from client: {}", e);
            return Err(MySqlError::Io(e));
        }
    };

    if n < 4 {
        tracing::error!("Expected at least 4 bytes, got {}", n);
        return Ok(());
    }

    let is_tls = n >= 2 && initial_header[0] == 0x16 && initial_header[1] == 0x03;

    if is_tls {
        tracing::info!("Detected TLS ClientHello, starting TLS handshake...");
        let tls_config = match generate_tls_config() {
            Ok(cfg) => std::sync::Arc::new(cfg),
            Err(e) => {
                tracing::error!("Failed to generate TLS config: {}", e);
                return Err(MySqlError::Protocol(format!("TLS config error: {}", e)));
            }
        };

        let mut tls_conn = match rustls::ServerConnection::new(tls_config) {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Failed to create TLS connection: {}", e);
                return Err(MySqlError::Protocol(format!("TLS connection error: {}", e)));
            }
        };

        match tls_conn.complete_io(&mut stream) {
            Ok((_, _)) => tracing::info!("TLS handshake completed"),
            Err(e) => {
                tracing::error!("TLS handshake failed: {:?}", e);
                return Err(MySqlError::Protocol(format!("TLS handshake error: {}", e)));
            }
        }

        tracing::info!("TLS connection established");

        let mut tls_stream = TlsStream::new(&mut tls_conn, &mut stream);

        let auth_packet = Packet::read_from(&mut tls_stream)?;
        tracing::info!(
            "[C->S] auth packet: len={}, seq={}",
            auth_packet.length,
            auth_packet.sequence
        );

        let mut seq = auth_packet.sequence.wrapping_add(1);

        make_ok_packet(seq, 0, 0).write_to(&mut tls_stream)?;
        tracing::info!("[S->C] OK packet: seq={}", seq);
        seq = seq.wrapping_add(1);

        let _ = run_command_loop(&mut tls_stream, addr, storage, monitor, seq)?;
        return Ok(());
    }

    // Not TLS - regular MySQL protocol
    let length = u32::from(initial_header[0])
        | (u32::from(initial_header[1]) << 8)
        | (u32::from(initial_header[2]) << 16);
    let recv_seq = initial_header[3];

    tracing::info!(
        "[C->S] auth packet: length={}, seq={}",
        length,
        recv_seq
    );

    if length == 0 {
        tracing::error!("Received empty packet");
        return Ok(());
    }

    let mut auth_payload = vec![0u8; length as usize];
    if n >= 5 {
        auth_payload[0] = initial_header[4];
        stream.read_exact(&mut auth_payload[1..])?;
    } else {
        stream.read_exact(&mut auth_payload)?;
    }

    let mut seq = recv_seq.wrapping_add(1);

    make_ok_packet(seq, 0, 0).write_to(&mut stream)?;
    tracing::info!("[S->C] OK packet: seq={}", seq);
    seq = seq.wrapping_add(1);

    let _ = run_command_loop(&mut stream, addr, storage, monitor.clone(), seq)?;

    tracing::info!("Connection {} closed", addr);
    monitor.record_connection_closed();
    Ok(())
}

fn run_command_loop<W: Read + Write>(
    stream: &mut W,
    addr: SocketAddr,
    storage: Arc<RwLock<MemoryStorage>>,
    monitor: SharedMonitor,
    mut seq: u8,
) -> MySqlResult<u8> {
    loop {
        let packet = match Packet::read_from(stream) {
            Ok(p) => p,
            Err(e) => {
                tracing::info!("Packet read error: {}", e);
                break;
            }
        };

        let cmd = packet.payload.first().copied().unwrap_or(0);
        let payload = &packet.payload[1..];
        tracing::info!(
            "[C->S] cmd=0x{:02x}, payload_len={}, seq={}",
            cmd,
            payload.len(),
            packet.sequence
        );

        match cmd {
            packet_type::COM_QUIT => {
                tracing::info!("Client {} QUIT", addr);
                monitor.record_connection_closed();
                break;
            }

            packet_type::COM_PING => {
                make_ok_packet(seq, 0, 0).write_to(stream)?;
                tracing::info!("[S->C] COM_PING OK: seq={}", seq);
                seq = seq.wrapping_add(1);
            }

            packet_type::COM_INIT_DB => {
                make_ok_packet(seq, 0, 0).write_to(stream)?;
                seq = seq.wrapping_add(1);
            }

            packet_type::COM_QUERY => {
                let query = String::from_utf8_lossy(payload)
                    .trim_end_matches('\0')
                    .trim()
                    .to_string();
                tracing::info!("Query from {}: [{}] (seq={})", addr, query, seq);

                if query.is_empty() {
                    make_ok_packet(seq, 0, 0).write_to(stream)?;
                    seq = seq.wrapping_add(1);
                    continue;
                }

                let upper = query.trim().to_uppercase();

                if upper.starts_with("SET ") {
                    make_ok_packet(seq, 0, 0).write_to(stream)?;
                    tracing::info!("[S->C] SET OK: seq={}", seq);
                    seq = seq.wrapping_add(1);
                    continue;
                }

                if upper == "BEGIN" || upper.starts_with("START TRANSACTION") {
                    make_ok_packet(seq, 0, 0).write_to(stream)?;
                    tracing::info!("[S->C] BEGIN OK: seq={}", seq);
                    seq = seq.wrapping_add(1);
                    continue;
                }

                if upper == "COMMIT" {
                    make_ok_packet(seq, 0, 0).write_to(stream)?;
                    tracing::info!("[S->C] COMMIT OK: seq={}", seq);
                    seq = seq.wrapping_add(1);
                    continue;
                }

                if upper == "ROLLBACK" {
                    make_ok_packet(seq, 0, 0).write_to(stream)?;
                    tracing::info!("[S->C] ROLLBACK OK: seq={}", seq);
                    seq = seq.wrapping_add(1);
                    continue;
                }

                if let Some(result) = handle_special_query(&query, stream, seq) {
                    match result {
                        Ok(new_seq) => {
                            seq = new_seq;
                        }
                        Err(e) => {
                            tracing::warn!("Special query error: {}", e);
                            make_err_packet(seq, 2000, &e.to_string()).write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                    }
                    continue;
                }

                let mut engine = ExecutionEngine::new(storage.clone());
                let query_type = get_query_type(&query);
                let start_time = std::time::Instant::now();

                if is_select_query(&query) {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        execute_select(&query, &mut engine)
                    }));
                    match result {
                        Ok(Ok((columns, column_types, rows))) => {
                            tracing::info!("SELECT returned {} rows", rows.len());
                            seq = send_result_set(stream, &columns, &column_types, &rows, seq)?;
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
                            make_err_packet(seq, code, &e.to_string()).write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                        Err(_) => {
                            tracing::error!("PANIC in execute_select!");
                            make_err_packet(seq, 2000, "Internal server error")
                                .write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                    }
                } else {
                    match execute_write(&query, &mut engine) {
                        Ok(affected) => {
                            make_ok_packet(seq, affected as u64, 0).write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                        Err(e) => {
                            tracing::warn!("Write error: {}", e);
                            make_err_packet(seq, 1064, &e.to_string()).write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                    }
                }

                let execution_time = start_time.elapsed().as_secs_f64();
                monitor.record_query(&query, &query_type, execution_time, 0);
            }

            packet_type::COM_STMT_PREPARE => {
                make_err_packet(seq, 1295, "Prepared statements not yet supported")
                    .write_to(stream)?;
                seq = seq.wrapping_add(1);
            }

            _ => {
                tracing::warn!(
                    "Unknown command 0x{:02x} from {} (payload first bytes: {:?})",
                    cmd,
                    addr,
                    &payload[..std::cmp::min(10, payload.len())]
                );
                make_err_packet(seq, 1047, "Unknown command").write_to(stream)?;
                seq = seq.wrapping_add(1);
            }
        }
    }

    Ok(seq)
}

// ============================================================================
// Server
// ============================================================================

pub fn run_server(host: &str, port: u16, monitoring_port: Option<u16>) -> MySqlResult<()> {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)?;
    tracing::info!("MySQL server listening on {}", addr);

    let storage: Arc<RwLock<MemoryStorage>> = Arc::new(RwLock::new(MemoryStorage::new()));
    let monitor = create_monitor();

    if let Some(mon_port) = monitoring_port {
        let http_server = http_server::MonitoringServer::new(monitor.clone(), mon_port);
        http_server.start();
    }

    {
        let mut engine = ExecutionEngine::new(storage.clone());
        let ddl_stmts = [
            "CREATE TABLE content (hash TEXT PRIMARY KEY, doc TEXT NOT NULL, created_at TEXT NOT NULL)",
            "CREATE TABLE vectors (hash_seq TEXT PRIMARY KEY, hash TEXT NOT NULL, embedding TEXT NOT NULL, created_at TEXT NOT NULL)",
            "CREATE TABLE documents (id TEXT PRIMARY KEY, title TEXT, content TEXT, created_at TEXT)",
        ];

        for sql in ddl_stmts {
            match parse(sql) {
                Ok(stmt) => {
                    if let Err(e) = engine.execute(stmt) {
                        tracing::warn!("Init table error (may already exist): {}", e);
                    } else {
                        tracing::info!(
                            "Created table via: {}",
                            sql.split_whitespace().nth(2).unwrap_or("?")
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("Parse error for '{}': {}", sql, e);
                }
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
                let monitor = monitor.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, addr, storage, monitor) {
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
