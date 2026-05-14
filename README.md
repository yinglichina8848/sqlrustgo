# SQLRustGo

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85+-dea584?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/version-v3.1.0-GA-green" alt="Version">
  <img src="https://img.shields.io/badge/branch-develop%2Fv3.1.0-green" alt="Branch">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

SQLRustGo 是一个使用 Rust 实现的关系型数据库教学与工程化项目，支持 SQL-92 子集，包含解析、规划、执行、存储、事务与网络模块，并具备向量存储与图存储等高级特性。

## 当前状态

| 项目 | 当前值 |
|------|--------|
| 当前版本状态 | **v3.1.0 (Beta)** |
| 当前主分支 | **develop/v3.1.0** |
| 当前阶段 | **Beta (发布候选)** |
| 上一稳定版本 | v3.0.0 (GA) |
| 版本目标 | 工业级 QA 骨架 |

- 版本文件: [VERSION](VERSION)
- 当前版本说明: [docs/releases/v3.1.0/README.md](docs/releases/v3.1.0/README.md)
- v3.1.0 文档入口: [docs/releases/v3.1.0/README.md](docs/releases/v3.1.0/README.md)
- v3.0.0 文档: [docs/releases/v3.0.0/README.md](docs/releases/v3.0.0/README.md)

## 核心能力

- **SQL**: `SELECT` `INSERT` `UPDATE` `DELETE` `CREATE TABLE` `DROP TABLE` + CTE/窗口函数/JSON
- **存储**: Buffer Pool + FileStorage + MemoryStorage + ColumnarStorage
- **索引**: B+ Tree + Hash Index + Vector Index
- **事务**: WAL + MVCC (Snapshot Isolation) + XA 两阶段提交
- **网络**: TCP / MySQL 风格协议
- **高级**: 向量存储、图存储、Prepared Statement、存储过程、触发器
- **分布式**: Semi-sync 复制、MTS 并行复制、Multi-source 复制

## 快速开始

```bash
# 构建
cargo build --all-features

# 运行测试
cargo test --all-features

# 启动 REPL
cargo run --bin sqlrustgo

# 代码检查
cargo clippy --all-targets -- -D warnings
```

## v3.1.0 Beta 门禁状态

> **Beta Gate: 18/18 全部通过 ✅** (2026-05-14)

| Gate | 检查项 | 状态 |
|------|--------|------|
| B1 | Build | ✅ PASS |
| B2 | L1 core crates test | ✅ PASS |
| B3 | Clippy | ✅ PASS |
| B4 | Format | ✅ PASS |
| B5 | Coverage | ✅ PASS |
| B6 | OO Docs | ✅ PASS |
| B7 | SQL Compat | 🟡 IN PROGRESS |
| B8 | TPC-H SF=1 | 🟡 SKIP |
| B9 | Docs Links | ✅ PASS |

## v3.1.0 核心进展

### 测试体系完成 ✅

| 功能 | 状态 |
|------|------|
| Alpha Gate | ✅ 12/12 PASS |
| Beta Gate | ✅ 18/18 PASS |
| RC Gate | 🟡 进行中 |
| QA 增强 | ✅ 6/6 完成 |

| 功能 | 状态 |
|------|------|
| Alpha Gate | ✅ 12/12 PASS |
| Beta Gate | ✅ 18/18 PASS |
| RC Gate | 🟡 进行中 |
| QA 增强 | ✅ 6/6 完成 |

## v3.1.0 版本目标

| 目标 | 说明 |
|------|------|
| 工业级 QA 骨架 | sqllogictest, Mutation Testing, CI/CD |
| 静态分析工具链 | Miri, Sanitizers, cargo-audit |
| Benchmark 标准化 | TPC-H SF=1, Point Select |
| 测试覆盖增强 | 覆盖率 ≥65% |

## 已知限制（待 v3.2.0 解决）

| 优先级 | 弱项 | 说明 |
|--------|------|------|
| P1 | 空间数据类型 | GEOMETRY/POINT 需完善 |
| P1 | TPC-H SF=1 | 需内存优化 |
| P2 | Complex WHERE QPS | 需谓词下推优化 |
| P2 | Mutation Testing | 需集成 cargo-mutants |

## 文档索引

### v3.1.0 (当前版本)

- [v3.1.0 文档中心](docs/releases/v3.1.0/README.md)
- [v3.1.0 变更日志](docs/releases/v3.1.0/CHANGELOG.md)
- [v3.1.0 功能矩阵](docs/releases/v3.1.0/FEATURE_MATRIX.md)
- [v3.1.0 开发计划](docs/releases/v3.1.0/DEVELOPMENT_PLAN.md)
- [v3.1.0 QA 增强计划](docs/releases/v3.1.0/QA_ENHANCEMENT_PLAN_RC_GA.md)

### 历史版本

- [v3.0.0 GA 文档](docs/releases/v3.0.0/README.md)
- [v2.9.0 RC 文档](docs/releases/v2.9.0/README.md)
- [v2.8.0 GA 文档](docs/releases/v2.8.0/README.md)

## v3.1.0 Beta 门禁状态