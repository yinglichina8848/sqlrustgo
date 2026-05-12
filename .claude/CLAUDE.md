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

## Gitea 工作流（关键）

### Gitea API Token
```
04bcda86dd601364a53eec33dc37aa6efa98a5b7
```

### 创建 PR（API）
```bash
TOKEN=04bcda86dd601364a53eec33dc37aa6efa98a5b7
curl -X POST http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/pulls \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "PR 标题",
    "head": "feature/my-branch",
    "base": "develop/v3.1.0",
    "body": "PR 描述"
  }'
```

### 创建 Issue（API）
```bash
TOKEN=04bcda86dd601364a53eec33dc37aa6efa98a5b7
curl -X POST http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/issues \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"标题","body":"描述"}'
```

### 关键注意事项
- Gitea 只有 HTTP，没有 HTTPS
- head 分支必须先 push 到 Gitea 才能创建 PR
- 使用 http:// 而非 https://
