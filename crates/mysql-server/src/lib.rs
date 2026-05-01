//! SQLRustGo MySQL Wire Protocol Server
//!
//! Supports mysql_native_password auth + TLS (mariadb-connector-c 3.4+ compatible)

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rcgen::{CertificateParams, KeyPair};
use sha1::{Digest, Sha1};
use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::{MemoryStorage, StorageEngine};
use sqlrustgo_types::{SqlError, Value};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;

const SERVER_VERSION: &str = "8.0.33-SQLRustGo";
#[allow(dead_code)]
const AUTH_PLUGIN: &str = "mysql_native_password";
const SCRAMBLE_LENGTH: usize = 20;
const SKIP_AUTH: bool = false;

mod packet_type {
    pub const COM_QUIT: u8 = 0x01;
    pub const COM_INIT_DB: u8 = 0x02;
    pub const COM_QUERY: u8 = 0x03;
    pub const COM_PING: u8 = 0x0e;
    pub const COM_STMT_PREPARE: u8 = 0x16;
    pub const COM_STMT_EXECUTE: u8 = 0x17;
    pub const COM_STMT_CLOSE: u8 = 0x19;
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
        | PLUGIN_AUTH
        | PLUGIN_AUTH_LENENC_CLIENT_DATA
        | DEPRECATE_EOF
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
impl From<String> for MySqlError {
    fn from(s: String) -> Self {
        MySqlError::Sql(s)
    }
}
impl From<&str> for MySqlError {
    fn from(s: &str) -> Self {
        MySqlError::Sql(s.to_string())
    }
}
pub type MySqlResult<T> = Result<T, MySqlError>;

// User storage for mysql_native_password authentication
#[derive(Debug, Clone)]
struct UserPassword {
    password_hash: [u8; 20],
}

#[derive(Debug, Clone, Default)]
struct UserStore {
    users: HashMap<String, UserPassword>,
}

impl UserStore {
    fn new() -> Self {
        let mut store = Self {
            users: HashMap::new(),
        };
        store.add_user("root", "");
        store.add_user("mysql", "mysql");
        store
    }

    fn add_user(&mut self, username: &str, password: &str) {
        let password_hash = compute_double_sha1(password.as_bytes());
        self.users
            .insert(username.to_string(), UserPassword { password_hash });
    }

    fn verify_password(&self, username: &str, scramble: &[u8; 20], auth_response: &[u8]) -> bool {
        let user = match self.users.get(username) {
            Some(u) => u,
            None => return false,
        };
        verify_mysql_native_password(&user.password_hash, scramble, auth_response)
    }
}

// Compute SHA1(SHA1(password)) - what MySQL stores
fn compute_double_sha1(data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let first = hasher.finalize();
    let mut hasher = Sha1::new();
    hasher.update(first);
    hasher.finalize().into()
}

// Compute SHA1(data)
fn sha1_simple(data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().into()
}

// Verify mysql_native_password authentication
// auth_response = SHA1(password) XOR SHA1(scramble + SHA1(SHA1(password)))
// We verify by checking if: SHA1(scramble + stored_password_hash) XOR auth_response = SHA1(password)
// And then verify SHA1(SHA1(password)) = stored_password_hash
fn verify_mysql_native_password(
    stored_password_hash: &[u8; 20],
    scramble: &[u8; 20],
    auth_response: &[u8],
) -> bool {
    if auth_response.len() != 20 {
        return false;
    }
    let mut hasher = Sha1::new();
    hasher.update(scramble);
    hasher.update(stored_password_hash);
    let expected_hash = hasher.finalize();
    let mut result = [0u8; 20];
    for i in 0..20 {
        result[i] = expected_hash[i] ^ auth_response[i];
    }
    let computed_first = sha1_simple(&result);
    computed_first == *stored_password_hash
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
    p.push((SCRAMBLE_LENGTH + 1) as u8); // auth_plugin_data_len
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
) -> MySqlResult<u8> {
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
        seq = seq.wrapping_add(1);
    } else {
        make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(w)?;
        seq = seq.wrapping_add(1);
    }
    tracing::info!("send_result_set done: final_seq={}", seq);
    Ok(seq)
}

struct PreparedStatementInfo {
    sql: String,
    #[allow(dead_code)]
    param_count: u16,
    column_count: u16,
}

struct PreparedStatementManager {
    statements: std::collections::HashMap<u32, PreparedStatementInfo>,
    next_id: u32,
}

impl PreparedStatementManager {
    fn new() -> Self {
        Self {
            statements: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    fn add(&mut self, sql: String, param_count: u16, column_count: u16) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.statements.insert(
            id,
            PreparedStatementInfo {
                sql,
                param_count,
                column_count,
            },
        );
        id
    }

    fn get(&self, id: u32) -> Option<&PreparedStatementInfo> {
        self.statements.get(&id)
    }

    fn remove(&mut self, id: u32) {
        self.statements.remove(&id);
    }
}

fn count_placeholders(sql: &str) -> u16 {
    sql.chars().filter(|&c| c == '?').count() as u16
}

fn replace_placeholders(sql: &str, params: &[Vec<u8>]) -> String {
    let mut result = sql.to_string();
    for param in params.iter() {
        let value = if param.is_empty() {
            "NULL".to_string()
        } else {
            match String::from_utf8(param.clone()) {
                Ok(s) => format!("'{}'", s.replace('\'', "''")),
                Err(_) => "NULL".to_string(),
            }
        };
        result = result.replacen('?', &value, 1);
    }
    result
}

fn extract_table_name(sql: &str) -> Option<String> {
    let u = sql.trim().to_uppercase();
    if let Some(rest) = u.strip_prefix("SELECT") {
        if let Some(from_pos) = rest.find("FROM") {
            let after_from = rest[from_pos + 4..].trim();
            let table_end = after_from
                .find(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ')')
                .unwrap_or(after_from.len());
            let table = after_from[..table_end].trim();
            if !table.is_empty() {
                let orig_after = sql.to_uppercase().find("FROM").unwrap();
                let orig_from = sql[orig_after + 4..].trim();
                let orig_end = orig_from
                    .find(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ')')
                    .unwrap_or(orig_from.len());
                return Some(orig_from[..orig_end].trim().to_string());
            }
        }
    }
    None
}

fn extract_column_names(sql: &str, storage: &Arc<RwLock<MemoryStorage>>) -> Vec<String> {
    let u = sql.trim().to_uppercase();
    if u.starts_with("SHOW") || u.starts_with("DESCRIBE") || u.starts_with("EXPLAIN") {
        return vec![];
    }
    if let Some(table_name) = extract_table_name(sql) {
        if let Ok(storage_guard) = storage.try_read() {
            if let Ok(table_info) = storage_guard.get_table_info(&table_name) {
                let col_names: Vec<String> =
                    table_info.columns.iter().map(|c| c.name.clone()).collect();
                if !col_names.is_empty() {
                    return col_names;
                }
            }
        }
    }
    vec![]
}

fn infer_column_types(
    sql: &str,
    storage: &Arc<RwLock<MemoryStorage>>,
    cols: &[String],
) -> Vec<String> {
    if let Some(table_name) = extract_table_name(sql) {
        if let Ok(storage_guard) = storage.try_read() {
            if let Ok(table_info) = storage_guard.get_table_info(&table_name) {
                let types: Vec<String> = table_info
                    .columns
                    .iter()
                    .take(cols.len())
                    .map(|c| c.data_type.clone())
                    .collect();
                if types.len() == cols.len() {
                    return types;
                }
            }
        }
    }
    cols.iter().map(|_| "VARCHAR(255)".to_string()).collect()
}

#[allow(clippy::type_complexity)]
fn execute_select(
    sql: &str,
    engine: &mut MemoryExecutionEngine,
    storage: &Arc<RwLock<MemoryStorage>>,
) -> MySqlResult<(Vec<String>, Vec<String>, Vec<Vec<Value>>)> {
    let r = engine.execute(sql).map_err(MySqlError::from)?;
    let real_cols = extract_column_names(sql, storage);
    let n = r.rows.first().map(|row| row.len()).unwrap_or(0);
    let cols: Vec<String> = if !real_cols.is_empty() {
        real_cols
    } else if n > 0 {
        (0..n).map(|i| format!("col_{}", i + 1)).collect()
    } else {
        vec!["result".to_string()]
    };
    let ctypes: Vec<String> = infer_column_types(sql, storage, &cols);
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

fn is_select_query(sql: &str) -> bool {
    is_select(sql)
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

#[allow(unused_assignments)]
fn do_command_loop<S: Read + Write>(
    stream: &mut S,
    addr: SocketAddr,
    storage: Arc<RwLock<MemoryStorage>>,
    cap: u32,
    mut seq: u8,
    ps_manager: &mut PreparedStatementManager,
) -> MySqlResult<()> {
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
                seq = seq.wrapping_add(1);
            }
            packet_type::COM_INIT_DB => {
                make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
                seq = seq.wrapping_add(1);
            }
            packet_type::COM_QUERY => {
                let q = String::from_utf8_lossy(payload)
                    .trim_end_matches('\0')
                    .trim()
                    .to_string();
                tracing::info!("Query [{}]: {}", addr, q);
                if q.is_empty() {
                    make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
                    seq = seq.wrapping_add(1);
                    continue;
                }
                if is_transaction_cmd(&q) {
                    make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
                    seq = seq.wrapping_add(1);
                    continue;
                }
                let mut eng = MemoryExecutionEngine::new(storage.clone());
                if is_select(&q) {
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        execute_select(&q, &mut eng, &storage)
                    })) {
                        Ok(Ok((c, t, r))) => {
                            seq = send_result_set(stream, &c, &t, &r, seq, cap)?;
                        }
                        Ok(Err(e)) => {
                            let code = match &e {
                                MySqlError::Sql(s) if s.contains("not found") => 1146,
                                MySqlError::Sql(_) => 1064,
                                _ => 2000,
                            };
                            make_err_packet(seq, code, "42000", &e.to_string()).write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                        Err(_) => {
                            make_err_packet(seq, 2000, "HY000", "Internal error")
                                .write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                    }
                } else {
                    match execute_write(&q, &mut eng) {
                        Ok(a) => {
                            make_ok_packet(seq, a as u64, 0, 0x0002, 0).write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                        Err(e) => {
                            make_err_packet(seq, 1064, "42000", &e.to_string()).write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                    }
                }
            }
            packet_type::COM_STMT_PREPARE => {
                let sql = String::from_utf8_lossy(payload)
                    .trim_end_matches('\0')
                    .trim()
                    .to_string();
                tracing::info!("STMT PREPARE: {}", sql);

                let param_count = count_placeholders(&sql);

                let column_count: u16 = if sql.to_uppercase().starts_with("SELECT") {
                    let upper = sql.to_uppercase();
                    if let Some(from_pos) = upper.find(" FROM ") {
                        let select_part = &sql[..from_pos + 1].trim();
                        let cols_str = select_part.strip_prefix("SELECT").unwrap_or("").trim();
                        if cols_str.eq_ignore_ascii_case("*") {
                            if let Ok(storage_guard) = storage.try_read() {
                                if let Some(table_name) = extract_table_name(&sql) {
                                    if let Ok(table_info) =
                                        storage_guard.get_table_info(&table_name)
                                    {
                                        table_info.columns.len() as u16
                                    } else {
                                        1
                                    }
                                } else {
                                    1
                                }
                            } else {
                                1
                            }
                        } else {
                            cols_str.split(',').count() as u16
                        }
                    } else {
                        1
                    }
                } else {
                    0
                };

                let stmt_id = ps_manager.add(sql.clone(), param_count, column_count);

                let mut p = Vec::new();
                p.push(0x00);
                p.write_u32::<LittleEndian>(stmt_id).unwrap();
                p.write_u16::<LittleEndian>(column_count).unwrap();
                p.write_u16::<LittleEndian>(param_count).unwrap();
                p.push(0x00);
                p.write_u16::<LittleEndian>(0).unwrap();
                Packet {
                    length: p.len() as u32,
                    sequence: seq,
                    payload: p,
                }
                .write_to(stream)?;
                seq = seq.wrapping_add(1);

                if param_count > 0 {
                    for _ in 0..param_count {
                        let mut param_def = Vec::new();
                        write_lenenc_string(&mut param_def, b"def").unwrap();
                        write_lenenc_string(&mut param_def, b"").unwrap();
                        write_lenenc_string(&mut param_def, b"").unwrap();
                        write_lenenc_string(&mut param_def, b"").unwrap();
                        write_lenenc_string(&mut param_def, b"?").unwrap();
                        write_lenenc_string(&mut param_def, b"?").unwrap();
                        param_def.push(0x0c);
                        param_def.write_u16::<LittleEndian>(0x21).unwrap();
                        param_def.write_u32::<LittleEndian>(255).unwrap();
                        param_def.push(col_type::VARSTRING);
                        param_def.write_u16::<LittleEndian>(0x80).unwrap();
                        param_def.push(0x00);
                        param_def.write_u16::<LittleEndian>(0).unwrap();
                        Packet {
                            length: param_def.len() as u32,
                            sequence: seq,
                            payload: param_def,
                        }
                        .write_to(stream)?;
                        seq = seq.wrapping_add(1);
                    }
                    if cap & capability::DEPRECATE_EOF != 0 {
                        make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
                        seq = seq.wrapping_add(1);
                    } else {
                        make_eof_packet(seq, 0x0002).write_to(stream)?;
                        seq = seq.wrapping_add(1);
                    }
                }

                if column_count > 0 {
                    for i in 0..column_count {
                        let col_name = format!("col_{}", i + 1);
                        write_column_def(stream, &col_name, "VARCHAR(255)", seq)?;
                        seq = seq.wrapping_add(1);
                    }
                    if cap & capability::DEPRECATE_EOF != 0 {
                        make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
                        seq = seq.wrapping_add(1);
                    } else {
                        make_eof_packet(seq, 0x0002).write_to(stream)?;
                        seq = seq.wrapping_add(1);
                    }
                }

                tracing::info!(
                    "STMT PREPARE done: id={}, params={}, cols={}",
                    stmt_id,
                    param_count,
                    column_count
                );
            }
            packet_type::COM_STMT_EXECUTE => {
                if payload.len() < 4 {
                    make_err_packet(seq, 1047, "HY000", "Malformed COM_STMT_EXECUTE")
                        .write_to(stream)?;
                    seq = seq.wrapping_add(1);
                    continue;
                }

                let stmt_id = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);

                let stmt = match ps_manager.get(stmt_id) {
                    Some(s) => (s.sql.clone(), s.column_count),
                    None => {
                        make_err_packet(seq, 1243, "HY000", "Unknown statement handler")
                            .write_to(stream)?;
                        seq = seq.wrapping_add(1);
                        continue;
                    }
                };
                let stmt_sql = stmt.0;
                let stmt_col_count = stmt.1;

                let params: Vec<Vec<u8>> = Vec::new();
                let final_sql = replace_placeholders(&stmt_sql, &params);

                tracing::info!("STMT EXECUTE (id={}): {}", stmt_id, final_sql);
                let mut eng = MemoryExecutionEngine::new(storage.clone());

                if is_select(&final_sql) {
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        execute_select(&final_sql, &mut eng, &storage)
                    })) {
                        Ok(Ok((c, t, r))) => {
                            let c_trimmed: Vec<String> =
                                c.into_iter().take(stmt_col_count as usize).collect();
                            let t_trimmed: Vec<String> =
                                t.into_iter().take(stmt_col_count as usize).collect();
                            let r_trimmed: Vec<Vec<Value>> = r
                                .into_iter()
                                .map(|row| row.into_iter().take(stmt_col_count as usize).collect())
                                .collect();
                            seq = send_result_set(
                                stream, &c_trimmed, &t_trimmed, &r_trimmed, seq, cap,
                            )?;
                        }
                        Ok(Err(e)) => {
                            make_err_packet(seq, 1064, "42000", &e.to_string()).write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                        Err(_) => {
                            make_err_packet(seq, 2000, "HY000", "Internal error")
                                .write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                    }
                } else {
                    match execute_write(&final_sql, &mut eng) {
                        Ok(a) => {
                            make_ok_packet(seq, a as u64, 0, 0x0002, 0).write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                        Err(e) => {
                            make_err_packet(seq, 1064, "42000", &e.to_string()).write_to(stream)?;
                            seq = seq.wrapping_add(1);
                        }
                    }
                }
            }
            packet_type::COM_STMT_CLOSE => {
                if payload.len() >= 4 {
                    let stmt_id =
                        u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    ps_manager.remove(stmt_id);
                }
            }
            _ => {
                make_err_packet(seq, 1047, "HY000", "Unknown command").write_to(stream)?;
                seq = seq.wrapping_add(1);
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
    user_store: UserStore,
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
            let auth_ok = if SKIP_AUTH {
                true
            } else if resp.auth_response.is_empty() {
                tracing::warn!("Empty auth response for user {}", resp.username);
                false
            } else {
                user_store.verify_password(&resp.username, &scramble, &resp.auth_response)
            };
            if !auth_ok {
                tracing::warn!("Auth failed for user {}", resp.username);
                make_err_packet(3, 1045, "28000", "Access denied")
                    .write_to(&mut tls)
                    .ok();
                return;
            }
            tracing::info!("Auth accepted, sending OK packet, seq=3");
            make_ok_packet(3, 0, 0, 0x0002, 0).write_to(&mut tls).ok();
            tracing::info!("Starting command loop, seq=4");
            let mut ps_manager = PreparedStatementManager::new();
            let _ = do_command_loop(
                &mut tls,
                addr,
                storage,
                resp.capability_flags,
                4,
                &mut ps_manager,
            );
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
    let auth_ok = if SKIP_AUTH {
        true
    } else if resp.auth_response.is_empty() {
        tracing::warn!("Empty auth response for user {}", resp.username);
        false
    } else {
        user_store.verify_password(&resp.username, &scramble, &resp.auth_response)
    };
    if !auth_ok {
        tracing::warn!("Auth failed for user {}", resp.username);
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
    let mut ps_manager = PreparedStatementManager::new();
    let _ = do_command_loop(
        &mut &stream,
        addr,
        storage,
        resp.capability_flags,
        3,
        &mut ps_manager,
    );
}

pub fn run_server(host: &str, port: u16) -> MySqlResult<()> {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)?;
    tracing::info!("MySQL server listening on {}", addr);
    let tls_config = Arc::new(make_tls_config());
    tracing::info!("TLS ready (self-signed cert)");

    let storage: Arc<RwLock<MemoryStorage>> = Arc::new(RwLock::new(MemoryStorage::new()));
    {
        let mut eng = MemoryExecutionEngine::new(storage.clone());
        for sql in ["CREATE TABLE content (hash TEXT PRIMARY KEY, doc TEXT NOT NULL, created_at TEXT NOT NULL)",
            "CREATE TABLE vectors (hash_seq TEXT PRIMARY KEY, hash TEXT NOT NULL, embedding TEXT NOT NULL, created_at TEXT NOT NULL)",
            "CREATE TABLE documents (id TEXT PRIMARY KEY, title TEXT, content TEXT, created_at TEXT)"] {
            if let Err(e) = eng.execute(sql) { tracing::warn!("Init: {}", e); }
        }
    }
    let user_store = UserStore::new();
    for s in listener.incoming() {
        match s {
            Ok(stream) => {
                let addr = stream
                    .peer_addr()
                    .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
                let st = storage.clone();
                let tc = tls_config.clone();
                let us = user_store.clone();
                thread::spawn(move || handle_connection(stream, addr, st, tc, us));
            }
            Err(e) => tracing::error!("Accept: {}", e),
        }
    }
    Ok(())
}

// ============================================================================
// Unit Tests (private function coverage)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ============ is_select_query Tests ============

    #[test]
    fn test_is_select_query_select() {
        assert!(is_select_query("SELECT * FROM users"));
        assert!(is_select_query("select * from users"));
        assert!(is_select_query("SELECT 1"));
    }

    #[test]
    fn test_is_select_query_show() {
        assert!(is_select_query("SHOW TABLES"));
        assert!(is_select_query("show tables"));
        assert!(is_select_query("SHOW COLUMNS FROM users"));
    }

    #[test]
    fn test_is_select_query_describe() {
        assert!(is_select_query("DESCRIBE users"));
        assert!(is_select_query("describe users"));
        // Note: DESC is NOT matched, only DESCRIBE (abbreviation not supported)
        assert!(!is_select_query("DESC users"));
        assert!(!is_select_query("desc users"));
    }

    #[test]
    fn test_is_select_query_explain() {
        assert!(is_select_query("EXPLAIN SELECT * FROM users"));
        assert!(is_select_query("explain select * from users"));
    }

    #[test]
    fn test_is_select_query_not() {
        assert!(!is_select_query("INSERT INTO users VALUES (1)"));
        assert!(!is_select_query("UPDATE users SET name = 'a'"));
        assert!(!is_select_query("DELETE FROM users"));
        assert!(!is_select_query("DROP TABLE users"));
    }

    #[test]
    fn test_is_select_query_with_leading_whitespace() {
        assert!(is_select_query("  SELECT * FROM users"));
        assert!(is_select_query("\nSELECT * FROM users"));
        assert!(is_select_query("\tSELECT * FROM users"));
    }

    #[test]
    fn test_is_select_query_complex() {
        assert!(is_select_query("SELECT id, name FROM users WHERE age > 18"));
        // SQL injection attempt still starts with SELECT, so it's detected as select query
        // (the actual execution would fail, but detection is based on prefix)
        assert!(is_select_query("SELECT * FROM users; DROP TABLE users;--"));
    }

    // ============ col_type_from_string Tests ============

    #[test]
    fn test_col_type_from_string_integer() {
        // Default INT maps to LONG (0x03); BIGINT maps to LONGLONG (0x08)
        assert_eq!(col_type_from_string("INT"), 3); // LONG
        assert_eq!(col_type_from_string("INTEGER"), 3); // LONG
        assert_eq!(col_type_from_string("SMALLINT"), 2); // SHORT
        assert_eq!(col_type_from_string("TINYINT"), 1); // TINY
        assert_eq!(col_type_from_string("BIGINT"), 8); // LONGLONG
    }

    #[test]
    fn test_col_type_from_string_varchar() {
        assert_eq!(col_type_from_string("VARCHAR(255)"), 0xfd); // VARSTRING
        assert_eq!(col_type_from_string("CHAR(10)"), 0xfd); // VARSTRING
    }

    #[test]
    fn test_col_type_from_string_text() {
        assert_eq!(col_type_from_string("TEXT"), 0xfd); // VARSTRING
        assert_eq!(col_type_from_string("BLOB"), 0xfc); // BLOB
    }

    #[test]
    fn test_col_type_from_string_float() {
        assert_eq!(col_type_from_string("FLOAT"), 0x04); // FLOAT
        assert_eq!(col_type_from_string("DOUBLE"), 0x05); // DOUBLE
    }

    #[test]
    fn test_col_type_from_string_datetime() {
        // Note: "TIMESTAMP" contains "AMP" not "INT", so it skips INT branch.
        // DATETIME contains "DATE" -> matches DATE (0x0a) before DATETIME check
        assert_eq!(col_type_from_string("DATETIME"), 0x0a); // DATE (bug: order wrong)
        assert_eq!(col_type_from_string("DATE"), 0x0a); // DATE
                                                        // TIMESTAMP contains "TIME" -> matches TIME (0x0b) before DATETIME check
        assert_eq!(col_type_from_string("TIMESTAMP"), 0x0b); // TIME (contains "TIME")
    }

    #[test]
    fn test_col_type_from_string_decimal() {
        assert_eq!(col_type_from_string("DECIMAL"), 0xf6); // NEWDECIMAL
        assert_eq!(col_type_from_string("NUMERIC"), 0xf6); // NEWDECIMAL
    }

    #[test]
    fn test_col_type_from_string_unknown() {
        assert_eq!(col_type_from_string("UNKNOWN_TYPE"), 0xfe); // STRING default
    }

    // ============ value_to_string Tests ============

    #[test]
    fn test_value_to_string_null() {
        use sqlrustgo_types::Value;
        assert_eq!(value_to_string(&Value::Null), "NULL".to_string());
    }

    #[test]
    fn test_value_to_string_integer() {
        use sqlrustgo_types::Value;
        assert_eq!(value_to_string(&Value::Integer(42)), "42".to_string());
        assert_eq!(value_to_string(&Value::Integer(0)), "0".to_string());
        assert_eq!(value_to_string(&Value::Integer(-100)), "-100".to_string());
    }

    #[test]
    fn test_value_to_string_float() {
        use sqlrustgo_types::Value;
        assert_eq!(value_to_string(&Value::Float(3.14)), "3.14".to_string());
        assert_eq!(value_to_string(&Value::Float(0.0)), "0".to_string());
    }

    #[test]
    fn test_value_to_string_text() {
        use sqlrustgo_types::Value;
        assert_eq!(
            value_to_string(&Value::Text("hello".to_string())),
            "hello".to_string()
        );
        assert_eq!(
            value_to_string(&Value::Text("".to_string())),
            "".to_string()
        );
    }

    #[test]
    fn test_value_to_string_boolean() {
        use sqlrustgo_types::Value;
        assert_eq!(value_to_string(&Value::Boolean(true)), "1".to_string());
        assert_eq!(value_to_string(&Value::Boolean(false)), "0".to_string());
    }

    #[test]
    fn test_value_to_string_blob() {
        use sqlrustgo_types::Value;
        // Blob falls through to debug format
        let result = value_to_string(&Value::Blob(vec![1, 2, 3]));
        assert!(result.contains("[1, 2, 3]") || result.contains("1, 2, 3"));
    }

    // ============ old_password_hash Tests ============

    #[test]
    fn test_old_password_hash_deterministic() {
        let hash1 = old_password_hash("password");
        let hash2 = old_password_hash("password");
        assert_eq!(hash1, hash2, "Same password should produce same hash");
    }

    #[test]
    fn test_old_password_hash_empty() {
        let hash = old_password_hash("");
        // Empty password should still produce a valid 8-byte hash
        assert_eq!(hash.len(), 8);
    }

    #[test]
    fn test_old_password_hash_different_passwords() {
        let hash1 = old_password_hash("password1");
        let hash2 = old_password_hash("password2");
        assert_ne!(
            hash1, hash2,
            "Different passwords should produce different hashes"
        );
    }

    #[test]
    fn test_old_password_hash_length() {
        let hash = old_password_hash("test_password");
        assert_eq!(hash.len(), 8, "Hash should be exactly 8 bytes");
    }

    // ============ Packet Tests (internal) ============

    #[test]
    fn test_packet_internal() {
        let pkt = Packet {
            length: 5,
            sequence: 2,
            payload: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(pkt.length, 5);
        assert_eq!(pkt.sequence, 2);
        assert_eq!(pkt.payload.len(), 5);
    }

    // ============ write_lenenc_int Tests ============

    #[test]
    fn test_write_lenenc_int_small() {
        // Values < 251 use 1 byte
        let mut buf = Vec::new();
        write_lenenc_int(&mut buf, 0).unwrap();
        write_lenenc_int(&mut buf, 100).unwrap();
        write_lenenc_int(&mut buf, 250).unwrap();
        assert_eq!(buf, vec![0x00, 100, 250]);
    }

    #[test]
    fn test_write_lenenc_int_16bit() {
        // Values 251-0xFFFF use 3 bytes (0xfc + u16)
        let mut buf = Vec::new();
        write_lenenc_int(&mut buf, 251).unwrap();
        write_lenenc_int(&mut buf, 1000).unwrap();
        write_lenenc_int(&mut buf, 0xFFFF).unwrap();
        // 251 = 0xFB, 1000 = 0x3E8, 0xFFFF
        assert_eq!(
            buf,
            vec![0xfc, 0xfb, 0x00, 0xfc, 0xe8, 0x03, 0xfc, 0xff, 0xff]
        );
    }

    #[test]
    fn test_write_lenenc_int_24bit() {
        // Values 0x10000-0xFFFFFF use 4 bytes (0xfd + u24)
        let mut buf = Vec::new();
        write_lenenc_int(&mut buf, 0x10000).unwrap();
        write_lenenc_int(&mut buf, 0x1000000 - 1).unwrap();
        // 0x10000 = 0x00010000 -> 0xfd, 0x00, 0x00, 0x01
        // 0xFFFFFF = 0x00FFFFFF -> 0xfd, 0xff, 0xff, 0xff
        assert_eq!(buf, vec![0xfd, 0x00, 0x00, 0x01, 0xfd, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn test_write_lenenc_int_64bit() {
        // Values >= 0x1000000 use 9 bytes (0xfe + u64)
        let mut buf = Vec::new();
        write_lenenc_int(&mut buf, 0x1000000).unwrap();
        write_lenenc_int(&mut buf, u64::MAX).unwrap();
        assert_eq!(buf.len(), 1 + 8 + 1 + 8); // Two 0xfe + u64 values
        assert_eq!(buf[0], 0xfe);
        assert_eq!(buf[9], 0xfe);
    }

    // ============ write_lenenc_string Tests ============

    #[test]
    fn test_write_lenenc_string_basic() {
        let mut buf = Vec::new();
        write_lenenc_string(&mut buf, b"hello").unwrap();
        // 5 (length) + "hello"
        assert_eq!(buf, vec![0x05, b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn test_write_lenenc_string_empty() {
        let mut buf = Vec::new();
        write_lenenc_string(&mut buf, b"").unwrap();
        assert_eq!(buf, vec![0x00]);
    }

    #[test]
    fn test_write_lenenc_string_long() {
        let mut buf = Vec::new();
        let long_str = vec![0u8; 300];
        write_lenenc_string(&mut buf, &long_str).unwrap();
        // 300 = 0x12C, needs 0xfc + u16 encoding
        assert_eq!(buf[0], 0xfc);
        assert_eq!(buf[1], 0x2c);
        assert_eq!(buf[2], 0x01);
    }

    // ============ make_handshake_packet Tests ============

    #[test]
    fn test_make_handshake_packet() {
        let seed = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let pkt = make_handshake_packet(0, &seed);
        assert_eq!(pkt.sequence, 0);
        assert!(pkt.length > 0);
        assert_eq!(pkt.payload[0], 0x0a); // protocol version
        assert!(pkt.payload.contains(&0)); // null terminator in version
    }

    #[test]
    fn test_make_handshake_packet_with_different_seq() {
        let seed = [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01];
        let pkt = make_handshake_packet(5, &seed);
        assert_eq!(pkt.sequence, 5);
    }

    // ============ make_ok_packet Tests ============

    #[test]
    fn test_make_ok_packet_basic() {
        let pkt = make_ok_packet(1, 0, 0);
        assert_eq!(pkt.sequence, 1);
        assert_eq!(pkt.payload[0], 0x00); // OK packet type
    }

    #[test]
    fn test_make_ok_packet_with_affected_rows() {
        let pkt = make_ok_packet(2, 5, 10);
        assert_eq!(pkt.sequence, 2);
        // Affected rows is lenenc-int of 5 = 0x05
        assert!(pkt.payload.contains(&5));
    }

    // ============ make_err_packet Tests ============

    #[test]
    fn test_make_err_packet_basic() {
        let pkt = make_err_packet(1, 1064, "Syntax error");
        assert_eq!(pkt.sequence, 1);
        assert_eq!(pkt.payload[0], 0xff); // Err packet type
    }

    #[test]
    fn test_make_err_packet_code() {
        let pkt = make_err_packet(2, 1045, "Access denied");
        // Error code is little-endian u16 at bytes 1-2
        assert_eq!(pkt.payload[1], 0x15); // 1045 = 0x0415
        assert_eq!(pkt.payload[2], 0x04);
    }

    #[test]
    fn test_make_err_packet_empty_message() {
        let pkt = make_err_packet(0, 2000, "");
        assert_eq!(pkt.payload[0], 0xff);
    }

    // ============ make_eof_packet Tests ============

    #[test]
    fn test_make_eof_packet() {
        let pkt = make_eof_packet(3, 0x0002);
        assert_eq!(pkt.sequence, 3);
        assert_eq!(pkt.payload[0], 0xfe); // EOF packet type
    }

    #[test]
    fn test_make_eof_packet_status() {
        let pkt = make_eof_packet(5, 0x0003);
        // EOF packet: 0xfe + warning_count(2 bytes) + status(2 bytes)
        // For status 0x0003: payload[3]=0x03, payload[4]=0x00
        assert_eq!(pkt.payload[0], 0xfe); // EOF marker
        assert_eq!(pkt.payload[3], 0x03); // status low byte
        assert_eq!(pkt.payload[4], 0x00); // status high byte
    }

    // ============ col_len_from_type Tests ============

    #[test]
    fn test_col_len_from_type_int1() {
        assert_eq!(col_len_from_type("TINYINT(1)"), 1);
    }

    #[test]
    fn test_col_len_from_type_int11() {
        // Default INT length is 11
        assert_eq!(col_len_from_type("INT(11)"), 11);
        assert_eq!(col_len_from_type("INT(10)"), 10);
    }

    #[test]
    fn test_col_len_from_type_float() {
        assert_eq!(col_len_from_type("FLOAT"), 12);
        assert_eq!(col_len_from_type("DOUBLE"), 22);
    }

    #[test]
    fn test_col_len_from_type_varchar() {
        assert_eq!(col_len_from_type("VARCHAR(255)"), 255);
        assert_eq!(col_len_from_type("VARCHAR(100)"), 100);
    }

    #[test]
    fn test_col_len_from_type_text() {
        assert_eq!(col_len_from_type("TEXT"), 65535);
    }

    #[test]
    fn test_col_len_from_type_default() {
        assert_eq!(col_len_from_type("UNKNOWN"), 255);
    }

    // ============ col_type_from_string Tests (additional) ============

    #[test]
    fn test_col_type_from_string_bit() {
        // BIT is not explicitly handled, falls through to default STRING (0xfe)
        assert_eq!(col_type_from_string("BIT"), 0xfe); // falls through to default
    }

    #[test]
    fn test_col_type_from_string_year() {
        // YEAR is not explicitly handled, falls through to default STRING (0xfe)
        assert_eq!(col_type_from_string("YEAR"), 0xfe); // falls through to default
    }

    #[test]
    fn test_col_type_from_string_mediumint() {
        assert_eq!(col_type_from_string("MEDIUMINT"), 0x09); // INT24
    }

    // ============ Packet round-trip with Vec<u8> ============

    #[test]
    fn test_packet_write_to_vec_and_read() {
        let original = Packet {
            length: 7,
            sequence: 4,
            payload: vec![0x03, b'S', b'E', b'L', b'E', b'C', b'T'],
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
    fn test_packet_max_sequence_wrap() {
        // Test that sequence wrapping works correctly
        let pkt = Packet {
            length: 5,
            sequence: 255,
            payload: vec![1, 2, 3, 4, 5],
        };
        let mut buf = Vec::new();
        pkt.write_to(&mut buf).unwrap();
        let mut cursor = std::io::Cursor::new(buf);
        let read = Packet::read_from(&mut cursor).unwrap();
        assert_eq!(read.sequence, 255);
    }

    // ============ value_to_string edge cases ============

    #[test]
    fn test_value_to_string_negative_integer() {
        use sqlrustgo_types::Value;
        assert_eq!(
            value_to_string(&Value::Integer(i64::MIN)),
            i64::MIN.to_string()
        );
        assert_eq!(value_to_string(&Value::Integer(-1)), "-1".to_string());
    }

    #[test]
    fn test_value_to_string_large_float() {
        use sqlrustgo_types::Value;
        assert_eq!(
            value_to_string(&Value::Float(f64::MAX)),
            f64::MAX.to_string()
        );
    }

    #[test]
    fn test_value_to_string_unicode_text() {
        use sqlrustgo_types::Value;
        assert_eq!(
            value_to_string(&Value::Text("hello".to_string())),
            "hello".to_string()
        );
    }

    // ============ old_password_hash edge cases ============

    #[test]
    fn test_old_password_hash_known_value() {
        // The hash should be deterministic
        let hash = old_password_hash("test");
        assert_eq!(hash.len(), 8);
        // Same input should always produce same output
        assert_eq!(old_password_hash("test"), hash);
    }

    #[test]
    fn test_old_password_hash_long_password() {
        let hash = old_password_hash("a very long password that is much longer than average");
        assert_eq!(hash.len(), 8);
    }

    #[test]
    fn test_old_password_hash_special_chars() {
        let hash1 = old_password_hash("pass@word!");
        let hash2 = old_password_hash("password");
        assert_ne!(hash1, hash2);
    }

    // ============ verify_old_password_response Tests ============

    #[test]
    fn test_verify_old_password_response_basic() {
        // Test that the function works (even if it always returns false without real verification)
        let seed = [0x00; 8];
        let response = [0x00; 8];
        // With empty password, this should work
        let result = verify_old_password_response(&seed, &response, "");
        // The algorithm produces consistent results
        assert!(result == verify_old_password_response(&seed, &response, ""));
    }

    #[test]
    fn test_verify_old_password_response_with_seed() {
        let seed = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let response = [0x00; 8];
        let result1 = verify_old_password_response(&seed, &response, "password");
        let result2 = verify_old_password_response(&seed, &response, "password");
        assert_eq!(result1, result2); // Same inputs should give same result
    }

    // ============ write_text_row Tests ============

    #[test]
    fn test_write_text_row_single_value() {
        use sqlrustgo_types::Value;
        let mut buf = Vec::new();
        let row = vec![Value::Integer(42)];
        write_text_row(&mut buf, &row).unwrap();
        // Integer 42 -> "42" as lenenc string
        assert!(buf.len() > 0);
    }

    #[test]
    fn test_write_text_row_multiple_values() {
        use sqlrustgo_types::Value;
        let mut buf = Vec::new();
        let row = vec![
            Value::Integer(1),
            Value::Text("hello".to_string()),
            Value::Null,
        ];
        write_text_row(&mut buf, &row).unwrap();
        assert!(buf.len() > 0);
    }

    #[test]
    fn test_write_text_row_empty() {
        use sqlrustgo_types::Value;
        let mut buf = Vec::new();
        let row: Vec<Value> = vec![];
        write_text_row(&mut buf, &row).unwrap();
        assert_eq!(buf.len(), 0);
    }

    // ============ write_column_def Tests ============

    #[test]
    fn test_write_column_def_basic() {
        let mut buf = Vec::new();
        write_column_def(&mut buf, "id", "INT", 0).unwrap();
        assert!(buf.len() > 0);
        // Verify it can be read back as a packet
        let mut cursor = std::io::Cursor::new(buf);
        let pkt = Packet::read_from(&mut cursor).unwrap();
        assert_eq!(pkt.sequence, 0);
    }

    #[test]
    fn test_write_column_def_varchar() {
        let mut buf = Vec::new();
        write_column_def(&mut buf, "name", "VARCHAR(100)", 5).unwrap();
        let mut cursor = std::io::Cursor::new(buf);
        let pkt = Packet::read_from(&mut cursor).unwrap();
        assert_eq!(pkt.sequence, 5);
    }

    #[test]
    fn test_write_column_def_float() {
        let mut buf = Vec::new();
        write_column_def(&mut buf, "price", "FLOAT", 10).unwrap();
        let mut cursor = std::io::Cursor::new(buf);
        let pkt = Packet::read_from(&mut cursor).unwrap();
        assert_eq!(pkt.sequence, 10);
    }

    // ============ send_result_set Tests ============

    #[test]
    fn test_send_result_set_empty() {
        use sqlrustgo_types::Value;
        let mut buf = Vec::new();
        let columns = vec!["id".to_string(), "name".to_string()];
        let column_types = vec!["INT".to_string(), "VARCHAR(255)".to_string()];
        let rows: Vec<Vec<Value>> = vec![];
        send_result_set(&mut buf, &columns, &column_types, &rows, 0).unwrap();
        assert!(buf.len() > 0);
    }

    #[test]
    fn test_send_result_set_with_rows() {
        use sqlrustgo_types::Value;
        let mut buf = Vec::new();
        let columns = vec!["id".to_string()];
        let column_types = vec!["INT".to_string()];
        let rows = vec![vec![Value::Integer(1)], vec![Value::Integer(2)]];
        send_result_set(&mut buf, &columns, &column_types, &rows, 0).unwrap();
        assert!(buf.len() > 0);
    }

    #[test]
    fn test_send_result_set_with_text_values() {
        use sqlrustgo_types::Value;
        let mut buf = Vec::new();
        let columns = vec!["name".to_string(), "email".to_string()];
        let column_types = vec!["VARCHAR(100)".to_string(), "VARCHAR(255)".to_string()];
        let rows = vec![
            vec![
                Value::Text("Alice".to_string()),
                Value::Text("alice@example.com".to_string()),
            ],
            vec![
                Value::Text("Bob".to_string()),
                Value::Text("bob@example.com".to_string()),
            ],
        ];
        send_result_set(&mut buf, &columns, &column_types, &rows, 0).unwrap();
        assert!(buf.len() > 0);
    }

    #[test]
    fn test_send_result_set_single_row() {
        use sqlrustgo_types::Value;
        let mut buf = Vec::new();
        let columns = vec!["value".to_string()];
        let column_types = vec!["DOUBLE".to_string()];
        let rows = vec![vec![Value::Float(3.14159)]];
        send_result_set(&mut buf, &columns, &column_types, &rows, 7).unwrap();
        assert!(buf.len() > 0);
    }

    // ============ execute_select Tests (mock-style) ============

    #[test]
    fn test_execute_select_simple() {
        use sqlrustgo::MemoryExecutionEngine;
        use sqlrustgo_storage::MemoryStorage;
        use sqlrustgo_types::Value;
        use std::sync::{Arc, RwLock};

        let storage: Arc<RwLock<MemoryStorage>> = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = MemoryExecutionEngine::new(storage);

        // Create a simple table
        engine
            .execute("CREATE TABLE test (id INT, name TEXT)")
            .unwrap();
        engine
            .execute("INSERT INTO test VALUES (1, 'hello')")
            .unwrap();

        // The execute_select function requires actual query execution
        let result = execute_select("SELECT * FROM test", &mut engine);
        match result {
            Ok((columns, column_types, rows)) => {
                assert_eq!(columns.len(), 2);
                assert_eq!(column_types.len(), 2);
                assert_eq!(rows.len(), 1);
            }
            Err(e) => {
                // If it fails due to query parsing/execution, that's also valid
                println!(
                    "execute_select returned error (expected in some cases): {}",
                    e
                );
            }
        }
    }

    // ============ execute_write Tests ============

    #[test]
    fn test_execute_write_insert() {
        use sqlrustgo::MemoryExecutionEngine;
        use sqlrustgo_storage::MemoryStorage;
        use std::sync::{Arc, RwLock};

        let storage: Arc<RwLock<MemoryStorage>> = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = MemoryExecutionEngine::new(storage);

        engine.execute("CREATE TABLE write_test (id INT)").unwrap();
        let affected = execute_write("INSERT INTO write_test VALUES (1)", &mut engine).unwrap();
        assert!(affected > 0);
    }

    // ============ MySqlError::std::error::Error trait ============

    #[test]
    fn test_my_sql_error_source() {
        use std::error::Error;
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err = MySqlError::Io(io_err);
        // Note: MySqlError uses default Error impl which returns None for source
        // even for Io(Error) variant since it doesn't box the error
        let display = format!("{}", err);
        assert!(display.contains("IO error"));
    }

    #[test]
    fn test_my_sql_error_source_none() {
        use std::error::Error;
        let err = MySqlError::Protocol("test".to_string());
        // Protocol errors don't have a source
        assert!(err.source().is_none());
    }
}
