# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
