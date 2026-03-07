# CLAUDE.md

此文件为 Claude Code (claude.ai/code) 提供使用此存储库中的代码时的指导。

## 项目概述

SQLRustGo 是支持 SQL-92 子集的关系数据库系统的 Rust 实现。采用现代分层架构从头开始构建。

## 常用命令

```bash
# Build
cargo build --all-features

# Run tests
cargo test --all-features

# Run a single test
cargo test test_name --all-features

# Lint with clippy
cargo clippy --all-features -- -D warnings

# Format check
cargo fmt --check --all

# Doc tests
cargo test --doc

# Run REPL
cargo run --bin sqlrustgo
```

＃＃ 建筑学

```
┌─────────────────────────────────────┐
│           main.rs (REPL)             │
├─────────────────────────────────────┤
│           executor/                 │  ← Query execution
├─────────────────────────────────────┤
│           parser/                    │  ← SQL → AST
│           lexer/                    │  ← SQL → Tokens
├─────────────────────────────────────┤
│           storage/                   │  ← Page, BufferPool, B+ Tree
├─────────────────────────────────────┤
│         transaction/                 │  ← WAL, TxManager
├─────────────────────────────────────┤
│           network/                   │  ← TCP server/client
├─────────────────────────────────────┤
│           types/                     │  ← Value, SqlError
└─────────────────────────────────────┘
```

## 关键模块

|模块|目的|
|--------|---------|
| `lexer` |对 SQL 输入进行标记|
|__代码0__|将 token 解析为语句 AST|
|__代码0__|页面管理、BufferPool (LRU)、B+ Tree 索引|
|__代码0__|执行 SQL 语句|
|__代码0__|预写日志，开始/提交/回滚|
|__代码0__|采用 MySQL 风格协议的 TCP 服务器/客户端|

## 铁锈版

将 Rust 版本 2024 与 Tokio 异步运行时结合使用。
