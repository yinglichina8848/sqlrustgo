# SQLRustGo 测试补全与文档完善计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 SQLRustGo 数据库项目补充测试至 80% 覆盖率，并添加充分的 What-Why-How 注释，帮助大二学生理解如何从零开始设计开发数据库软件。

**Architecture:** 采用模块化测试策略，先分析每个模块的测试覆盖率，再针对性补充。使用 TDD 流程，先写测试再验证。

**Tech Stack:** Rust, cargo test, cargo-llvm-cov (覆盖率工具)

---

## 阶段 1: 环境准备与覆盖率分析

### Task 1.1: 安装覆盖率工具

**Files:**
- Modify: `Cargo.toml`

**Step 1: 添加 llvm-cov 依赖**

```bash
cargo install cargo-llvm-cov
```

**Step 2: 运行覆盖率检查**

```bash
cargo llvm-cov --all-features --text
```

预期输出：显示各模块的覆盖率百分比

---

## 阶段 2: Types 模块 (已有测试，需要完善注释)

### Task 2.1: 完善 types/error.rs 注释

**Files:**
- Modify: `src/types/error.rs`

**Step 1: 添加 What-Why-How 注释**

在文件顶部添加模块级注释：

```rust
//! Types Module - Error Handling
//!
//! # What (是什么)
//! 本模块定义了 SQLRustGo 数据库的错误类型系统，包括 SqlError 枚举和 SqlResult 类型。
//!
//! # Why (为什么)
//! 统一的错误处理是数据库系统的核心需求。通过定义标准化的错误类型，
//! 可以让调用者清楚地知道发生了什么错误，并采取相应的恢复措施。
//!
//! # How (如何实现)
//! - SqlError: 使用枚举变体表示不同类型的错误（解析错误、执行错误、存储错误等）
//! - SqlResult: 是 Result 类型的别名，简化错误传播
//! - SqlError 实现 Display trait 以便人类可读的错误信息
```

**Step 2: 运行测试验证**

```bash
cargo test --lib types
```

---

### Task 2.2: 完善 types/value.rs 注释

**Files:**
- Modify: `src/types/value.rs`

**Step 1: 添加 What-Why-How 注释**

在 Value 枚举和关键方法处添加注释：

```rust
/// SQL Value enum representing all supported SQL data types
///
/// # What (是什么)
/// Value 是 SQL 数据的运行时表示，支持 NULL、布尔、整数、浮点、文本和二进制类型
///
/// # Why (为什么)
/// SQL 标准定义了多种数据类型，数据库需要在内部统一表示这些类型
///
/// # How (如何实现)
/// - 使用 Rust 枚举表示 SQL 类型，每种变体对应一种 SQL 类型
/// - 实现 Display trait 用于 SQL 输出格式
/// - 使用 serde 进行序列化/反序列化（支持持久化）
```

---

## 阶段 3: Lexer 模块 (需要补充测试和注释)

### Task 3.1: 补充 lexer/token.rs 测试

**Files:**
- Modify: `src/lexer/token.rs`

**Step 1: 检查现有测试覆盖率**

```bash
cargo test --lib lexer
```

**Step 2: 添加测试用例**

添加以下测试：

```rust
#[test]
fn test_token_display() {
    assert_eq!(Token::Select.to_string(), "SELECT");
    assert_eq!(Token::Integer.to_string(), "INTEGER");
}

#[test]
fn test_token_from_keyword() {
    assert_eq!(Token::from_keyword("SELECT"), Some(Token::Select));
    assert_eq!(Token::from_keyword("INSERT"), Some(Token::Insert));
    assert_eq!(Token::from_keyword("UNKNOWN"), None);
}
```

**Step 3: 提交**

```bash
git add src/lexer/token.rs
git commit -m "test: add token tests"
```

---

### Task 3.2: 补充 lexer/lexer.rs 测试

**Files:**
- Modify: `src/lexer/lexer.rs`

**Step 1: 添加词法分析测试**

```rust
#[test]
fn test_lexer_string_literal() {
    let tokens = Lexer::new("'hello world'").tokenize();
    assert!(matches!(tokens[0], Token::StringLiteral(s) if s == "hello world"));
}

#[test]
fn test_lexer_operators() {
    let tokens = Lexer::new("<> <= >=").tokenize();
    assert!(matches!(tokens[0], Token::NotEqual));
    assert!(matches!(tokens[1], Token::LessEqual));
    assert!(matches!(tokens[2], Token::GreaterEqual));
}

#[test]
fn test_lexer_multiple_statements() {
    let tokens = Lexer::new("SELECT 1; SELECT 2").tokenize();
    assert!(matches!(tokens.last(), Some(Token::Semicolon)));
}
```

**Step 2: 添加 What-Why-How 注释**

在 Lexer 结构体处添加：

```rust
/// SQL Lexer - Tokenizer
///
/// # What (是什么)
/// Lexer 将原始 SQL 字符串分解为 Token 序列，是编译器的第一阶段
///
/// # Why (为什么)
/// Parser 需要结构化的 Token 而不是原始字符串，Lexer 负责这项转换工作
///
/// # How (如何实现)
/// - 逐字符扫描输入
/// - 识别关键字、标识符、字面量、运算符
/// - 跳过空白字符
/// - 使用有限状态机处理不同 token 类型
```

---

## 阶段 4: Parser 模块 (已有测试，完善注释)

### Task 4.1: 完善 parser/mod.rs 注释

**Files:**
- Modify: `src/parser/mod.rs`

**Step 1: 添加模块级 What-Why-How 注释**

```rust
//! SQL Parser Module
//!
//! # What (是什么)
//! Parser 将 Lexer 输出的 Token 序列转换为抽象语法树 (AST)
//!
//! # Why (为什么)
//! Token 序列只是单词的列表，无法表达 SQL 语句的层级结构
//! AST 将单词组织成有意义的树结构，表示查询的语义
//!
//! # How (如何实现)
//! - 递归下降解析器：自顶向下处理 SQL 语句
//! - 每个 Statement 类型有对应的 parse_xxx 方法
//! - 支持：SELECT, INSERT, UPDATE, DELETE, CREATE TABLE, DROP TABLE
//! - 表达式解析支持基本二元运算
```

---

## 阶段 5: Storage 模块 (需要大量补充)

### Task 5.1: 补充 storage/page.rs 测试

**Files:**
- Modify: `src/storage/page.rs`

**Step 1: 添加更多测试**

```rust
#[test]
fn test_page_data_access() {
    let mut page = Page::new(1);
    // Write some data
    page.data[0] = 0xAB;
    page.data[1] = 0xCD;
    // Read it back
    assert_eq!(page.data[0], 0xAB);
    assert_eq!(page.data[1], 0xCD);
}

#[test]
fn test_page_default_size() {
    assert_eq!(Page::size(), 4096);
}
```

---

### Task 5.2: 补充 storage/buffer_pool.rs 测试

**Files:**
- Create: `src/storage/buffer_pool.rs` (如果不存在)
- Test: `src/storage/buffer_pool.rs`

**Step 1: 添加 BufferPool 测试**

```rust
#[test]
fn test_buffer_pool_basic() {
    let pool = BufferPool::new(10);
    assert_eq!(pool.capacity(), 10);
}

#[test]
fn test_buffer_pool_get_page() {
    let mut pool = BufferPool::new(10);
    let page = Page::new(1);
    pool.put_page(1, page);
    let retrieved = pool.get_page(1);
    assert!(retrieved.is_some());
}
```

---

### Task 5.3: 补充 storage/file_storage.rs 测试

**Files:**
- Modify: `src/storage/file_storage.rs`

**Step 1: 添加文件存储测试**

```rust
#[test]
fn test_file_storage_crud() {
    let temp_dir = std::env::temp_dir().join("test_fs");
    let storage = FileStorage::new(temp_dir.clone()).unwrap();

    // Test table operations
    let table = TableData { /* ... */ };
    storage.insert_table("test".to_string(), table).unwrap();
    assert!(storage.contains_table("test"));

    storage.drop_table("test").unwrap();
    assert!(!storage.contains_table("test"));

    // Cleanup
    let _ = std::fs::remove_dir_all(temp_dir);
}
```

---

### Task 5.4: 完善 storage/bplus_tree/tree.rs 注释

**Files:**
- Modify: `src/storage/bplus_tree/tree.rs`

**Step 1: 添加 B+ Tree 核心概念注释**

```rust
/// B+ Tree index
///
/// # What (是什么)
/// B+ Tree 是一种自平衡的多叉树数据结构，保持数据有序，支持高效的对数时间复杂度操作
///
/// # Why (为什么)
/// - 磁盘友好：B+ Tree 的扇出因子大，树高小，磁盘 IO 次数少
/// - 范围查询快：叶子节点链表连接，支持高效范围扫描
/// - 平衡保证：所有叶子节点深度一致，操作时间复杂度稳定
///
/// # How (如何实现)
/// - 内部节点仅存储键（用于路由），叶子节点存储键值对
/// - 每个节点有最大和最小键数限制（MAX_KEYS, MIN_KEYS）
/// - 节点满时分裂，保持平衡
/// - 使用 Node 枚举表示内部节点和叶子节点
```

---

## 阶段 6: Executor 模块 (已有测试，完善注释)

### Task 6.1: 完善 executor/mod.rs 注释

**Files:**
- Modify: `src/executor/mod.rs`

**Step 1: 添加查询执行器注释**

```rust
//! Query Execution Engine
//!
//! # What (是什么)
//! Executor 负责执行 Parser 生成的 AST，产生查询结果
//!
//! # Why (为什么)
//! Parser 解析 SQL 只是理解"要做什么"，Executor 才是真正"执行操作"的组件
//! Executor 需要访问存储层，按需读写数据
//!
//! # How (如何实现)
//! - Volcano 模型：迭代器风格，每步处理一个算子
//! - 算子下推：将过滤等操作下推到存储层
//! - 支持：DML (INSERT/UPDATE/DELETE) 和 DQL (SELECT)
//! - 表达式求值：WHERE 子句的布尔表达式
```

---

## 阶段 7: Transaction 模块 (需要补充测试和注释)

### Task 7.1: 补充 transaction/wal.rs 测试

**Files:**
- Modify: `src/transaction/wal.rs`

**Step 1: 添加 WAL 测试**

```rust
#[test]
fn test_wal_basic_write() {
    let path = "/tmp/test_wal.log";
    std::fs::remove_file(path).ok();

    let wal = WriteAheadLog::new(path).unwrap();
    wal.write("BEGIN".as_bytes()).unwrap();
    wal.write("COMMIT".as_bytes()).unwrap();

    std::fs::remove_file(path).ok();
}
```

---

### Task 7.2: 补充 transaction/manager.rs 测试

**Files:**
- Modify: `src/transaction/manager.rs`

**Step 1: 添加事务管理器测试**

```rust
#[test]
fn test_transaction_manager() {
    let path = "/tmp/test_tx.log";
    std::fs::remove_file(path).ok();

    let wal = Arc::new(WriteAheadLog::new(path).unwrap());
    let tm = TransactionManager::new(wal);

    // Begin transaction
    let tx_id = tm.begin().unwrap();
    assert!(tm.is_active(tx_id));

    // Commit
    tm.commit(tx_id).unwrap();
    assert!(!tm.is_active(tx_id));

    std::fs::remove_file(path).ok();
}
```

---

### Task 7.3: 添加事务模块注释

**Files:**
- Modify: `src/transaction/mod.rs`

```rust
//! Transaction Module
//!
//! # What (是什么)
//! 事务管理模块，提供 ACID 特性保证
//!
//! # Why (为什么)
//! 数据库的核心价值之一是提供事务支持，确保数据一致性
//! 即使发生系统崩溃，也不会导致数据损坏
//!
//! # How (如何实现)
//! - WAL (Write-Ahead Log)：先写日志再写数据
//! - 事务状态机：Active -> Committed / Aborted
//! - Commit：持久化日志
//! - Rollback：逆向操作
```

---

## 阶段 8: Network 模块 (注释完善)

### Task 8.1: 完善 network 模块注释

**Files:**
- Modify: `src/network/mod.rs`

```rust
//! Network Module
//!
//! # What (是什么)
//! 网络通信模块，支持客户端-服务器架构
//!
//! # Why (为什么)
//! 单一进程数据库只能单机使用，网络支持让数据库可以服务多个客户端
//!
//! # How (如何实现)
//! - TCP 服务器监听连接
//! - MySQL 协议兼容（可选）
//! - 连接池管理并发
```

---

## 阶段 9: 集成测试

### Task 9.1: 补充集成测试

**Files:**
- Modify: `tests/integration_test.rs`

**Step 1: 添加完整工作流测试**

```rust
#[test]
fn test_full_workflow() {
    let mut engine = ExecutionEngine::new();

    // Create table
    engine.execute(parse("CREATE TABLE test (id INTEGER, name TEXT)")).unwrap();

    // Insert
    engine.execute(parse("INSERT INTO test VALUES (1, 'Alice')")).unwrap();

    // Select
    let result = engine.execute(parse("SELECT * FROM test")).unwrap();
    assert_eq!(result.rows.len(), 1);

    // Update
    engine.execute(parse("UPDATE test SET name = 'Bob' WHERE id = 1")).unwrap();

    // Delete
    engine.execute(parse("DELETE FROM test WHERE id = 1")).unwrap();

    // Drop
    engine.execute(parse("DROP TABLE test")).unwrap();
}
```

---

## 阶段 10: 最终验证

### Task 10.1: 运行完整测试套件

```bash
cargo test --all-features
```

### Task 10.2: 检查覆盖率

```bash
cargo llvm-cov --all-features --text
```

目标：总覆盖率 >= 80%

### Task 10.3: 运行 clippy 检查

```bash
cargo clippy --all-features -- -D warnings
```

### Task 10.4: 格式化检查

```bash
cargo fmt --check --all
```

### Task 10.5: 提交代码

```bash
git add .
git commit -m "test: 补充测试至 80% 覆盖率，添加 What-Why-How 注释"
```

---

## 预期输出

完成后，项目将具有：
- 80%+ 的测试覆盖率
- 每个核心模块都有 What-Why-How 注释
- 帮助大二学生理解数据库系统设计的教学注释
- 完整的 CI 检查通过

---

**Plan complete. Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
