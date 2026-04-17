---
entity_type: tool
confidence: 80
domains: [sqlrustgo, optimizer, database]
last_updated: 2026-04-17
---

# Optimizer 模块

> 查询优化器 - 选择最优执行计划

## 概述

Optimizer 模块负责将逻辑计划转换为物理执行计划，选择最优执行路径。

## 源码位置

```
crates/optimizer/
├── src/
│   ├── lib.rs
│   ├── rules.rs        # 优化规则
│   ├── cost.rs         # 成本模型
│   ├── stats.rs        # 统计信息
│   └── context.rs      # 优化上下文
└── Cargo.toml
```

## 优化器类型

### DefaultOptimizer

基于规则的优化器 (RBO)。

```rust
pub struct DefaultOptimizer {
    rules: Vec<Box<dyn OptimizationRule>>,
}
```

### CostBasedOptimizer (CBO)

基于成本的优化器 (CBO)，根据统计信息选择最优计划。

**当前状态**: ⚠️ 已实现但未集成 (Issue #1497)

## 优化规则

| 规则 | 状态 | 说明 |
|------|------|------|
| Predicate Pushdown | ✅ | 谓词下推 |
| Projection Pushdown | ✅ | 列剪裁 |
| Constant Folding | ✅ | 常量折叠 |
| Join Reordering | ✅ | 连接重排 |
| Index Selection | ❌ | 未启用 |

## 成本模型

```rust
pub struct CostModel {
    seq_scan_cost: f64,
    index_scan_cost: f64,
    hash_join_cost: f64,
    sort_merge_join_cost: f64,
}
```

**当前状态**: 成本模型已定义，但未被调用

## Planner 集成状态

文件: `crates/planner/src/planner.rs`

```rust
pub struct DefaultPlanner {
    optimizer: DefaultOptimizer,
    noop_optimizer: NoOpOptimizer,
    use_noop: bool,  // 由 SQLRUSTGO_TEACHING_MODE 控制
}
```

**问题**: 
1. `DefaultOptimizer` 已创建但从未调用 `.optimize()`
2. 始终使用默认计划，未选择最优路径

## 测试

```bash
cargo test --package sqlrustgo-optimizer
cargo test --test optimizer_rules_test
```

## 已知问题 (Issue #1497)

1. **CBO 未集成** - 成本模型存在但未被调用
2. **索引选择未实现** - `should_use_index() = false`
3. **统计信息缺失** - 无表/列统计

## 相关文件

- [Planner 模块](./Planner.md) - 调用优化器
- [Executor 模块](./Executor.md) - 接收优化后的计划

---

*最后更新: 2026-04-17*
