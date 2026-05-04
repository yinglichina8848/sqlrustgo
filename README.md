# SQLRustGo

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85+-dea584?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/version-v2.9.0--RC-blue" alt="Version">
  <img src="https://img.shields.io/badge/branch-develop%2Fv2.9.0-green" alt="Branch">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

SQLRustGo 是一个使用 Rust 实现的关系型数据库教学与工程化项目，支持 SQL-92 子集，包含解析、规划、执行、存储、事务与网络模块，并具备向量存储与图存储等高级特性。

## 当前状态

| 项目 | 当前值 |
|------|--------|
| 当前版本状态 | **v2.9.0 (RC)** |
| 当前主分支 | **develop/v2.9.0** |
| 当前阶段 | RC（v2.9.0-rc.1） |
| 上一稳定版本 | v2.8.0 (GA) |
| 版本目标 | 企业级韧性 (Enterprise Resilience) |

- 版本文件: [VERSION](VERSION)
- 当前版本说明: [CURRENT_VERSION.md](CURRENT_VERSION.md)
- v2.9.0 文档入口: [docs/releases/v2.9.0/README.md](docs/releases/v2.9.0/README.md)
- v2.9.0 RC 状态: [docs/releases/v2.9.0/RC_STATUS_20260505.md](docs/releases/v2.9.0/RC_STATUS_20260505.md)
- v2.8.0 文档: [docs/releases/v2.8.0/README.md](docs/releases/v2.8.0/README.md)

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

## v2.9.0 RC 门禁状态

> **RC Gate: 13/13 全部通过 ✅**

| Gate | 检查项 | 结果 |
|------|--------|------|
| B1 | 总覆盖率 ≥75% | ✅ **84.18%** |
| B2 | executor 覆盖率 ≥60% | ✅ **71.08%** |
| R8 | SQL Corpus ≥80% | ✅ **92.6%** |
| R4 | cargo test --all-features | ✅ |
| R7 | cargo clippy / fmt | ✅ 零警告 |
| R8 | cargo audit | ✅ 0 漏洞 |
| D1-D4 | 必需文档 | ✅ |

## v2.9.0 核心进展

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
| INSERT ON DUPLICATE KEY UPDATE | ✅ |

### TPC-H 测试状态

| 指标 | 结果 | 说明 |
|------|------|------|
| Parser | **22/22 ✅** | 全部查询解析成功 |
| Executor 可运行 | **22/22 ✅** | Q1-Q22 全部可执行 |
| 结果与 SQLite 一致 | 3/22 | Q6, Q17, Q19 |
| 性能差距 | ⚠️ | 仅性能差距，无语法/运行时错误 |

### RC 覆盖率

| 指标 | 数值 | 目标 | 状态 |
|------|------|------|------|
| 整体行覆盖率 | **84.18%** | ≥75% | ✅ |
| SQL Corpus 通过率 | **92.6%** | ≥80% | ✅ |
| executor 覆盖率 | **71.08%** | ≥60% | ✅ |
| 形式化证明 | **18/18** | — | ✅ |

## v2.9.0 已知限制（延期至 v2.10.0）

| 优先级 | 弱项 | 说明 |
|--------|------|------|
| **P0** | E-08 性能优化 | QPS ~1,000，目标 ≥10,000 |
| P1 | INSERT...SELECT | 尚未实现 |
| P1 | 窗口函数不完整 | 缺 NTILE/LEAD/LAG/NTH_VALUE |
| P2 | 存储过程游标/异常 | 基础控制流 |
| P2 | 事件调度器 | CREATE EVENT 未实现 |
| P2 | 全文索引 | MATCH...AGAINST 未实现 |
| P2 | 空间数据类型 | GEOMETRY/POINT 未实现 |

详见: [docs/releases/v2.9.0/README.md](docs/releases/v2.9.0/README.md)

## MySQL 5.6 / 5.7 差距总览

> v2.9.0 综合评分: **56.7/100**（v2.8.0: 45.5/100）

| 维度 | v2.9.0 vs MySQL 5.7 | 主要缺口 |
|------|----------------------|----------|
| SQL 覆盖度 | 72/100 | 事件调度器、全文索引、空间数据 |
| 存储引擎 | 65/100 | 无聚簇索引、无压缩 |
| 事务 ACID | 70/100 | 缺 SERIALIZABLE 隔离级别 |
| 复制与 HA | 68/100 | 无组复制、无自动故障转移 |
| 安全 | 62/100 | 无 AES-256 加密、无 TLS |
| 性能 | 45/100 | QPS ~1,000（目标 ≥10K） |
| 运维生态 | 35/100 | 无 INFORMATION_SCHEMA、无 performance_schema |

详见: [docs/releases/v2.9.0/README.md](docs/releases/v2.9.0/README.md#mysql-56--57-差距分析)

## 文档索引

### v2.9.0

- [v2.9.0 文档中心](docs/releases/v2.9.0/README.md)
- [v2.9.0 RC 状态报告](docs/releases/v2.9.0/RC_STATUS_20260505.md)
- [版本计划](docs/releases/v2.9.0/VERSION_PLAN.md)
- [发布门禁清单](docs/releases/v2.9.0/RELEASE_GATE_CHECKLIST.md)
- [RC 门禁报告](docs/releases/v2.9.0/RC_GATE_REPORT.md)
- [功能矩阵](docs/releases/v2.9.0/FEATURE_MATRIX.md)
- [集成状态](docs/releases/v2.9.0/INTEGRATION_STATUS.md)
- [测试计划](docs/releases/v2.9.0/TEST_PLAN.md)
- [快速开始](docs/releases/v2.9.0/QUICK_START.md)

### 历史版本

- [v2.8.0 GA 文档](docs/releases/v2.8.0/README.md)
- [v2.7.0 文档](docs/releases/v2.7.0/README.md)
