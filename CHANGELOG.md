# Changelog

All notable changes to SQLRustGo will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0] - TBD

### Added

- **Architecture**: Vectorized execution engine with RecordBatch
- **Architecture**: StorageEngine trait for pluggable storage backends
- **Architecture**: FileStorage and MemoryStorage implementations
- **Architecture**: Cost-based optimizer (CBO) with statistics
- **Feature**: ANALYZE command for statistics collection
- **Feature**: Simplified CBO with table/column statistics
- **Feature**: LocalExecutor for embedded usage
- **Testing**: LocalExecutor tests (T-005)

### Changed

- **Refactor**: Storage layer abstraction
- **Refactor**: Statistics infrastructure

### Security

- **Audit**: Dependency audit passed (no high-severity vulnerabilities)

### Documentation

- **New**: v1.2.0 Release Notes
- **New**: v1.2.0 Upgrade Guide
- **New**: v1.2.0 Maturity Assessment
- **New**: v1.2.0 Test Plan (target 85%+ coverage)

## [1.1.0] - 2026-03-05

### Added

- **Architecture**: LogicalPlan/PhysicalPlan separation for query execution
- **Architecture**: ExecutionEngine trait for pluggable executors
- **Architecture**: Client-Server architecture with async network layer
- **Feature**: HashJoin implementation for efficient join operations
- **Feature**: Connection pool support for multiple clients
- **Feature**: WHERE clause AND/OR logical operators support
- **Feature**: Expression evaluation for BinaryOp (+, -, *, /)
- **Feature**: TEXT column index support (hash-based)
- **Testing**: Performance benchmark framework with Criterion
- **Testing**: Test coverage improved to 90.66%

### Changed

- **Refactor**: Replaced unwrap with proper error propagation in executor
- **Refactor**: Improved error handling with SqlResult<T>
- **Docs**: Updated gate checklist with correct branch workflow
- **Docs**: Reorganized teaching materials (student/TA separation)

### Fixed

- **Fix**: Clippy warnings resolved (zero warnings)
- **Fix**: Rust 2021 compatibility (let chains syntax)
- **Fix**: Code formatting issues

### Security

- **Audit**: Dependency audit passed
- **Audit**: No sensitive information leakage

### Documentation

- **New**: DeepSeek evaluation report
- **New**: Improvement plan for v1.1.0-draft
- **New**: AI-CLI collaboration notice
- **New**: v1.3.0 version plan with observability track
- **New**: 2.0 architecture design documents
- **New**: Distributed interface design (3.0 preview)
- **New**: Teaching practice materials (student/TA handbooks)

## [1.0.0] - 2026-02-22

### Added

- **Core**: SQL parser supporting SELECT, INSERT, UPDATE, DELETE
- **Core**: B+ tree storage engine
- **Core**: Transaction support with WAL
- **Core**: Basic query execution
- **Testing**: Unit test framework
- **Docs**: Initial documentation

### Changed

- Initial release

---

## Version History

| Version | Date | Maturity | Notes |
|---------|------|----------|-------|
| v1.2.0 | TBD | L3+ | Vectorization, CBO, Storage abstraction |
| v1.1.0 | 2026-03-05 | L3 | Architecture upgrade, Clippy passed |
| v1.0.0 | 2026-02-22 | L3 GA | Initial release |

---

## Roadmap

- **v1.2.0**: Development in progress (vectorization, CBO)
- **v1.1.0**: Released (architecture upgrade)
- **v1.3.0**: Enterprise features (observability, MVCC)
- **v2.0**: Distributed architecture

---

*This changelog is maintained by yinglichina8848*
=======
## [1.0.0] - 2026-02-16

### Added

- **SQL-92 子集支持**
  - SELECT 查询语句
  - INSERT 数据插入
  - UPDATE 数据更新
  - DELETE 数据删除
  - CREATE TABLE 表创建
  - DROP TABLE 表删除

- **存储引擎**
  - Buffer Pool 实现 (LRU 缓存)
  - FileStorage 持久化存储
  - 页面管理 (Page)

- **B+ Tree 索引**
  - 索引持久化
  - 查询优化

- **事务支持**
  - Write-Ahead Log (WAL)
  - TransactionManager
  - BEGIN/COMMIT/ROLLBACK

- **网络协议**
  - MySQL 风格协议实现
  - TCP 服务器/客户端
  - 数据包编解码

- **REPL 交互界面**
  - 命令行交互
  - SQL 语句执行

- **测试覆盖**
  - 集成测试
  - 项目结构测试
  - CI 配置验证

### Changed

- 使用 Rust Edition 2024
- 集成 Tokio 异步运行时
- 重构模块导出结构

### Fixed

- 修复 .exit 命令 bug
- 修复编译警告
- 统一项目名称为 sqlrustgo

---

## [0.0.1] - 2026-02-13

### Added

- 项目初始化
- 设计文档
- 实施计划
- AI 工具链配置
- 基础项目结构
>>>>>>> origin/main
=======
>>>>>>> origin/develop-v1.2.0
