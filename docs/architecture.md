# SQLRustGo 架构设计文档

## 概述

SQLRustGo 是一个用 Rust 实现的 SQL-92 数据库管理系统，采用分层架构设计。

## 系统架构图

```
┌─────────────────────────────────────┐
│           main.rs (REPL)             │
├─────────────────────────────────────┤
│           executor/                 │  ← 查询执行引擎
├─────────────────────────────────────┤
│           parser/                    │  ← SQL → AST
│           lexer/                    │  ← SQL → Tokens
├─────────────────────────────────────┤
│           storage/                   │  ← Page, BufferPool, B+ Tree
├─────────────────────────────────────┤
│         transaction/                 │  ← WAL, TxManager
├─────────────────────────────────────┤
│           network/                   │  ← TCP 服务器/客户端
├─────────────────────────────────────┤
│           types/                     │  ← Value, SqlError
└─────────────────────────────────────┘
```

## 核心模块

### 1. Lexer (词法分析器)

**What (是什么)**: Lexer 将原始 SQL 字符串分解为 Token 序列，是编译器的第一阶段。

**Why (为什么)**: Parser 需要结构化的 Token 而不是原始字符串，Lexer 负责这项转换工作。

**How (如何实现)**:
- 逐字符扫描输入
- 识别关键字、标识符、字面量、运算符
- 跳过空白字符
- 使用有限状态机处理不同 token 类型

### 2. Parser (解析器)

**What (是什么)**: Parser 将 Lexer 输出的 Token 序列转换为抽象语法树 (AST)。

**Why (为什么)**: Token 序列只是单词的列表，无法表达 SQL 语句的层级结构。AST 将单词组织成有意义的树结构，表示查询的语义。

**How (如何实现)**:
- 递归下降解析器：自顶向下处理 SQL 语句
- 每个 Statement 类型有对应的 parse_xxx 方法
- 支持：SELECT, INSERT, UPDATE, DELETE, CREATE TABLE, DROP TABLE
- 表达式解析支持基本二元运算

### 3. Executor (执行器)

**What (是什么)**: Executor 负责执行 Parser 生成的 AST，调用存储层完成数据操作。

**Why (为什么)**: AST 只是查询的抽象表示，需要执行器将其转化为具体的数据库操作。

**How (如何实现)**:
- 遍历 AST 节点
- 调用 Storage 层 API 完成数据读写
- 返回执行结果 (影响的行数、结果集等)

### 4. Storage (存储层)

**What (是什么)**: Storage 层负责数据的持久化和检索，包含 Page 管理、BufferPool 缓存和 B+ Tree 索引。

**Why (为什么)**: 数据库需要高效地存储和检索大量数据，Storage 层提供这一核心能力。

**How (如何实现)**:
- **Page**: 固定大小的数据块 (4096 字节)
- **BufferPool**: 内存缓存池，使用 LRU 策略
- **B+ Tree**: 磁盘友好的索引结构，支持高效范围查询
- **FileStorage**: 基于文件的数据持久化

### 5. Transaction (事务管理)

**What (是什么)**: Transaction 模块提供 ACID 事务支持，通过 Write-Ahead Log (WAL) 实现。

**Why (为什么)**: 事务是数据库的核心特性，保证数据一致性。

**How (如何实现)**:
- **WAL**: 写前日志，所有修改先记录日志
- **TxManager**: 事务状态管理，支持 BEGIN/COMMIT/ROLLBACK
- 崩溃恢复时重放 WAL 日志

### 6. Network (网络层)

**What (是什么)**: 网络通信模块，支持客户端-服务器架构。

**Why (为什么)**: 单一进程数据库只能单机使用，网络支持让数据库可以服务多个客户端。

**How (如何实现)**:
- TCP 服务器监听连接
- MySQL 协议兼容 (部分)
- 每个连接一个 Handler 处理

### 7. Types (类型系统)

**What (是什么)**: Types 模块定义 SQL 数据类型的运行时表示。

**Why (为什么)**: SQL 有多种数据类型，需要统一的内部表示。

**How (如何实现)**:
- Value 枚举: NULL, Boolean, Integer, Float, Text, Blob
- SqlError 枚举: 统一的错误类型
- 支持类型转换和序列化

## SQL 执行数据流

```
用户输入 SQL
     ↓
Lexer: "SELECT * FROM users" → [SELECT, STAR, FROM, IDENTIFIER(users)]
     ↓
Parser: Tokens → Statement::Select { table: "users", columns: [*], where_clause: None }
     ↓
Executor: 根据 Statement 类型调用 Storage 层
     ↓
Storage: 读取数据、构建结果
     ↓
返回 ExecutionResult { rows: [...], columns: [...], rows_affected: n }
```

## 错误处理机制

- **ParseError**: SQL 语法错误
- **ExecutionError**: 查询执行错误
- **TypeMismatch**: 类型不匹配
- **TableNotFound**: 表不存在
- **ColumnNotFound**: 列不存在
- **IoError**: I/O 错误

所有错误都实现 `std::error::Error` trait，支持 `Display` 输出。

## 扩展点

1. **新增 SQL 语句**: 在 Parser 添加新的 `parse_xxx` 方法，在 Executor 添加对应处理
2. **新增数据类型**: 在 Types 模块添加新的 Value 变体
3. **新存储引擎**: 实现 Storage trait
4. **新网络协议**: 在 Network 模块添加协议处理

## 测试覆盖

| 模块 | 行覆盖率 | 函数覆盖率 |
|------|---------|-----------|
| lexer/token.rs | 100% | 100% |
| types/error.rs | 100% | 100% |
| storage/file_storage.rs | 98.94% | 95.65% |
| storage/buffer_pool.rs | 97.96% | 100% |
| transaction/manager.rs | 93.69% | 80% |
| executor/mod.rs | 86.14% | 92.31% |
| parser/mod.rs | 78.74% | 96.15% |
| **总体** | **82.24%** | **84.73%** |
