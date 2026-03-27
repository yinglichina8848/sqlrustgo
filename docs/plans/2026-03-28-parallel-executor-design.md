# ParallelExecutor 设计文档

## 概述

ParallelExecutor 是并行查询框架的核心组件，为 Issue #976，实现 VolcanoExecutor 的并行化封装。

## 技术方案

### 实现选择
- **Wrapper 模式** - 封装现有 VolcanoExecutor，最小侵入

### 架构

```
ParallelExecutor (pub trait)
    │
    └── ParallelVolcanoExecutor (struct)
            ├── scheduler: Arc<dyn TaskScheduler>
            ├── parallel_degree: usize
            │
            ├── wrap_scan()      - 并行化 SeqScan
            ├── wrap_hash_join() - 并行化 HashJoin
            └── wrap_aggregate() - 并行化 Aggregate
```

### 核心 API

```rust
pub trait ParallelExecutor: Send + Sync {
    fn execute_parallel(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult>;
    
    fn set_parallel_degree(&mut self, degree: usize);
    
    fn parallel_degree(&self) -> usize;
}

pub struct ParallelVolcanoExecutor {
    scheduler: Arc<dyn TaskScheduler>,
    parallel_degree: usize,
}
```

## 并行化策略

### Scan 并行
- 按数据文件分片
- 每个分片创建独立任务提交到 TaskScheduler
- 结果合并

### HashJoin 并行
- 分桶并行构建
- 分桶并行探测
- 结果合并

### 并行度控制
- parallel_degree 全局配置
- 动态可调

## 依赖

- 依赖 Issue #975 TaskScheduler
- 依赖现有 VolcanoExecutor

## 验收标准

- 4 核机器 SELECT COUNT(*) 加速比 > 2
- 并行度可通过 parallel_degree 变量控制