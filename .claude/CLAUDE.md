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

## PR 审核检查清单

### 1. 检查目标分支
- 确认 PR 目标分支是否为 `develop/v1.2.0`
- **禁止** 合并到 `main` (v1.2.0 尚未达到 RC/GA)

### 2. 本地验证流程
```bash
# 切换到 develop/v1.2.0 分支
git fetch origin develop/v1.2.0
git checkout origin/develop/v1.2.0

# 拉取 PR 分支并验证
gh pr checkout <PR号>
cargo build
cargo test
cargo clippy -- -D warnings
```

### 3. 覆盖率验证
```bash
# 必须先确保测试编译通过
cargo test --workspace --all-targets

# 然后运行覆盖率
cargo tarpaulin
```

**重要**：
- 测试必须能编译通过才能验证覆盖率
- 覆盖率数据必须基于能编译运行的测试
- PR 声称的覆盖率必须本地验证

### 4. 合并冲突解决
- 如果有冲突，使用 `git merge origin/develop/v1.2.0 -X theirs` 解决
- 解决后 push 并合并 PR

### 5. Issue 关联
- 检查 PR 关联的 Issue 状态
- 合并后关闭对应的 Issue
