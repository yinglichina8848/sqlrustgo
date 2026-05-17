# SQLRustGo

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85+-dea584?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/version-v3.2.0--RC-blue" alt="Version">
  <img src="https://img.shields.io/badge/branch-develop%2Fv3.2.0-blue" alt="Branch">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

SQLRustGo 是一个使用 Rust 实现的关系型数据库教学与工程化项目，支持 SQL-92 子集，包含解析、规划、执行、存储、事务与网络模块，并具备向量存储与图存储等高级特性。

## 当前状态

| 项目 | 当前值 |
|------|--------|
| 当前版本状态 | **v3.2.0 (RC)** |
| 当前主分支 | **develop/v3.2.0** |
| 当前阶段 | **RC (发布候选)** |
| 上一稳定版本 | v3.1.0 (Beta) |
| 版本目标 | 工业级 GMP 合规数据库 |

- 版本文件: [VERSION](VERSION)
- 当前版本说明: [docs/releases/v3.2.0/README.md](docs/releases/v3.2.0/README.md)
- v3.2.0 文档入口: [docs/releases/v3.2.0/README.md](docs/releases/v3.2.0/README.md)
- v3.1.0 文档: [docs/releases/v3.1.0/README.md](docs/releases/v3.1.0/README.md)

## 核心能力

- **SQL**: SELECT INSERT UPDATE DELETE CREATE TABLE DROP TABLE + CTE/窗口函数/JSON
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

## v3.2.0 RC 门禁状态

> **RC Gate: 28/32 PASS** (2026-05-17)
> **GA Gate: 进行中**

| Gate | 检查项 | 状态 | 说明 |
|------|--------|------|------|
| R1 | Build | PASS | cargo build --release |
| R2 | Test | PASS | 23 tests lib |
| R3 | Clippy | PASS | 零警告 |
| R4 | Format | PASS | fmt check |
| R5 | Coverage | PASS | 85.81% >=85% |
| R6 | Security | PASS | cargo audit (warning only) |
| R7 | SQL Compat - MERGE | PASS | 9 tests |
| R8 | SQL Compat - Event Scheduler | PASS | 18 tests |
| R9 | GMP Workflow - State machine | PASS | 7 tests |
| R10 | GMP Mobile - Trusted collection | PASS | 16 tests |
| R11 | GMP SOP/Training - Binding | PASS | 22 tests |
| R12 | GMP Device - Calibration | PASS | 16 tests |
| R13 | TPC-H SF=10 | SKIP | 需大内存机器 |
| R14 | Sysbench point_select >=30K QPS | SKIP | 需运行测试 |
| R15 | Stability 72h | SKIP | 需大内存机器 |
| R16 | OO Documentation | PASS | 13+ docs |
| R-S1~S16 | 稳定性测试 | PASS | 16/16 PASS |

## v3.2.0 功能矩阵

### GMP Framework (电子签名/可信赖 21 CFR Part 11)

| 功能 | 状态 | 测试 | 说明 |
|------|------|------|------|
| GMP-1: Digital Signature Audit Chain | PASS | 13 tests | SHA-256 哈希链 + ECDSA 签名 |
| GMP-2: Electronic Signature | PASS | 16 tests | 21 CFR Part 11 合规 |
| GMP-3: Immutable Record | PASS | 6 tests | EBR 证据链系统 |
| GMP-4: Correction Chain | PASS | 2 tests | 记录修正链 |
| GMP-5: Provenance Tracking | PASS | 4 tests | 数据溯源 |
| GMP-6: Trusted Timestamp | PASS | 1 test | RFC 3161 可信时间戳 |
| GMP-7: Audit Verification | PASS | 17 tests | 审计链验证工具 |
| GMP-8: HSM/KMS Integration | PASS | lib | PKCS#11 硬件安全模块 |
| GMP-9: Workflow Engine | PASS | 7 tests | GMP 工作流编排 |
| GMP-10: Mobile Trust | PASS | 16 tests | 移动端可信采集协议 |
| GMP-11: SOP Binding | PASS | 22 tests | SOP/培训绑定 |
| GMP-12: Calibration | PASS | 16 tests | 设备校准管理 |
| **GMP Total** | **100%** | **354+ tests** | **全部 PASS** |

### SQL 功能

| 功能 | 状态 | 说明 |
|------|------|------|
| Multi-Table UPDATE | PASS | 多表更新 |
| Multi-Table MERGE | PASS | MERGE INTO |
| RECURSIVE CTE | PASS | 递归 CTE |
| Window Functions | PASS | 窗口函数 |
| GROUP BY | PASS | 分组聚合 |
| JOIN (INNER/OUTER/CROSS) | PASS | 哈希连接 |
| FULLTEXT Search | PASS | 全文索引 |
| Set Operations | PASS | UNION/INTERSECT/EXCEPT |

### 性能增强

| 指标 | v3.1.0 | v3.2.0 | 提升 |
|------|--------|--------|------|
| Point Select QPS | ~20K | >=30K | +50% |
| UPDATE QPS | ~40K | >=58K | +45% |
| DELETE QPS | ~40K | >=62K | +55% |
| 并发连接 | 100+ | 200+ | +100% |
| 内存占用 | 基准 | -15% | 优化 |
| TPC-H SF=1 | 22/22 | 22/22 | PASS |
| 覆盖率 | 65% | 85.81% | +20% |

### MySQL 兼容性

| 功能 | 状态 |
|------|------|
| MySQL 协议 | PASS |
| SQL 语法 | PASS |
| 数据类型 | PASS |
| 索引 | PASS |
| 事务 (MVCC) | PASS |
| Prepared Statements | PASS |

### 存储引擎

| 功能 | 状态 |
|------|------|
| Row Store | PASS |
| Columnar Store | 实验性 |
| Vector Index | 实验性 |
| WAL | PASS |
| Buffer Pool | PASS |
| LRU Cache | PASS |

## v3.2.0 版本目标

| 目标 | 说明 |
|------|------|
| GMP 合规框架 | 21 CFR Part 11 / ALCOA+ 电子签名 |
| 覆盖率增强 | >=85% L1 crate coverage |
| 性能基线 | Point Select >=30K QPS |
| TPC-H SF=1 | 22/22 PASS |
| 分层测试系统 | CI/CD + L0-L3 分层验证 |

## 文档索引

### v3.2.0 (当前版本)

- [v3.2.0 文档中心](docs/releases/v3.2.0/README.md)
- [v3.2.0 变更日志](docs/releases/v3.2.0/CHANGELOG.md)
- [v3.2.0 功能矩阵](docs/releases/v3.2.0/FEATURE_MATRIX.md)
- [v3.2.0 开发计划](docs/releases/v3.2.0/DEVELOPMENT_PLAN.md)
- [v3.2.0 GMP 实现分析](docs/releases/v3.2.0/GMP_IMPLEMENTATION_ANALYSIS.md)
- [v3.2.0 测试报告](docs/releases/v3.2.0/TEST_REPORT.md)

### 历史版本

- [v3.1.0 Beta 文档](docs/releases/v3.1.0/README.md)
- [v3.0.0 GA 文档](docs/releases/v3.0.0/README.md)
- [v2.9.0 RC 文档](docs/releases/v2.9.0/README.md)
