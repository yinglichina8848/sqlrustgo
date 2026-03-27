# TaskScheduler 设计文档

## 概述

TaskScheduler 是并行查询框架的基础组件，为 Issue #975，实现任务调度和线程池管理功能。

## 技术方案

### 实现选择
- **rayon 库** - 成熟的 work-stealing 线程池，自动负载均衡

### 线程管理策略
- **混合模式** - 固定线程 + 动态线程，兼顾延迟和吞吐

### 核心架构

```
TaskScheduler (pub trait)
    │
    └── RayonTaskScheduler (实现)
            ├── submit(task: F)           // 提交单个任务
            ├── submit_batch(tasks)       // 批量提交
            ├── wait()                     // 等待所有任务完成
            ├── set_parallelism(n)         // 动态调整并行度
            └── current_parallelism()      // 获取当前并行度
```

### API 设计

```rust
pub trait TaskScheduler: Send + Sync {
    fn submit<F>(&self, task: F)
    where F: Fn() + Send + 'static;

    fn submit_batch<I>(&self, tasks: I)
    where I: IntoIterator<Item = Box<dyn FnOnce() + Send + 'static>>;

    fn wait(&self);

    fn set_parallelism(&self, n: usize);

    fn current_parallelism(&self) -> usize;
}
```

## 验收标准

| 指标 | 目标 | 测试方法 |
|------|------|----------|
| 单线程任务提交延迟 | < 1ms | benchmark 测试 |
| 1000任务4核并行完成 | < 5s | 基准测试 |

## 依赖

- 被 Issue #976 ParallelExecutor 依赖
- 来自 Issue #942 Phase 1