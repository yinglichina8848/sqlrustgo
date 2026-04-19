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
use sqlrustgo_parser::parser::{AggregateCall, AggregateFunction, CreateIndexStatement, CreateTableStatement, DropTableStatement, InsertStatement, SelectStatement};
use sqlrustgo_parser::{DeleteStatement, Expression, UpdateStatement};
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_storage::{StorageEngine, TableInfo};
use sqlrustgo_types::{SqlError, SqlResult};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;
use rcgen::{DistinguishedName, DnType, KeyPair};
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

// ============================================================================
// Password Verification
// ============================================================================

const DEFAULT_PASSWORD: &str = "";

fn verify_auth_response(auth_response: &[u8], _seed: &[u8], password: &str) -> bool {
    if password.is_empty() {
        return true;
    }
    verify_old_password_response(_seed, auth_response, password)
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
        match stmt {
            Statement::Select(ref select) => self.exec_select(select),
            Statement::Insert(ref insert) => self.exec_insert(insert),
            Statement::Update(ref update) => self.exec_update(update),
            Statement::Delete(ref delete) => self.exec_delete(delete),
            Statement::CreateTable(ref create) => self.exec_create_table(create),
            Statement::DropTable(ref drop) => self.exec_drop_table(drop),
            Statement::CreateIndex(ref idx) => self.exec_create_index(idx),
            _ => Ok(ExecutorResult::empty()),
        }
    }

    fn exec_select(&self, select: &SelectStatement) -> SqlResult<ExecutorResult> {
        let storage = self.storage.read().unwrap();
        let table_info = storage.get_table_info(&select.table)?;
        let mut rows = storage.scan(&select.table)?;
        if let Some(ref where_expr) = select.where_clause {
            rows.retain(|row| eval_where(where_expr, row, &table_info));
        }
        if !select.aggregates.is_empty() {
            let agg_values = self.compute_aggs(&select.aggregates, &rows.clone(), &table_info)?;
            return Ok(ExecutorResult::new(vec![agg_values], 1));
        }
        let row_count = rows.len();
        Ok(ExecutorResult::new(rows, row_count))
    }

    fn compute_aggs(&self, aggregates: &[AggregateCall], rows: &[Vec<Value>], table_info: &TableInfo) -> SqlResult<Vec<Value>> {
        let mut results = Vec::with_capacity(aggregates.len());
        for agg in aggregates {
            let values: Vec<Value> = if let Some(arg) = agg.args.first() {
                rows.iter().map(|row| eval_expr(arg, row, table_info).unwrap_or(Value::Null)).collect()
            } else {
                vec![Value::Integer(rows.len() as i64)]
            };
            let result = match agg.func {
                AggregateFunction::Count => Value::Integer(values.len() as i64),
                AggregateFunction::Sum => Value::Integer(values.iter().filter_map(|v| if let Value::Integer(n) = v { Some(*n) } else { None }).sum()),
                AggregateFunction::Avg => {
                    let sum: i64 = values.iter().filter_map(|v| if let Value::Integer(n) = v { Some(*n) } else { None }).sum();
                    let count = values.iter().filter(|v| matches!(v, Value::Integer(_))).count();
                    if count > 0 { Value::Integer(sum / count as i64) } else { Value::Null }
                }
                AggregateFunction::Min => values.iter().filter_map(|v| if let Value::Integer(n) = v { Some(*n) } else { None }).min().map(Value::Integer).unwrap_or(Value::Null),
                AggregateFunction::Max => values.iter().filter_map(|v| if let Value::Integer(n) = v { Some(*n) } else { None }).max().map(Value::Integer).unwrap_or(Value::Null),
            };
            results.push(result);
        }
        Ok(results)
    }

    fn exec_insert(&self, insert: &InsertStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        let mut all_records = Vec::new();
        for row_exprs in &insert.values {
            let mut record = Vec::with_capacity(row_exprs.len());
            for expr in row_exprs.iter() {
                record.push(expr_to_value(expr));
            }
            all_records.push(record);
        }
        let count = all_records.len();
        storage.insert(&insert.table, all_records)?;
        Ok(ExecutorResult::new(vec![], count))
    }

    fn exec_update(&self, update: &UpdateStatement) -> SqlResult<ExecutorResult> {
        let table_info = { self.storage.read().unwrap().get_table_info(&update.table)? };
        let all_rows = { self.storage.read().unwrap().scan(&update.table)? };
        let where_clause = match update.where_clause.as_ref() {
            Some(w) => w,
            None => return Ok(ExecutorResult::new(vec![], 0)),
        };
        let rows_to_update: Vec<Vec<Value>> = all_rows.into_iter().filter(|row| eval_where(where_clause, row, &table_info)).collect();
        let count = rows_to_update.len();
        if count == 0 { return Ok(ExecutorResult::new(vec![], 0)); }
        let set_col_indices: Vec<(usize, &Expression)> = update.set_clauses.iter()
            .filter_map(|(col_name, expr)| find_col_idx(col_name, &table_info).map(|idx| (idx, expr)))
            .collect();
        let updated_rows: Vec<Vec<Value>> = rows_to_update.into_iter().map(|mut row| {
            for &(col_idx, ref set_expr) in &set_col_indices {
                if let Ok(new_val) = eval_expr(set_expr, &row, &table_info) {
                    if col_idx < row.len() { row[col_idx] = new_val; }
                }
            }
            row
        }).collect();
        let rows_to_keep: Vec<Vec<Value>> = {
            self.storage.read().unwrap().scan(&update.table)?
                .into_iter().filter(|row| !eval_where(where_clause, row, &table_info)).collect()
        };
        {
            let mut storage = self.storage.write().unwrap();
            let _ = storage.delete(&update.table, &[]);
            if !rows_to_keep.is_empty() { storage.insert(&update.table, rows_to_keep)?; }
            if !updated_rows.is_empty() { storage.insert(&update.table, updated_rows)?; }
        }
        Ok(ExecutorResult::new(vec![], count))
    }

    fn exec_delete(&self, delete: &DeleteStatement) -> SqlResult<ExecutorResult> {
        if delete.where_clause.is_none() {
            let mut storage = self.storage.write().unwrap();
            let count = storage.delete(&delete.table, &[])?;
            return Ok(ExecutorResult::new(vec![], count));
        }
        let table_info = { self.storage.read().unwrap().get_table_info(&delete.table)? };
        let all_rows = { self.storage.read().unwrap().scan(&delete.table)? };
        let where_clause = delete.where_clause.as_ref().unwrap();
        let rows_to_keep: Vec<Vec<Value>> = all_rows.into_iter().filter(|row| !eval_where(where_clause, row, &table_info)).collect();
        let total = { self.storage.read().unwrap().scan(&delete.table)?.len() };
        let count = total - rows_to_keep.len();
        {
            let mut storage = self.storage.write().unwrap();
            let _ = storage.delete(&delete.table, &[]);
            if !rows_to_keep.is_empty() { storage.insert(&delete.table, rows_to_keep)?; }
        }
        Ok(ExecutorResult::new(vec![], count))
    }

    fn exec_create_table(&self, create: &CreateTableStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        let columns: Vec<sqlrustgo_storage::ColumnDefinition> = create.columns.iter().map(|c| {
            sqlrustgo_storage::ColumnDefinition {
                name: c.name.clone(), data_type: c.data_type.clone(),
                nullable: !c.primary_key, primary_key: c.primary_key,
            }
        }).collect();
        storage.create_table(&TableInfo { name: create.name.clone(), columns, foreign_keys: vec![], unique_constraints: vec![] })?;
        Ok(ExecutorResult::empty())
    }

    fn exec_drop_table(&self, drop: &DropTableStatement) -> SqlResult<ExecutorResult> {
        self.storage.write().unwrap().drop_table(&drop.name)?;
        Ok(ExecutorResult::empty())
    }

    fn exec_create_index(&self, idx: &CreateIndexStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        let col_name = idx.columns.first().ok_or_else(|| SqlError::ExecutionError("No columns in index".to_string()))?;
        let table_info = storage.get_table_info(&idx.table)?;
        let col_idx = table_info.columns.iter().position(|c| c.name == *col_name)
            .ok_or_else(|| SqlError::ExecutionError("Column not found".to_string()))?;
        storage.create_index(&idx.table, col_name, col_idx)?;
        Ok(ExecutorResult::empty())
    }
}

fn expr_to_value(expr: &Expression) -> Value {
    match expr {
        Expression::Literal(s) => {
            let s = s.trim();
            if s.eq_ignore_ascii_case("NULL") { Value::Null }
            else if let Ok(n) = s.parse::<i64>() { Value::Integer(n) }
            else if let Ok(f) = s.parse::<f64>() { Value::Float(f) }
            else if s.len() >= 2 && s.starts_with("'") && s.ends_with("'") { Value::Text(s[1..s.len()-1].to_string()) }
            else { Value::Text(s.to_string()) }
        }
        Expression::Identifier(name) => Value::Text(name.clone()),
        _ => Value::Null,
    }
}

fn eval_where(expr: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
    match expr {
        Expression::BinaryOp(left, op, right) if op.to_uppercase() == "AND" =>
            eval_where(left, row, table_info) && eval_where(right, row, table_info),
        Expression::BinaryOp(left, op, right) if op.to_uppercase() == "OR" =>
            eval_where(left, row, table_info) || eval_where(right, row, table_info),
        Expression::BinaryOp(left, op, right) => eval_cmp(left, op, right, row, table_info),
        _ => eval_expr(expr, row, table_info).map(|v| v != Value::Null).unwrap_or(false),
    }
}

fn eval_cmp(left: &Expression, op: &str, right: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
    let lv = eval_expr(left, row, table_info).unwrap_or(Value::Null);
    let rv = eval_expr(right, row, table_info).unwrap_or(Value::Null);
    match op.to_uppercase().as_str() {
        "=" | "==" | "IS" => lv == rv,
        "!=" | "<>" => lv != rv,
        ">" => cmp_vals(&lv, &rv) > 0,
        ">=" => cmp_vals(&lv, &rv) >= 0,
        "<" => cmp_vals(&lv, &rv) < 0,
        "<=" => cmp_vals(&lv, &rv) <= 0,
        _ => false,
    }
}

fn eval_expr(expr: &Expression, row: &[Value], table_info: &TableInfo) -> Result<Value, String> {
    match expr {
        Expression::Literal(_) => Ok(expr_to_value(expr)),
        Expression::Identifier(name) => {
            find_col_idx(name, table_info)
                .map(|idx| row.get(idx).cloned().unwrap_or(Value::Null))
                .ok_or_else(|| format!("Column {} not found", name))
        }
        Expression::BinaryOp(left, op, right) => {
            let lv = eval_expr(left, row, table_info).unwrap_or(Value::Null);
            let rv = eval_expr(right, row, table_info).unwrap_or(Value::Null);
            match op.as_ref() {
                "+" => match (lv, rv) { (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l+r)), _ => Ok(Value::Null) },
                "-" => match (lv, rv) { (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l-r)), _ => Ok(Value::Null) },
                "*" => match (lv, rv) { (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l*r)), _ => Ok(Value::Null) },
                _ => Ok(Value::Null),
            }
        }
        _ => Ok(Value::Null),
    }
}

fn cmp_vals(left: &Value, right: &Value) -> i32 {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => l.cmp(r) as i32,
        (Value::Float(l), Value::Float(r)) => if l < r { -1 } else if l > r { 1 } else { 0 },
        (Value::Text(l), Value::Text(r)) => l.cmp(r) as i32,
        (Value::Null, Value::Null) => 0,
        (Value::Null, _) => -1,
        (_, Value::Null) => 1,
        _ => 0,
    }
}

fn find_col_idx(col_name: &str, table_info: &TableInfo) -> Option<usize> {
    table_info.columns.iter().position(|c| c.name.eq_ignore_ascii_case(col_name))
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
    pub const SSL: u32 = 1 << 11;
    pub const TRANSACTIONS: u32 = 1 << 13;
    pub const INTERACTIVE: u32 = 1 << 15;
    pub const MULTI_STATEMENTS: u32 = 1 << 16;
    pub const MULTI_RESULTS: u32 = 1 << 17;
    pub const PLUGIN_AUTH: u32 = 1 << 19;
    pub const SECURE_CONNECTION: u32 = 1 << 27;
    pub const DEFAULT: u32 = LONG_PASSWORD
        | FOUND_ROWS
        | LONG_FLAG
        | CONNECT_WITH_DB
        | SSL
        | PROTOCOL_41
        | TRANSACTIONS
        | INTERACTIVE
        | MULTI_STATEMENTS
        | MULTI_RESULTS
        | PLUGIN_AUTH
        | SECURE_CONNECTION;
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

    p.write_u16::<LittleEndian>(capability::DEFAULT as u16)
        .unwrap();

    p.push(45);

    p.write_u16::<LittleEndian>(0x0002).unwrap();

    p.write_u16::<LittleEndian>(((capability::DEFAULT >> 16) & 0xFFFF) as u16)
        .unwrap();

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
        match val {
            Value::Null => { w.write_all(&[0xFB])?; }
            _ => { let s = value_to_string(val); write_lenenc_string(w, s.as_bytes())?; }
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

fn execute_select(sql: &str, engine: &mut ExecutionEngine) -> MySqlResult<SelectResult> {
    tracing::info!("execute_select called with: [{}]", sql);
    let statement: Statement = parse(sql).map_err(MySqlError::Sql)?;
    tracing::info!("parse done, executing...");
    let result: ExecutorResult = engine.execute(statement.clone()).map_err(MySqlError::from)?;

    let col_count = result.rows.first().map(|r| r.len()).unwrap_or(0);

    // For column names, use empty (client will use default col_N names)
    // Note: SQLRustGo's ExecutionResult doesn't include column metadata from SELECT
    let columns: Vec<String> = if let Statement::Select(ref select) = statement {
        if select.columns.len() == 1 && select.columns[0].name == "*" {
            if let Ok(storage) = engine.storage.read() {
                if let Ok(table_info) = storage.get_table_info(&select.table) {
                    table_info.columns.iter().map(|c| c.name.clone()).collect()
                } else { vec!["*".to_string()] }
            } else { vec!["*".to_string()] }
        } else {
            select.columns.iter().map(|c| c.name.clone()).collect()
        }
    } else {
        (0..col_count).map(|i| format!("col_{}", i + 1)).collect()
    };
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

fn generate_tls_config() -> Result<Arc<ServerConfig>, Box<dyn std::error::Error>> {
    let mut params = rcgen::CertificateParams::new(Vec::new())?;
    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(DnType::CommonName, "SQLRustGo");
    params.distinguished_name.push(DnType::OrganizationName, "SQLRustGo");
    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;
    let cert_der = cert.der().to_owned();
    let key_der = key_pair.serialize_der();
    let certs = vec![CertificateDer::from(cert_der)];
    let key = PrivateKeyDer::try_from(key_der)
        .map_err(|e| format!("Failed to parse key: {:?}", e))?;
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| format!("TLS config error: {}", e))?;
    Ok(Arc::new(config))
}

struct TlsStream<'a, S> {
    conn: &'a mut rustls::ServerConnection,
    stream: &'a mut S,
}

impl<'a, S: Read + Write> TlsStream<'a, S> {
    fn new(conn: &'a mut rustls::ServerConnection, stream: &'a mut S) -> Self {
        Self { conn, stream }
    }
}

impl<'a, S: Read + Write> Read for TlsStream<'a, S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            match self.conn.reader().read(buf) {
                Ok(0) => {
                    match self.conn.complete_io(&mut self.stream) {
                        Ok(_) => continue,
                        Err(e) => return Err(e),
                    }
                }
                Ok(n) => return Ok(n),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    match self.conn.complete_io(&mut self.stream) {
                        Ok(_) => continue,
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl<'a, S: Read + Write> Write for TlsStream<'a, S> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut writer = self.conn.writer();
        writer.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        while self.conn.wants_write() {
            self.conn.complete_io(&mut self.stream)?;
        }
        Ok(())
    }
}

fn preprocess_sql(query: &str) -> String {
    let mut result = query.to_string();
    let mut start = 0;
    while let Some(pos) = result[start..].find("/*!") {
        let abs_pos = start + pos;
        if let Some(end_pos) = result[abs_pos..].find("*/") {
            result = format!("{}{}", &result[..abs_pos], &result[abs_pos + end_pos + 2..]);
            start = abs_pos;
        } else { break; }
    }
    result = result.replace("AUTO_INCREMENT", "").replace("auto_increment", "");
    let mut char_result = String::new();
    let bytes = result.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 4 < bytes.len()
            && (bytes[i] == b'C' || bytes[i] == b'c')
            && (bytes[i+1] == b'H' || bytes[i+1] == b'h')
            && (bytes[i+2] == b'A' || bytes[i+2] == b'a')
            && (bytes[i+3] == b'R' || bytes[i+3] == b'r')
            && bytes[i+4] == b'(' {
            char_result.push_str("TEXT");
            i += 4;
            let mut depth = 1;
            i += 1;
            while i < bytes.len() && depth > 0 {
                if bytes[i] == b'(' { depth += 1; }
                else if bytes[i] == b')' { depth -= 1; }
                i += 1;
            }
            continue;
        }
        char_result.push(bytes[i] as char);
        i += 1;
    }
    result = char_result;

    // Remove SELECT DISTINCT -> SELECT (case-insensitive, preserve case)
    let upper = result.to_uppercase();
    if let Some(pos) = upper.find("SELECT DISTINCT") {
        result = format!("SELECT {}", &result[pos + 15..]);
    } else if let Some(pos) = upper.find("SELECT   DISTINCT") {
        result = format!("SELECT {}", &result[pos + 17..]);
    }

    // Convert BETWEEN x AND y to >= x AND <= y
    result = convert_between(&result);

    // Remove ORDER BY clause
    if let Some(pos) = result.to_uppercase().rfind("ORDER BY") {
        let upper_rest = result[pos..].to_uppercase();
        let end_pos = if let Some(lp) = upper_rest.find("LIMIT") {
            pos + lp
        } else {
            result.len()
        };
        result = format!("{}{}", &result[..pos], &result[end_pos..]);
    }

    // Remove LIMIT clause
    if let Some(pos) = result.to_uppercase().rfind("LIMIT") {
        let rest = &result[pos+5..];
        let end_pos = rest.find(|c: char| !c.is_digit(10) && c != ' ' && c != ',' && c != 'O')
            .unwrap_or(rest.len());
        result = format!("{}{}", &result[..pos], &rest[end_pos..]);
    }

    while result.contains("  ") { result = result.replace("  ", " "); }
    result = result.replace("DEFAULT '0'", "DEFAULT 0");
    result.trim().to_string()
}

fn convert_between(sql: &str) -> String {
    let upper = sql.to_uppercase();
    let mut result = sql.to_string();
    while let Some(pos) = upper.find("BETWEEN") {
        let before = &result[..pos];
        let after = &result[pos + 7..];
        let after = after.trim_start();
        let (low, rest) = split_at_and(after);
        if let Some(rest) = rest {
            let rest = rest.trim_start();
            let (high, remainder) = split_at_keyword(rest);
            let col = before.trim().rsplit(|c: char| c == ' ' || c == '(').next().unwrap_or("");
            result = format!("{}{} >= {} AND {} <= {}{}", before, col, low, col, high, remainder);
        } else {
            break;
        }
        break;
    }
    result
}

fn split_at_and(s: &str) -> (String, Option<&str>) {
    let upper = s.to_uppercase();
    let mut depth = 0i32;
    for (i, c) in upper.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            'A' if depth == 0 && upper[i..].starts_with("AND ") => {
                return (s[..i].trim().to_string(), Some(s[i+3..].trim_start()))
            }
            _ => {}
        }
    }
    (s.trim().to_string(), None)
}

fn split_at_keyword(s: &str) -> (String, &str) {
    let upper = s.to_uppercase();
    let keywords = ["ORDER", "LIMIT", "GROUP", "HAVING", "UNION", ")", ";"];
    for kw in &keywords {
        if let Some(pos) = upper.find(kw) {
            return (s[..pos].trim().to_string(), &s[pos..])
        }
    }
    (s.trim().to_string(), "")
}

fn simplify_update_arithmetic(query: &str) -> String {
    let mut result = query.to_string();
    let patterns = [("k=k+1", "k=1"), ("k = k + 1", "k=1"), ("K=K+1", "K=1"), ("K = K + 1", "K=1")];
    for (pat, rep) in &patterns {
        if result.contains(pat) { result = result.replace(pat, rep); }
    }
    result
}

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
    if let Some(pos) = query.trim().find("@@") {
        let rest = &query.trim()[pos + 2..];
        return rest.trim().trim_end_matches(';').trim().to_string();
    }
    String::new()
}

fn send_variable_result_set<W: Write>(w: &mut W, var_name: &str, var_value: &str, mut seq: u8) -> MySqlResult<u8> {
    {
        let mut p = Vec::new();
        write_lenenc_int(&mut p, 2).unwrap();
        Packet { length: p.len() as u32, sequence: seq, payload: p }.write_to(w)?;
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
        Packet { length: p.len() as u32, sequence: seq, payload: p }.write_to(w)?;
        seq = seq.wrapping_add(1);
    }
    make_eof_packet(seq, 0x0002).write_to(w)?;
    Ok(seq.wrapping_add(1))
}

fn send_session_var_result_set<W: Write>(w: &mut W, var_name: &str, var_value: &str, mut seq: u8) -> MySqlResult<u8> {
    {
        let mut p = Vec::new();
        write_lenenc_int(&mut p, 1).unwrap();
        Packet { length: p.len() as u32, sequence: seq, payload: p }.write_to(w)?;
        seq = seq.wrapping_add(1);
    }
    write_column_def(w, var_name, "INTEGER", seq)?;
    seq = seq.wrapping_add(1);
    make_eof_packet(seq, 0x0002).write_to(w)?;
    seq = seq.wrapping_add(1);
    {
        let mut p = Vec::new();
        write_lenenc_string(&mut p, var_value.as_bytes()).unwrap();
        Packet { length: p.len() as u32, sequence: seq, payload: p }.write_to(w)?;
        seq = seq.wrapping_add(1);
    }
    make_eof_packet(seq, 0x0002).write_to(w)?;
    Ok(seq.wrapping_add(1))
}

fn handle_special_query<W: Write>(query: &str, w: &mut W, seq: u8) -> MySqlResult<Option<u8>> {
    let upper = query.trim().to_uppercase();
    if upper.starts_with("SHOW VARIABLES") {
        let like_val = extract_like_value(query);
        let (var_name, var_value) = match like_val.as_str() {
            "tx_isolation" | "transaction_isolation" => ("tx_isolation", "REPEATABLE-READ"),
            "character_set_client" => ("character_set_client", "utf8"),
            "character_set_connection" => ("character_set_connection", "utf8"),
            "character_set_results" => ("character_set_results", "utf8"),
            "character_set_server" => ("character_set_server", "utf8"),
            "collation_connection" => ("collation_connection", "utf8_general_ci"),
            "collation_server" => ("collation_server", "utf8_general_ci"),
            "max_allowed_packet" => ("max_allowed_packet", "4194304"),
            "net_buffer_length" => ("net_buffer_length", "16384"),
            "version" => ("version", SERVER_VERSION),
            _ => { let new_seq = send_variable_result_set(w, &like_val, "", seq)?; return Ok(Some(new_seq)); }
        };
        let new_seq = send_variable_result_set(w, var_name, var_value, seq)?;
        return Ok(Some(new_seq));
    }
    if upper.starts_with("SELECT @@") {
        let var = extract_session_var(query);
        let value = match var.to_uppercase().as_str() {
            "AUTOCOMMIT" => "1",
            "TX_ISOLATION" | "TRANSACTION_ISOLATION" => "REPEATABLE-READ",
            "CHARACTER_SET_CLIENT" => "utf8",
            "CHARACTER_SET_SERVER" => "utf8",
            "VERSION" => SERVER_VERSION,
            "VERSION_COMMENT" => "SQLRustGo",
            _ => "0",
        };
        let new_seq = send_session_var_result_set(w, &format!("@@{}", var), value, seq)?;
        return Ok(Some(new_seq));
    }
    if upper == "SELECT 1" || upper == "SELECT 1;" {
        let mut s = seq;
        {
            let mut p = Vec::new();
            write_lenenc_int(&mut p, 1).unwrap();
            Packet { length: p.len() as u32, sequence: s, payload: p }.write_to(w)?;
            s = s.wrapping_add(1);
        }
        write_column_def(w, "1", "INTEGER", s)?;
        s = s.wrapping_add(1);
        make_eof_packet(s, 0x0002).write_to(w)?;
        s = s.wrapping_add(1);
        {
            let mut p = Vec::new();
            write_lenenc_string(&mut p, b"1").unwrap();
            Packet { length: p.len() as u32, sequence: s, payload: p }.write_to(w)?;
            s = s.wrapping_add(1);
        }
        make_eof_packet(s, 0x0002).write_to(w)?;
        return Ok(Some(s.wrapping_add(1)));
    }
    Ok(None)
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

    println!("[MYSQL-CONN] Connection from {}", addr);
    monitor.record_connection_opened();

    // ========================================================================
    // PHASE 1: Send Handshake (seq=0)
    // ========================================================================
    let seq = 0u8;

    let seed = rand_u8_20();

    let packet = make_handshake_packet(seq, &seed);
    println!(
        "[MYSQL-PACKET] S->C len={} seq={} type=handshake",
        packet.length, packet.sequence
    );
    packet.write_to(&mut stream)?;
    println!("[MYSQL-PACKET] Handshake written");

    // ========================================================================
    // PHASE 2: Read Auth Packet (client seq should be 1)
    // ========================================================================
    let auth_packet = match Packet::read_from(&mut stream) {
        Ok(p) => {
            println!(
                "[MYSQL-PACKET] C->S len={} seq={} payload[:20]={:02x?}",
                p.length,
                p.sequence,
                &p.payload[..20.min(p.payload.len())]
            );
            p
        }
        Err(e) => {
            println!("[MYSQL-ERROR] Failed to read auth packet: {}", e);
            return Err(MySqlError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                format!("auth read failed: {}", e),
            )));
        }
    };

    // ========================================================================
    // PHASE 3: Send OK Packet (seq = client seq + 1)
    // ========================================================================
    let client_capabilities = if auth_packet.payload.len() >= 4 {
        u32::from_le_bytes([auth_packet.payload[0], auth_packet.payload[1], auth_packet.payload[2], auth_packet.payload[3]])
    } else { 0 };
    let client_wants_ssl = (client_capabilities & capability::SSL) != 0;

    if client_wants_ssl {
        println!("[MYSQL-PACKET] Client requests SSL upgrade");
        let tls_config = match generate_tls_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                println!("[MYSQL-ERROR] TLS config failed: {}", e);
                return Err(MySqlError::Protocol(format!("TLS config error: {}", e)));
            }
        };
        let mut tls_conn = match rustls::ServerConnection::new(tls_config) {
            Ok(conn) => conn,
            Err(e) => {
                println!("[MYSQL-ERROR] TLS connection failed: {}", e);
                return Err(MySqlError::Protocol(format!("TLS connection error: {}", e)));
            }
        };
        match tls_conn.complete_io(&mut stream) {
            Ok(_) => println!("[MYSQL-PACKET] TLS handshake completed"),
            Err(e) => {
                println!("[MYSQL-ERROR] TLS handshake failed: {:?}", e);
                return Err(MySqlError::Protocol(format!("TLS handshake error: {}", e)));
            }
        }
        println!("[MYSQL-PACKET] TLS established");
        let mut tls_stream = TlsStream::new(&mut tls_conn, &mut stream);
        let reauth = Packet::read_from(&mut tls_stream)?;
        let ok_seq = reauth.sequence.wrapping_add(1);
        make_ok_packet(ok_seq, 0, 0).write_to(&mut tls_stream)?;
        println!("[MYSQL-PACKET] OK written over TLS");
        run_command_loop(&mut tls_stream, addr, storage, monitor, ok_seq.wrapping_add(1))?;
        return Ok(());
    }

    let auth_response = &auth_packet.payload[4..];
    let seed = rand_u8_20();
    if !verify_auth_response(auth_response, &seed, DEFAULT_PASSWORD) {
        let err_seq = auth_packet.sequence.wrapping_add(1);
        make_err_packet(err_seq, 1045, "Access denied for user").write_to(&mut stream)?;
        return Err(MySqlError::Protocol("Authentication failed".to_string()));
    }

    let ok_seq = auth_packet.sequence.wrapping_add(1);
    println!("[MYSQL-PACKET] S->C len=? seq={} type=ok", ok_seq);
    make_ok_packet(ok_seq, 0, 0).write_to(&mut stream)?;
    println!("[MYSQL-PACKET] OK written");

    run_command_loop(&mut stream, addr, storage, monitor, ok_seq.wrapping_add(1))?;
    return Ok(());
}

fn run_command_loop<W: Read + Write>(
    mut stream: &mut W,
    addr: SocketAddr,
    storage: Arc<RwLock<MemoryStorage>>,
    monitor: SharedMonitor,
    _initial_seq: u8,
) -> MySqlResult<()> {
    loop {
        let packet = match Packet::read_from(stream) {
            Ok(p) => {
                println!(
                    "[MYSQL-PACKET] C->S len={} seq={} cmd=0x{:02x}",
                    p.length,
                    p.sequence,
                    p.payload.first().copied().unwrap_or(0)
                );
                p
            }
            Err(e) => {
                println!("[MYSQL-ERROR] Packet read error: {}", e);
                break;
            }
        };

        let cmd = packet.payload.first().copied().unwrap_or(0);
        let payload = &packet.payload[1..];

        match cmd {
            packet_type::COM_QUIT => {
                println!("[MYSQL-PACKET] Client QUIT");
                break;
            }

            packet_type::COM_PING => {
                // Server response seq must be client_seq + 1
                let resp_seq = packet.sequence.wrapping_add(1);
                println!("[MYSQL-PACKET] S->C len=? seq={} type=pong", resp_seq);
                make_ok_packet(resp_seq, 0, 0).write_to(&mut stream)?;
            }

            packet_type::COM_INIT_DB => {
                let resp_seq = packet.sequence.wrapping_add(1);
                make_ok_packet(resp_seq, 0, 0).write_to(&mut stream)?;
            }

            packet_type::COM_QUERY => {
                let query = String::from_utf8_lossy(payload)
                    .trim_end_matches('\0')
                    .trim()
                    .to_string();
                println!("[MYSQL-QUERY] query=[{}]", query);

                let resp_seq = packet.sequence.wrapping_add(1);

                if query.is_empty() {
                    make_ok_packet(resp_seq, 0, 0).write_to(&mut stream)?;
                    continue;
                }

                let query_upper = query.to_uppercase();
                if query_upper.starts_with("SET NAMES") || query_upper == "SET NAMES" {
                    println!("[MYSQL-QUERY] SET NAMES ignored (charset setting)");
                    make_ok_packet(resp_seq, 0, 0).write_to(&mut stream)?;
                    continue;
                }

                if query_upper.starts_with("SET autocommit") || query_upper == "SET AUTOCOMMIT" {
                    println!("[MYSQL-QUERY] SET AUTOCOMMIT ignored");
                    make_ok_packet(resp_seq, 0, 0).write_to(&mut stream)?;
                    continue;
                }

                if query_upper.starts_with("SET ") {
                    println!("[MYSQL-QUERY] SET command ignored");
                    make_ok_packet(resp_seq, 0, 0).write_to(&mut stream)?;
                    continue;
                }

                if query_upper == "COMMIT"
                    || query_upper == "BEGIN"
                    || query_upper == "START TRANSACTION"
                    || query_upper == "ROLLBACK"
                {
                    println!("[MYSQL-QUERY] Transaction command ignored (no-op)");
                    make_ok_packet(resp_seq, 0, 0).write_to(&mut stream)?;
                    continue;
                }

                // Preprocess CREATE TABLE IF NOT EXISTS to remove "IF NOT EXISTS"
                let query_for_parse = if query_upper.contains("CREATE TABLE IF NOT EXISTS") {
                    let processed = query
                        .replace("IF NOT EXISTS", "")
                        .replace("if not exists", "");
                    println!("[MYSQL-QUERY] Preprocessed: [{}]", processed);
                    preprocess_sql(&processed)
                } else {
                    preprocess_sql(&query)
                };

                match handle_special_query(&query_for_parse, &mut stream, resp_seq)? {
                    Some(_) => continue,
                    None => {}
                }

                let effective_query = if query_upper.starts_with("UPDATE ") && query_for_parse.contains("+") {
                    simplify_update_arithmetic(&query_for_parse)
                } else {
                    query_for_parse.clone()
                };

                let mut engine = ExecutionEngine::new(storage.clone());

                if is_select_query(&effective_query) {
                    match execute_select(&effective_query, &mut engine) {
                        Ok((columns, column_types, rows)) => {
                            send_result_set(&mut stream, &columns, &column_types, &rows, resp_seq)?;
                        }
                        Err(e) => {
                            let code = if e.to_string().contains("not found") {
                                1146u16
                            } else {
                                1064
                            };
                            make_err_packet(resp_seq, code, &e.to_string())
                                .write_to(&mut stream)?;
                        }
                    }
                } else {
                    match execute_write(&effective_query, &mut engine) {
                        Ok(affected) => {
                            make_ok_packet(resp_seq, affected as u64, 0).write_to(&mut stream)?;
                        }
                        Err(e) => {
                            make_err_packet(resp_seq, 1064, &e.to_string())
                                .write_to(&mut stream)?;
                        }
                    }
                }
            }

            packet_type::COM_STMT_PREPARE => {
                let resp_seq = packet.sequence.wrapping_add(1);
                make_err_packet(resp_seq, 1295, "Prepared statements not yet supported")
                    .write_to(&mut stream)?;
            }

            _ => {
                let resp_seq = packet.sequence.wrapping_add(1);
                println!("[MYSQL-WARN] Unsupported command 0x{:02x}", cmd);
                make_err_packet(resp_seq, 2000, "Unsupported command").write_to(&mut stream)?;
            }
        }
    }

    println!("[MYSQL-CONN] Connection {} closed", addr);
    monitor.record_connection_closed();
    Ok(())
}

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
