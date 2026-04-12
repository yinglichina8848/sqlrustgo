# 变更日志 (Changelog)

> **版本**: v1.4.0  
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

## [已发布] v1.4.0

> **发布日期**: 2026-03-16  
> **代号**: CBO & Vectorization Ready (性能增强版)

### Added

#### CBO 优化器
- **功能**: 代价模型基础 (CostModel) ✅ 已完成
- **功能**: 统计信息集成 (CBO-02) ✅ 已完成
- **功能**: 索引选择优化 (CBO-04) ✅ 已完成
- **功能**: Join 顺序优化 (CBO-03) ✅ 已存在
- **功能**: IndexSelect 规则实现

#### Join 算法增强
- **功能**: SortMergeJoin 算子 (SMJ-01) ✅ 已完成
- **功能**: SortMergeJoin 测试 (SMJ-02) ✅ 已完成
- **功能**: NestedLoopJoin 算子 (NLJ-01) ✅ 已完成
- **功能**: Cross Join 支持 ✅ 已完成
- **功能**: Left/Right/Full Outer Join 支持 ✅ 已完成
- **功能**: Semi/Anti Join 支持 ✅ 已完成

#### 向量化基础
- **功能**: SIMD 基础设施 (vectorization.rs) ✅ 已完成
- **功能**: Vector 数据结构 ✅ 已完成
- **功能**: BatchIterator trait ✅ 已完成

#### 可观测性增强
- **功能**: /metrics 端点 (M-004) ✅ 已完成
- **功能**: Prometheus 格式支持 (M-003) ✅ 已完成
- **功能**: Grafana Dashboard 模板 (M-005) ✅ 已完成
- **功能**: actix-web HTTP 服务器 ✅ 已完成

#### 性能基准
- **功能**: TPC-H 基准测试 (PB-01) ✅ 已完成
- **功能**: CBO 基准测试 ✅ 已完成
- **功能**: v1.4.0 性能报告 ✅ 已完成

### Changed

- **增强**: HashJoin 支持更多 Join 类型
- **增强**: Optimizer 规则稳定性提升

### Fixed

- **修复**: 优化器规则编译问题

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
- **功能**: Metrics trait 定义
- **功能**: BufferPool/Executor/Network 指标
- **功能**: Grafana Dashboard
- **功能**: /health/live, /health/ready, /health 端点

### Changed

- **重构**: 逻辑计划/物理计划分离
- **重构**: Executor 接口标准化

---

## 版本历史

| 版本 | 发布日期 | 代号 | 状态 |
|------|----------|------|------|
| v1.4.0 | 2026-03-16 | CBO & Vectorization Ready | ✅ |
| v1.3.0 | 2026-03-15 | Enterprise Ready | ✅ |
| v1.2.0 | 2026-02-15 | Statistics Ready | ✅ |
| v1.1.0 | 2026-01-15 | Observability Ready | ✅ |
| v1.0.0 | 2025-12-15 | MVP | ✅ |

---

**文档版本**: 1.0
**最后更新**: 2026-03-16
