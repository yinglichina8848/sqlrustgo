# SQLRustGo

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85+-dea584?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/version-v2.9.0--alpha-blue" alt="Version">
  <img src="https://img.shields.io/badge/branch-develop%2Fv2.9.0-green" alt="Branch">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

SQLRustGo 是一个使用 Rust 实现的关系型数据库教学与工程化项目，支持 SQL-92 子集，包含解析、规划、执行、存储、事务与网络模块。

## 当前状态

| 项目 | 当前值 |
|------|--------|
| 当前版本状态 | **v2.9.0 (Alpha)** |
| 当前主分支 | **develop/v2.9.0** |
| 当前阶段 | Alpha → Beta 过渡中 |
| 基准版本 | v2.8.0 (GA) |
| 版本目标 | 企业级韧性 (Enterprise Resilience) |

- 版本文件: [VERSION](VERSION)
- 当前版本说明: [docs/releases/v2.9.0/README.md](docs/releases/v2.9.0/README.md)
- v2.8.0 文档入口: [docs/releases/v2.8.0/README.md](docs/releases/v2.8.0/README.md)

## 核心能力

### SQL 支持
- **DDL**: CREATE/DROP TABLE, ALTER TABLE, CREATE/DROP INDEX, CREATE/DROP VIEW
- **DML**: SELECT, INSERT, UPDATE, DELETE, REPLACE
- **高级**: CTE/WITH, 递归 CTE, 窗口函数 (ROW_NUMBER, RANK, DENSE_RANK), CASE/WHEN, JSON 操作

### 存储引擎
- Buffer Pool + FileStorage + MemoryStorage + ColumnarStorage
- B+ Tree + Hash Index + Vector Index
- MVCC (Snapshot Isolation)
- WAL (Write-Ahead Logging)

### 分布式
- Semi-sync 复制
- MTS 并行复制 (Multi-Threaded Slave)
- Multi-source 复制
- XA 事务 (两阶段提交)

### 网络与安全
- TCP / MySQL 5.7 风格协议
- RBAC 权限管理 (GRANT/REVOKE)
- AES-256 静态加密
- 安全审计日志

### 交互
- REPL
- Prepared Statement
- 存储过程与触发器

## 快速开始

```bash
# 构建
cargo build --all-features

# 运行测试
cargo test --all-features

# 启动 REPL
cargo run --bin sqlrustgo

# 代码检查
cargo clippy --all-features -- -D warnings

# 格式化检查
cargo fmt --check --all
```

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                    SQLRustGo v2.9.0                          │
├─────────────────────────────────────────────────────────────┤
│  网络层 (network/)      │  MySQL 5.7 协议兼容                │
├─────────────────────────────────────────────────────────────┤
│  服务层 (server/)       │  TCP Server + 连接管理              │
├─────────────────────────────────────────────────────────────┤
│  查询处理               │                                    │
│  ┌─────────┬─────────┐ │                                    │
│  │ Parser  │ Lexer   │ │  SQL → AST                         │
│  ├─────────┼─────────┤ │                                    │
│  │ Planner │ Optimizer│ │  AST → Physical Plan + CBO        │
│  ├─────────┼─────────┤ │                                    │
│  │ Executor│         │ │  Volcano 模型 / Hash Join / 聚合   │
│  └─────────┴─────────┘ │                                    │
├─────────────────────────────────────────────────────────────┤
│  存储层 (storage/)      │  Buffer Pool + B+Tree + MVCC + WAL │
├─────────────────────────────────────────────────────────────┤
│  分布式 (distributed/)  │  Semi-sync / MTS / XA 事务         │
└─────────────────────────────────────────────────────────────┘
```

## 测试体系

| 级别 | 测试数 | 说明 |
|------|--------|------|
| Executor | 294 | 查询执行引擎测试 |
| Parser | 100 | SQL 解析测试 |
| SQL Corpus | 485 | 兼容性测试 (96.9% 通过) |
| TPC-H | 9/22 | 基准测试 |

## 质量门禁

| 门禁 | 目标 | 覆盖率要求 |
|------|------|-----------|
| A-Gate | 开发完成 | ≥50% |
| B-Gate | 功能冻结 | ≥65% |
| R-Gate | 发布候选 | ≥80% |
| G-Gate | 正式发布 | ≥85% |

## 相关资源

- [v2.9.0 文档索引](docs/releases/v2.9.0/README.md)
- [v2.9.0 综合说明](docs/releases/v2.9.0/README.md)
- [v2.9.0 发布说明](docs/releases/v2.9.0/RELEASE_NOTES.md)
- [v2.9.0 变更日志](docs/releases/v2.9.0/CHANGELOG.md)
- [v2.9.0 功能矩阵](docs/releases/v2.9.0/FEATURE_MATRIX.md)
- [v2.9.0 测试策略](docs/releases/v2.9.0/TEST_STRATEGY.md)
- [v2.9.0 Proof 覆盖](docs/releases/v2.9.0/PROOF_COVERAGE.md)
- [文档总索引](docs/README.md)