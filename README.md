# SQLRustGo

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85+-dea584?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/version-v2.6.0-blue" alt="Version">
  <img src="https://img.shields.io/badge/branch-main-green" alt="Branch">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

SQLRustGo 是一个使用 Rust 实现的关系型数据库教学与工程化项目，支持 SQL-92 子集，包含解析、规划、执行、存储、事务与网络模块。

## 当前状态

| 项目 | 当前值 |
|------|--------|
| 当前版本状态 | **v2.6.0 (GA)** |
| 当前主分支 | **main** (release/v2.6.0) |
| 当前阶段 | GA (正式发布) |
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
