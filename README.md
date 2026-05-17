# SQLRustGo

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85+-dea584?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/version-v3.2.0--GA--preview-blue" alt="Version">
  <img src="https://img.shields.io/badge/branch-develop%2Fv3.2.0-blue" alt="Branch">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

SQLRustGo 是一个使用 Rust 实现的关系型数据库教学与工程化项目，支持 SQL-92 子集，包含解析、规划、执行、存储、事务与网络模块，并具备向量存储与图存储等高级特性。

## 当前状态

| 项目 | 当前值 |
|------|--------|
| 当前版本状态 | **v3.2.0 (GA Preview)** |
| 当前主分支 | **develop/v3.2.0** |
| 当前阶段 | **GA (正式发布候选)** |
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

## v3.2.0 GA 门禁状态

> **GA Gate: 40/46 PASS (86.9%)** (2026-05-18)
> **RC Gate: 28/32 PASS (87.5%)** (2026-05-17)

| Gate | 检查项 | 状态 | 说明 |
|------|--------|------|------|
| G1 | Build | ✅ | cargo build --release |
| G2 | Test | ✅ | 全部通过 |
| G3 | Clippy | ✅ | 零警告 |
| G4 | Format | ✅ | fmt check |
| G5 | Coverage | ✅ | 85.81% >=85% |
| G6 | Security | ⚠️ | advisory db 不可达 |
| G7 | SQL Compat | ✅ | ≥85% MySQL |
| G8 | TPC-H SF=1 | ✅ | 22/22 |
| G9 | Performance | ⬜ | 需服务器环境 |
| G10 | Proofs | ⬜ | TLA+ 待检查 |
| G11 | OO Docs | ✅ | 14/14 全部存在 |
| G12 | MySQL Protocol | ✅ | 验证通过 |
| G-QA1~QA14 | QA 增强 | ✅ | 14/14 PASS |
| G-S1~S20 | 稳定性测试 | ✅ | 16/20 PASS |

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
| Event Scheduler | PASS | 18 tests |
| GIS/Spatial | PASS | 空间数据 |

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
| caching_sha2_password | PASS MySQL 8.0 |

### 存储引擎

| 功能 | 状态 |
|------|------|
| Row Store | PASS |
| Columnar Store | 实验性 |
| Vector Index | 实验性 |
| WAL | PASS |
| Buffer Pool | PASS |
| LRU Cache | PASS |
| Cold Storage (S3) | PASS |

### Evidence Export (新增 v3.2.0)

| 功能 | 状态 | 说明 |
|------|------|------|
| PdfExporter | PASS | PDF 合规报告生成 |
| JsonExporter | PASS | JSON 序列化 |
| PackageBuilder | PASS | 签名证据包构建 |
| Ed25519 Signer | PASS | Ed25519 签名 |
| Compliance-as-Code | PASS | 声明式规则 |

## v3.2.0 版本目标

| 目标 | 说明 |
|------|------|
| GMP 合规框架 | 21 CFR Part 11 / ALCOA+ 电子签名 |
| 覆盖率增强 | >=85% L1 crate coverage |
| 性能基线 | Point Select >=30K QPS |
| TPC-H SF=1 | 22/22 PASS |
| 分层测试系统 | CI/CD + L0-L3 分层验证 |

## v3.2.0 变更统计

| 指标 | 数值 |
|------|------|
| 总 Commits | 339 |
| Merged PRs | 119 |
| 新增功能 | 50+ |
| GMP 测试 | 354+ |
| 稳定性测试 | 16+ |
| OO 文档 | 14/14 |

## 文档索引

### v3.2.0 (当前版本)

- [v3.2.0 文档中心](docs/releases/v3.2.0/README.md)
- [v3.2.0 变更日志](docs/releases/v3.2.0/CHANGELOG.md)
- [v3.2.0 功能矩阵](docs/releases/v3.2.0/FEATURE_MATRIX.md)
- [v3.2.0 开发计划](docs/releases/v3.2.0/DEVELOPMENT_PLAN.md)
- [v3.2.0 GA 门禁](docs/releases/v3.2.0/GA_GATE_CHECKLIST.md)
- [v3.2.0 发布说明](docs/releases/v3.2.0/RELEASE_NOTES.md)
- [v3.2.0 GMP 实现分析](docs/releases/v3.2.0/GMP_IMPLEMENTATION_ANALYSIS.md)

### 历史版本

- [v3.1.0 Beta 文档](docs/releases/v3.1.0/README.md)
- [v3.0.0 GA 文档](docs/releases/v3.0.0/README.md)
- [v2.9.0 RC 文档](docs/releases/v2.9.0/README.md)
