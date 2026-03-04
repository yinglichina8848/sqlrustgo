# SQLRustGo

Rust 实现的关系型数据库系统，支持 SQL-92 子集。

## 核心特性

- **SQL-92 支持**: 支持 SELECT, INSERT, UPDATE, DELETE, CREATE TABLE, DROP TABLE 等常用语句
- **事务支持**: ACID 事务，通过 Write-Ahead Log (WAL) 实现
- **存储引擎**: Page 管理 + BufferPool 缓存 + B+ Tree 索引
- **网络支持**: TCP 服务器，支持多客户端连接
- **REPL 交互**: 交互式命令行界面

## 快速开始

```bash
# 构建
cargo build --all-features

# 运行测试
cargo test --all-features

# 启动 REPL
cargo run --bin sqlrustgo

# Lint 检查
cargo clippy --all-features -- -D warnings
```

## 架构概览

```
┌─────────────────────────────────────┐
│           main.rs (REPL)             │
├─────────────────────────────────────┤
│           executor/                 │  ← 查询执行
├─────────────────────────────────────┤
│           parser/                    │  ← SQL → AST
│           lexer/                    │  ← SQL → Tokens
├─────────────────────────────────────┤
│           storage/                   │  ← Page, BufferPool, B+ Tree
├─────────────────────────────────────┤
│         transaction/                 │  ← WAL, TxManager
├─────────────────────────────────────┤
│           network/                   │  ← TCP Server/Client
├─────────────────────────────────────┤
│           types/                     │  ← Value, SqlError
└─────────────────────────────────────┘
```

## 项目结构

```
sqlrustgo/
├── Cargo.toml
├── README.md
├── .github/workflows/
├── docs/
│   └── architecture.md       ← 架构设计文档
├── src/
│   ├── main.rs              ← REPL 入口
│   ├── lib.rs               ← 库入口
│   ├── lexer/               ← 词法分析
│   ├── parser/              ← 语法分析
│   ├── executor/            ← 查询执行
│   ├── storage/             ← 存储层
│   ├── transaction/         ← 事务管理
│   ├── network/             ← 网络通信
│   └── types/               ← 类型系统
└── tests/                   ← 集成测试
```

## 功能特性

- ✅ SQL-92 子集支持 (SELECT, INSERT, UPDATE, DELETE)
- ✅ 存储引擎 (Buffer Pool, FileStorage)
- ✅ B+ Tree 索引
- ✅ 事务支持 (WAL)
- ✅ 网络协议支持

## 文档

- [架构设计](docs/architecture.md)
- [设计文档](docs/2026-02-13-sqlcc-rust-redesign-design.md)
- [实施计划](docs/2026-02-13-sqlcc-rust-impl-plan.md)

## 测试覆盖

- 测试数量: 118+ 个
- 行覆盖率: 82.24%
- 函数覆盖率: 84.73%

## 技术栈

- Rust Edition 2024
- Tokio 异步运行时
- thiserror 错误处理
- serde 序列化

## 参与贡献

欢迎提交 PR！请参考 GitHub 上的 Issue 和 PR 列表。
