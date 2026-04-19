# SQLRustGo

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85+-dea584?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/version-alpha%2Fv2.6.0-blue" alt="Version">
  <img src="https://img.shields.io/badge/branch-develop%2Fv2.6.0-green" alt="Branch">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

SQLRustGo 是一个使用 Rust 实现的关系型数据库教学与工程化项目，支持 SQL-92 子集，包含解析、规划、执行、存储、事务与网络模块。

## 当前状态

| 项目 | 当前值 |
|------|--------|
| 当前版本状态 | **alpha/v2.6.0** |
| 当前开发分支 | **develop/v2.6.0** |
| 当前阶段 | Alpha (内测) |
| 上一稳定版本 | v2.5.0 |
| 版本目标 | 生产就绪 (替代 MySQL) |

- 版本文件: [VERSION](VERSION)
- 当前版本说明: [CURRENT_VERSION.md](CURRENT_VERSION.md)
- v2.6.0 文档入口: [docs/releases/v2.6.0/README.md](docs/releases/v2.6.0/README.md)

## 核心能力

- SQL: `SELECT` `INSERT` `UPDATE` `DELETE` `CREATE TABLE` `DROP TABLE`
- 存储: Buffer Pool + FileStorage + MemoryStorage + ColumnarStorage
- 索引: B+ Tree + Hash Index + Vector Index
- 事务: WAL + MVCC (Snapshot Isolation)
- 网络: TCP / MySQL 风格协议
- 交互: REPL
- 高级: 向量存储、图存储、Prepared Statement、存储过程、触发器

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

## 文档导航

### 当前版本 (alpha/v2.6.0)

- [v2.6.0 文档索引](docs/releases/v2.6.0/README.md)
- [版本计划](docs/releases/v2.6.0/VERSION_PLAN.md)
- [发布门禁](docs/releases/v2.6.0/RELEASE_GATE_CHECKLIST.md)
- [测试计划](docs/releases/v2.6.0/TEST_PLAN.md)
- [功能集成状态](docs/releases/v2.6.0/INTEGRATION_STATUS.md)
- [性能目标](docs/releases/v2.6.0/PERFORMANCE_TARGETS.md)

### 已发布版本

- [v2.5.0 文档](docs/releases/v2.5.0/)
- [v2.4.0 文档](docs/releases/v2.4.0/)
- [v2.1.0 文档](docs/releases/v2.1.0/)
- [v2.0.0 文档](docs/releases/v2.0.0/)
- [v1.9.0 文档](docs/releases/v1.9.0/)
- [v1.6.1 文档](docs/releases/v1.6.1/)
- [v1.1.0 文档](docs/releases/v1.1.0/)
- [v1.0.0 文档](docs/releases/v1.0.0/)

### 中长期规划

- [长期路线图](docs/releases/LONG_TERM_ROADMAP.md)
- [2.0 路线图](docs/v2.0/SQLRUSTGO_2_0_ROADMAP.md)

## 分支与提交流程

当前主开发分支是 `develop/v2.6.0`。

推荐流程：

1. 从 `develop/v2.6.0` 拉出功能/修复分支
2. 提交 PR 到 `develop/v2.6.0`
3. CI 通过后合并

## v2.6.0 开发目标

| 类别 | 目标 |
|------|------|
| SQL-92 支持 | 完整支持聚合函数、JOIN、GROUP BY |
| 隔离级别 | MVCC SSI (Serializable Snapshot Isolation) |
| 覆盖率 | 49% → 70% |
| 性能 | TPC-H SF=1 < 5s |

详见 [v2.6.0 版本计划](docs/releases/v2.6.0/VERSION_PLAN.md)

## 版本演进

| 版本 | 状态 | 核心特性 |
|------|------|----------|
| v1.0.0 | ✅ 已发布 | 基础 SQL 引擎 |
| v1.1.0 | ✅ 已发布 | 查询执行器、HASH JOIN |
| v2.0.0 | ✅ 已发布 | 异步网络、连接池 |
| v2.4.0 | ✅ 已发布 | SIMD 加速、列式存储 |
| v2.5.0 | ✅ 已发布 | MVCC、Vector/Graph 存储、统一查询 |
| **v2.6.0** | 🔄 开发中 | **生产就绪、SQL-92 完整** |
| v2.7.0 | 📋 规划中 | 分布式架构 |
