# SQLRustGo

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85+-dea584?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/version-v3.0.0--GA-green" alt="Version">
  <img src="https://img.shields.io/badge/branch-develop%2Fv3.0.0-green" alt="Branch">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

SQLRustGo 是一个使用 Rust 实现的关系型数据库教学与工程化项目，支持 SQL-92 子集，包含解析、规划、执行、存储、事务与网络模块，并具备向量存储与图存储等高级特性。

## 当前状态

| 项目 | 当前值 |
|------|--------|
| 当前版本状态 | **v3.0.0 (GA)** |
| 当前主分支 | **develop/v3.0.0** |
| 当前阶段 | **GA (General Availability)** |
| 上一稳定版本 | v2.9.0 (RC) |
| 版本目标 | 企业级韧性 (Enterprise Resilience) |

- 版本文件: [VERSION](VERSION)
- 当前版本说明: [docs/releases/v3.0.0/README.md](docs/releases/v3.0.0/README.md)
- v3.0.0 文档入口: [docs/releases/v3.0.0/README.md](docs/releases/v3.0.0/README.md)
- v2.9.0 文档: [docs/releases/v2.9.0/README.md](docs/releases/v2.9.0/README.md)

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

## v3.0.0 GA 门禁状态

> **Beta Gate: 6/6 全部通过 ✅**
> **RC Gate: 10+/10+ 全部通过 ✅**
> **GA Gate: 全部通过 ✅**

| Gate | 检查项 | 状态 |
|------|--------|------|
| B-S1 | concurrency_stress_test (9 tests) | ✅ PASS |
| B-S2 | crash_recovery_test (8 tests) | ✅ PASS |
| B-S3 | long_run_stability_test (10 tests) | ✅ PASS |
| B-S4 | wal_integration_test (16 tests) | ✅ PASS |
| B-S5 | network_tcp_smoke_test (6 tests) | ✅ PASS |
| B-S6 | ssi_stress_test | ✅ PASS |
| R7 | clippy / fmt | ✅ 零警告 |
| R8 | SQL Corpus ≥80% | ✅ 92.6% |
| R9 | 性能回归检测 | ✅ |
| R10 | TPC-H 基线 | ✅ |

## v3.0.0 核心进展

### 测试体系完成 (B-S1 ~ B-S6) ✅

| 功能 | 状态 |
|------|------|
| B-S1 并发压力测试 | ✅ 9/9 PASS |
| B-S2 崩溃恢复测试 | ✅ 8/8 PASS |
| B-S3 长期稳定性测试 | ✅ 10/10 PASS |
| B-S4 WAL 集成测试 | ✅ 16/16 PASS |
| B-S5 网络 TCP 冒烟测试 | ✅ 6/6 PASS |
| B-S6 SSI 隔离级别压力测试 | ✅ |

### 性能基线建立 ✅

| 基准测试 | QPS | 说明 |
|----------|-----|------|
| simple_select | 398,353 | 单表单查询 |
| update | 43,121 | E-09 ≥10,000 ✅ |
| delete | 64,896 | E-09 ≥10,000 ✅ |
| aggregation | 1,666,100 | 聚合查询 |

### 分布式架构完成 (D-01 ~ D-04) ✅

| 功能 | 状态 |
|------|------|
| D-01 Semi-sync 复制 | ✅ |
| D-02 MTS 并行复制 | ✅ |
| D-03 Multi-source 复制 | ✅ |
| D-04 XA 事务协调器 | ✅ |

### SQL 兼容性提升 (C-01 ~ C-06) ✅

| 功能 | 状态 |
|------|------|
| CTE / WITH 递归 | ✅ |
| 窗口函数 (ROW_NUMBER/RANK/DENSE_RANK) | ✅ |
| CASE / WHEN | ✅ |
| JSON 操作 | ✅ |
| INSERT...SELECT | ✅ |

## v3.0.0 已知限制（延期至 v3.1.0）

| 优先级 | 弱项 | 说明 |
|--------|------|------|
| P1 | 事件调度器 | CREATE EVENT 未实现 |
| P1 | 全文索引 | MATCH...AGAINST 未实现 |
| P2 | 空间数据类型 | GEOMETRY/POINT 未实现 |
| P2 | 存储过程游标/异常 | 基础控制流 |
| P2 | INFORMATION_SCHEMA | 不完整 |
| P2 | performance_schema | 未实现 |

详见: [docs/releases/v3.0.0/README.md](docs/releases/v3.0.0/README.md)

## MySQL 5.7 / 8.3 差距总览

> v3.0.0 综合评分: **62.5/100**（v2.9.0: 56.7/100）

| 维度 | v3.0.0 vs MySQL 8.3 | 主要缺口 |
|------|----------------------|----------|
| SQL 覆盖度 | 75/100 | 事件调度器、全文索引、空间数据 |
| 存储引擎 | 68/100 | 无聚簇索引、无压缩 |
| 事务 ACID | 72/100 | 缺 SERIALIZABLE 隔离级别 |
| 复制与 HA | 70/100 | 无组复制、无自动故障转移 |
| 安全 | 65/100 | 无 TLS/SSL |
| 性能 | 55/100 | P99 延迟差距 |
| 运维生态 | 40/100 | 无 INFORMATION_SCHEMA、performance_schema |

详见: [docs/releases/v3.0.0/README.md](docs/releases/v3.0.0/README.md#六mysql-57--83-差距分析)

## 文档索引

### v3.0.0 (当前版本)

- [v3.0.0 文档中心](docs/releases/v3.0.0/README.md)
- [v3.0.0 变更日志](docs/releases/v3.0.0/CHANGELOG.md)
- [v3.0.0 功能矩阵](docs/releases/v3.0.0/FEATURE_MATRIX.md)

### 历史版本

- [v2.9.0 RC 文档](docs/releases/v2.9.0/README.md)
- [v2.8.0 GA 文档](docs/releases/v2.8.0/README.md)

# v3.0.0 Gate Verification