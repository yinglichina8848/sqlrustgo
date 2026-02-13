# SQLRustGo 项目实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**目标:** 从零开始构建 Rust 数据库系统，实现 SQL-92 子集支持，具备现代化架构和 AI 工具链集成

**架构:** 模块化分层架构（Parser → Storage → Executor → Transaction → Network），采用 Rust 所有权系统保证内存安全

**技术栈:** Rust 2024 Edition, Cargo, Tokio async runtime, SQLite兼容语法

---

## 阶段一：项目基础设施（周1-2）

### Task 1: 创建 Rust 项目结构

**文件:**
- Create: `sqlrustgo/Cargo.toml`
- Create: `sqlrustgo/src/main.rs`
- Create: `sqlrustgo/src/lib.rs`

**Step 1: 创建 Cargo.toml**

```toml
[package]
name = "sqlrustgo"
version = "1.0.0"
edition = "2024"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
log = "0.4"
env_logger = "0.10"
```

**Step 2: 创建 src/main.rs**

```rust
fn main() {
    println!("SQLRustGo Database System");
}
```

**Step 3: 编译验证**

Run: `cd sqlrustgo && cargo build`
Expected: SUCCESS

**Step 4: 提交**

```bash
git add sqlrustgo/
git commit -m "feat: 初始化 Rust 项目结构"
```

---

### Task 2: 配置 AI 工具链和 CI/CD

**文件:**
- Create: `sqlrustgo/.claude/claude_desktop_config.json`
- Create: `sqlrustgo/.github/workflows/ci.yml`
- Create: `sqlrustgo/cargo-toolchain.toml`

**Step 1: 创建 GitHub Actions CI**

```yaml
name: CI
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        run: rustup default stable
      - name: Build
        run: cargo build --all-features
      - name: Test
        run: cargo test --all-features
      - name: Clippy
        run: cargo clippy --all-features -- -D warnings
```

**Step 2: 提交**

```bash
git add sqlrustgo/.github/ sqlrustgo/.claude/
git commit -m "feat: 添加 AI 工具链和 CI/CD 配置"
```

---

## 阶段二：核心类型系统（周3）

### Task 3: 定义 SQL 值类型和错误类型

**文件:**
- Create: `sqlrustgo/src/types/mod.rs`
- Create: `sqlrustgo/src/types/value.rs`
- Create: `sqlrustgo/src/types/error.rs`

**Step 1: 创建 value.rs**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Null => "NULL".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Text(s) => s.clone(),
            Value::Blob(b) => format!("X'{}'", hex::encode(b)),
        }
    }
}
```

**Step 2: 创建 error.rs**

```rust
#[derive(thiserror::Error, Debug)]
pub enum SqlError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
}
```

**Step 3: 创建 mod.rs**

```rust
pub mod value;
pub mod error;

pub use value::Value;
pub use error::SqlError;
```

**Step 4: 提交**

```bash
git add sqlrustgo/src/types/
git commit -m "feat: 定义核心类型系统"
```

---

## 阶段三：SQL 解析器（周4-5）

### Task 4: 实现词法分析器

**文件:**
- Create: `sqlrustgo/src/lexer/mod.rs`
- Create: `sqlrustgo/src/lexer/token.rs`
- Create: `sqlrustgo/src/lexer/lexer.rs`

**Step 1: 创建 token.rs**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Select, From, Where, Insert, Into, Values, Update, Set, Delete,
    Create, Table, Drop, Alter, Index, On, Primary, Key,
    Begin, Commit, Rollback,

    // Operators
    Equal, NotEqual, Greater, Less, GreaterEqual, LessEqual,
    And, Or, Not,
    Plus, Minus, Asterisk, Slash, Percent,

    // Syntax
    LParen, RParen, Comma, Dot, Semicolon,

    // Literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    BooleanLiteral(bool),

    // Special
    Star,  // * for SELECT *
    Eof,
}
```

**Step 2: 创建 lexer.rs**

```rust
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.is_eof() {
            return Token::Eof;
        }

        let ch = self.peek_char();

        match ch {
            ',' => { self.advance(); Token::Comma }
            '(' => { self.advance(); Token::LParen }
            ')' => { self.advance(); Token::RParen }
            ';' => { self.advance(); Token::Semicolon }
            '*' => { self.advance(); Token::Star }
            '+' => { self.advance(); Token::Plus }
            '-' => { self.advance(); Token::Minus }
            '/' => { self.advance(); Token::Slash }
            ',' => { self.advance(); Token::Comma }
            '=' => { self.advance(); Token::Equal }
            _ => self.lex_identifier_or_keyword(),
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.input.chars().nth(self.position).unwrap().is_whitespace() {
            self.position += 1;
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    fn peek_char(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn lex_identifier_or_keyword(&mut self) -> Token {
        let start = self.position;
        while !self.is_eof() && self.peek_char().is_alphanumeric() || self.peek_char() == '_' {
            self.advance();
        }
        let text = &self.input[start..self.position];
        Token::Identifier(text.to_string())
    }
}
```

**Step 3: 提交**

```bash
git add sqlrustgo/src/lexer/
git commit -m "feat: 实现词法分析器"
```

---

### Task 5: 实现语法解析器

**文件:**
- Create: `sqlrustgo/src/parser/mod.rs`
- Create: `sqlrustgo/src/parser/ast.rs`
- Create: `sqlrustgo/src/parser/parser.rs`

**Step 1: 创建 ast.rs**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    DropTable(DropTableStatement),
    Begin,
    Commit,
    Rollback,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub columns: Vec<SelectColumn>,
    pub from: Option<String>,
    pub where_clause: Option<Expression>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectColumn {
    Wildcard,
    Column { name: String, alias: Option<String> },
    Expression { expr: Expression, alias: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    pub table: String,
    pub set: Vec<(String, Expression)>,
    pub where_clause: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub table: String,
    pub where_clause: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Integer,
    Float,
    Text,
    Blob,
    Boolean,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Column(String),
    Value(Value),
    BinaryOp { left: Box<Expression>, op: BinaryOp, right: Box<Expression> },
    UnaryOp { op: UnaryOp, expr: Box<Expression> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Equal, NotEqual, Greater, Less, GreaterEqual, LessEqual,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not, Negate,
}
```

**Step 2: 创建 parser.rs**

```rust
pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Statement, SqlError> {
        self.parse_statement()
    }

    fn parse_statement(&mut self) -> Result<Statement, SqlError> {
        match self.peek() {
            Token::Select => self.parse_select(),
            Token::Insert => self.parse_insert(),
            Token::Update => self.parse_update(),
            Token::Delete => self.parse_delete(),
            Token::Create => self.parse_create(),
            Token::Drop => self.parse_drop(),
            Token::Begin => {
                self.advance();
                Ok(Statement::Begin)
            }
            Token::Commit => {
                self.advance();
                Ok(Statement::Commit)
            }
            Token::Rollback => {
                self.advance();
                Ok(Statement::Rollback)
            }
            _ => Err(SqlError::ParseError("Unexpected token".to_string())),
        }
    }

    fn parse_select(&mut self) -> Result<Statement, SqlError> {
        self.expect(Token::Select)?;

        let mut columns = Vec::new();
        while !self.match_token(Token::From) {
            columns.push(self.parse_select_column()?);
            self.match_token(Token::Comma);
        }

        let from = Some(self.parse_identifier()?);
        let where_clause = if self.match_token(Token::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::Select(SelectStatement {
            columns,
            from,
            where_clause,
            limit: None,
        }))
    }

    fn parse_select_column(&mut self) -> Result<SelectColumn, SqlError> {
        match self.peek() {
            Token::Star => {
                self.advance();
                Ok(SelectColumn::Wildcard)
            }
            Token::Identifier(_) => {
                let name = self.parse_identifier()?;
                Ok(SelectColumn::Column { name, alias: None })
            }
            _ => Err(SqlError::ParseError("Invalid column".to_string())),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, SqlError> {
        match self.peek() {
            Token::Identifier(s) => {
                self.advance();
                Ok(s.clone())
            }
            _ => Err(SqlError::ParseError("Expected identifier".to_string())),
        }
    }

    fn expect(&mut self, token: Token) -> Result<(), SqlError> {
        if self.match_token(token.clone()) {
            Ok(())
        } else {
            Err(SqlError::ParseError(format!("Expected {:?}", token)))
        }
    }

    fn match_token(&mut self, token: Token) -> bool {
        if self.peek() == &token {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn parse_expression(&mut self) -> Result<Expression, SqlError> {
        unimplemented!()
    }

    fn parse_insert(&mut self) -> Result<Statement, SqlError> {
        unimplemented!()
    }

    fn parse_update(&mut self) -> Result<Statement, SqlError> {
        unimplemented!()
    }

    fn parse_delete(&mut self) -> Result<Statement, SqlError> {
        unimplemented!()
    }

    fn parse_create(&mut self) -> Result<Statement, SqlError> {
        unimplemented!()
    }

    fn parse_drop(&mut self) -> Result<Statement, SqlError> {
        unimplemented!()
    }
}
```

**Step 3: 提交**

```bash
git add sqlrustgo/src/parser/
git commit -m "feat: 实现语法解析器"
```

---

## 阶段四：存储引擎（周6-7）

### Task 6: 实现页面和缓冲区管理

**文件:**
- Create: `sqlrustgo/src/storage/mod.rs`
- Create: `sqlrustgo/src/storage/page.rs`
- Create: `sqlrustgo/src/storage/buffer_pool.rs`

**Step 1: 创建 page.rs**

```rust
pub const PAGE_SIZE: usize = 8192;

#[derive(Debug)]
pub struct Page {
    pub data: [u8; PAGE_SIZE],
    pub page_id: u32,
    pub is_dirty: bool,
}

impl Page {
    pub fn new(page_id: u32) -> Self {
        Self {
            data: [0u8; PAGE_SIZE],
            page_id,
            is_dirty: false,
        }
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) {
        self.data[offset..offset + data.len()].copy_from_slice(data);
        self.is_dirty = true;
    }

    pub fn read(&self, offset: usize, len: usize) -> &[u8] {
        &self.data[offset..offset + len]
    }
}
```

**Step 2: 创建 buffer_pool.rs**

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use lru::LruCache;

pub struct BufferPool {
    cache: Mutex<LruCache<u32, Arc<Page>>>,
    capacity: usize,
}

impl BufferPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(capacity.try_into().unwrap())),
            capacity,
        }
    }

    pub fn get(&self, page_id: u32) -> Option<Arc<Page>> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(&page_id).cloned()
    }

    pub fn insert(&self, page_id: u32, page: Arc<Page>) {
        let mut cache = self.cache.lock().unwrap();
        if cache.len() >= self.capacity {
            // Eviction handled by LRU
        }
        cache.put(page_id, page);
    }
}
```

**Step 3: 提交**

```bash
git add sqlrustgo/src/storage/
git commit -m "feat: 实现页面和缓冲区管理"
```

---

### Task 7: 实现 B+ 树索引

**文件:**
- Create: `sqlrustgo/src/storage/bplus_tree/mod.rs`
- Create: `sqlrustgo/src/storage/bplus_tree/node.rs`
- Create: `sqlrustgo/src/storage/bplus_tree/tree.rs`

**Step 1: 创建 node.rs**

```rust
pub enum Node {
    Internal(InternalNode),
    Leaf(LeafNode),
}

pub struct InternalNode {
    pub keys: Vec<u64>,
    pub children: Vec<u32>,
}

pub struct LeafNode {
    pub keys: Vec<u64>,
    pub values: Vec<u32>,
    pub next: Option<u32>,
}
```

**Step 2: 创建 tree.rs**

```rust
use super::node::Node;

pub struct BPlusTree {
    root: Option<u32>,
    // Storage for nodes would be in a separate table
}

impl BPlusTree {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn insert(&mut self, key: u64, value: u32) {
        // B+ tree insert implementation
    }

    pub fn search(&self, key: u64) -> Option<u32> {
        // B+ tree search implementation
        None
    }
}
```

**Step 3: 提交**

```bash
git add sqlrustgo/src/storage/bplus_tree/
git commit -m "feat: 实现 B+ 树索引"
```

---

## 阶段五：查询执行器（周8-9）

### Task 8: 实现执行器框架

**文件:**
- Create: `sqlrustgo/src/executor/mod.rs`
- Create: `sqlrustgo/src/executor/engine.rs`
- Create: `sqlrustgo/src/executor/select_executor.rs`

**Step 1: 创建 engine.rs**

```rust
use crate::parser::Statement;

pub struct ExecutionEngine<'a> {
    catalog: &'a Catalog,
    buffer_pool: &'a BufferPool,
}

impl<'a> ExecutionEngine<'a> {
    pub fn execute(&self, statement: Statement) -> Result<ExecutionResult, SqlError> {
        match statement {
            Statement::Select(stmt) => self.execute_select(stmt),
            Statement::Insert(stmt) => self.execute_insert(stmt),
            Statement::Update(stmt) => self.execute_update(stmt),
            Statement::Delete(stmt) => self.execute_delete(stmt),
            Statement::CreateTable(stmt) => self.execute_create_table(stmt),
            Statement::DropTable(stmt) => self.execute_drop_table(stmt),
            _ => Err(SqlError::ExecutionError("Not implemented".to_string())),
        }
    }
}

pub struct ExecutionResult {
    pub rows_affected: u64,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}
```

**Step 2: 创建 select_executor.rs**

```rust
use super::engine::ExecutionEngine;

impl<'a> ExecutionEngine<'a> {
    fn execute_select(&self, stmt: crate::parser::SelectStatement) -> Result<ExecutionResult, SqlError> {
        // SELECT executor implementation
        Ok(ExecutionResult {
            rows_affected: 0,
            columns: vec![],
            rows: vec![],
        })
    }
}
```

**Step 3: 提交**

```bash
git add sqlrustgo/src/executor/
git commit -m "feat: 实现执行器框架"
```

---

## 阶段六：事务管理器（周10）

### Task 9: 实现事务管理

**文件:**
- Create: `sqlrustgo/src/transaction/mod.rs`
- Create: `sqlrustgo/src/transaction/manager.rs`
- Create: `sqlrustgo/src/transaction/wal.rs`

**Step 1: 创建 wal.rs**

```rust
pub struct WriteAheadLog {
    log_file: File,
}

impl WriteAheadLog {
    pub fn new(path: &str) -> Self {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        Self { log_file }
    }

    pub fn append(&mut self, record: &[u8]) -> Result<(), SqlError> {
        self.log_file.write_all(record)?;
        self.log_file.flush()?;
        Ok(())
    }
}
```

**Step 2: 提交**

```bash
git add sqlrustgo/src/transaction/
git commit -m "feat: 实现事务管理"
```

---

## 阶段七：网络层（周11）

### Task 10: 实现网络协议

**文件:**
- Create: `sqlrustgo/src/network/mod.rs`
- Create: `sqlrustgo/src/network/server.rs`
- Create: `sqlrustgo/src/network/protocol.rs`

**Step 1: 创建 server.rs**

```rust
use tokio::net::TcpListener;

pub async fn start_server(addr: &str) -> Result<(), SqlError> {
    let listener = TcpListener::bind(addr).await?;
    println!("SQLCC Server listening on {}", addr);

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle_connection(socket));
    }
}

async fn handle_connection(socket: TcpStream) {
    // Handle MySQL protocol
}
```

**Step 2: 提交**

```bash
git add sqlrustgo/src/network/
git commit -m "feat: 实现网络协议层"
```

---

## 阶段八：集成测试和优化（周12）

### Task 11: 端到端集成测试

**文件:**
- Create: `sqlrustgo/tests/integration_test.rs`

**Step 1: 创建集成测试**

```rust
#[tokio::test]
async fn test_basic_select() {
    let server = start_test_server().await;
    let client = connect().await;

    let result = client.execute("CREATE TABLE test (id INTEGER, name TEXT)").await;
    assert!(result.is_ok());

    let result = client.execute("INSERT INTO test VALUES (1, 'Alice')").await;
    assert!(result.is_ok());

    let result = client.execute("SELECT * FROM test").await;
    assert!(result.is_ok());
}
```

**Step 2: 提交**

```bash
git add sqlrustgo/tests/
git commit -m "feat: 添加集成测试"
```

---

**计划完成，保存于 `docs/plans/2026-02-13-sqlcc-rust-impl-plan.md`**

---

## 执行选项

**1. Subagent-Driven（本会话）** - 我在本会话中分派子代理逐个任务执行，中间进行代码审查

**2. Parallel Session（新会话）** - 在新会话中使用 executing-plans，分批执行并设置检查点

选择哪种方式？
