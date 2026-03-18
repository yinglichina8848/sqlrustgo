# 变更日志 (Changelog)

> **版本**: v1.3.0  
> **格式**: 基于 Keep a Changelog

---

## 格式说明

本文档使用 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/) 格式。

### 版本类型

- **Added**: 新功能
- **Changed**: 已有功能的变化
- **Deprecated**: 已废弃的功能
- **Removed**: 已移除的功能
- **Fixed**: 问题修复
- **Security**: 安全相关

---

## [已发布] v1.3.0

> **发布日期**: 2026-03-15  
> **代号**: Enterprise Ready (架构稳定版)

### Added

#### Executor 模块
- **架构**: Volcano Model 统一 Executor trait
- **功能**: TableScan 算子完善
- **功能**: Projection 算子实现
- **功能**: Filter 算子实现
- **功能**: HashJoin 算子实现 (内连接)
- **测试**: Executor 测试框架

#### Planner 模块
- **测试**: Planner 测试套件

#### 可观测性
- **功能**: Metrics trait 定义 (M-001) ✅ 已完成
- **功能**: BufferPool 指标收集 (M-002) ✅ 已完成
- **功能**: /health/live 端点 (H-001) ✅ 已完成
- **功能**: /health/ready 端点 (H-002) ✅ 已完成
- **功能**: /health 综合端点 (H-004) ✅ 已完成
- **功能**: /metrics 端点 (E-001) ✅ 已完成
- **功能**: Prometheus 格式支持 ✅ 已完成

### Changed

- **重构**: 逻辑计划/物理计划分离
- **重构**: Executor 接口标准化

### Fixed

- **修复**: 性能基准测试编译错误
- **修复**: HashJoin 测试解析问题

### Security

- 依赖审核已通过 (无高严重性漏洞)

---

## [已发布] v1.2.0

> **发布日期**: 2026-03-13  
> **代号**: Vector Engine

### Added

- **架构**: 带有 RecordBatch 的矢量化执行引擎
- **架构**: 可插拔存储后端的 StorageEngine 特征
- **架构**: 文件存储和内存存储实现
- **架构**: 带有统计信息的基于成本的优化器 (CBO)
- **功能**: 用于统计收集的 ANALYZE 命令
- **功能**: 简化 CBO 与表/列统计信息
- **功能**: 用于嵌入式使用的 LocalExecutor
- **功能**: HashJoinExec 表连接实现
- **功能**: 聚合函数 (COUNT, SUM, AVG, MIN, MAX)
- **功能**: ProjectionExec 列投影 (Wildcard, Alias, BinaryExpr)
- **功能**: FilterExec 谓词过滤
- **功能**: SeqScanExec 全表扫描
- **优化器**: Predicate Pushdown 谓词下推
- **优化器**: Projection Pruning 投影裁剪
- **优化器**: Constant Folding 常量折叠
- **优化器**: Expression Simplification 表达式简化
- **优化器**: Join Reordering 连接重排序

### Changed

- **重构**: 存储层抽象
- **重构**: 统计基础设施
- **重构**: PhysicalPlan execute() 方法实现

### Fixed

- **修复**: Optimizer 规则测试编译错误
- **修复**: CI 配置完善 (release/* 分支触发)
- **修复**: Benchmark 编译适配

### Security

- 依赖审核已通过 (无高严重性漏洞)

---

## [已发布] v1.1.0

> **发布日期**: 2026-02-28

### Added

- **架构**: 查询执行的逻辑计划/物理计划分离
- **架构**: 可插入执行器的 ExecutionEngine 特征
- **架构**: 具有异步网络层的客户端-服务器架构
- **功能**: HashJoin 高效连接操作
- **功能**: 连接池支持多个客户端
- **功能**: WHERE 子句 AND/OR 逻辑运算符支持
- **功能**: BinaryOp 表达式评估 (+, -, *, /)
- **功能**: TEXT 列索引支持 (基于哈希)
- **测试**: 使用 Criterion 的性能基准框架
- **测试**: 测试覆盖率提高至 90.66%

### Changed

- **重构**: 用执行器中正确的错误传播替换了 unwrap
- **重构**: 改进了 SqlResult<T> 的错误处理

### Fixed

- **修复**: Clippy 警告 (零警告)
- **修复**: Rust 2021 兼容性 (let 链语法)
- **修复**: 代码格式问题

---

## [已发布] v1.0.0

> **发布日期**: 2026-02-22

### Added

- **核心**: SQL 解析器 (SELECT, INSERT, UPDATE, DELETE)
- **核心**: 逻辑计划构建器
- **核心**: 基础优化器
- **核心**: 简单执行器
- **存储**: 内存存储引擎
- **存储**: 基础缓冲池
- **网络**: 简单客户端/服务器通信

---

## 版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.3.0 | 2026-03-15 | Enterprise Ready (架构稳定版) |
| v1.2.0 | 2026-03-13 | Vector Engine |
| v1.1.0 | 2026-02-28 | 稳定性增强 |
| v1.0.0 | 2026-02-22 | 初始发布 |

---

## 贡献者

> 按贡献时间排序

- Claude Code
- OpenCode
- Codex
- DeepSeek

---

## 相关文档

- [v1.3.0 开发计划](./DEVELOPMENT_PLAN.md)
- [v1.3.0 任务矩阵](./TASK_MATRIX.md)
- [v1.3.0 测试计划](./TEST_PLAN.md)
- [v1.2.0 发布说明](../v1.2.0/RELEASE_NOTES.md)

---

**最后更新**: 2026-03-15
