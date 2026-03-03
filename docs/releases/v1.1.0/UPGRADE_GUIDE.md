# v1.0.0 → v1.1.0 升级指南

## 概述

本文档提供从 v1.0.0 升级到 v1.1.0 的指南。

## 主要变更

### 1. 性能测试框架

v1.1.0 引入了完整的性能测试框架，包括：
- Lexer 基准测试
- Parser 基准测试
- Executor 基准测试
- Storage 基准测试
- Network 基准测试
- Planner 基准测试
- Integration 基准测试

**影响**: 无 API 变更，仅增加性能测试能力。

### 2. HashJoin 支持

添加了 HashJoin 的基础实现，支持 Inner Join 和 Left Join。

**影响**: 无破坏性变更。

### 3. Planner 模块

添加了新的 Planner 模块，包括：
- LogicalPlan 定义
- PhysicalPlan trait
- Analyzer 实现
- Executor trait

**影响**: 
- 新增 `planner` 模块，可选择使用
- 不影响现有 API

### 4. 代码质量改进

- 移除了生产代码中的 unwrap/panic
- 使用 expect 提供清晰的错误信息
- 改进了错误处理

**影响**: 无破坏性变更。

---

## 升级步骤

### 1. 更新依赖

```toml
# Cargo.toml
[dependencies]
sqlrustgo = "1.1.0"
```

### 2. 运行测试

```bash
cargo test
```

### 3. 运行性能测试 (可选)

```bash
cargo bench
```

---

## 已知问题

1. 测试覆盖率未达到 90% 目标 (当前 86.22%)
2. planner_bench.rs 暂时移除 (编译问题)

---

## 支持

如有问题，请提交 Issue 或联系维护者。
