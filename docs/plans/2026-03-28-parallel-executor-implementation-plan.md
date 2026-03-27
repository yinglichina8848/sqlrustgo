# ParallelExecutor 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 ParallelExecutor trait 和 ParallelVolcanoExecutor，为 VolcanoExecutor 提供并行执行能力

**Architecture:** Wrapper 模式封装现有 VolcanoExecutor，基于 TaskScheduler 实现任务并行调度

**Tech Stack:** Rust, TaskScheduler, VolcanoExecutor, executor crate

---

## Task 1: 创建 parallel_executor 模块

**Files:**
- Create: `crates/executor/src/parallel_executor.rs`
- Modify: `crates/executor/src/lib.rs`

**Step 1: 创建 parallel_executor.rs 文件**

```rust
//! ParallelExecutor Module
//!
//! Provides parallel execution wrapper for VolcanoExecutor.

use crate::task_scheduler::{create_default_scheduler, TaskScheduler};
use crate::{ExecutorResult, PhysicalPlan, SqlError, SqlResult, VolcanoExecutor};
use std::sync::Arc;

/// ParallelExecutor trait - unified interface for parallel execution
pub trait ParallelExecutor: Send + Sync {
    /// Execute a plan in parallel
    fn execute_parallel(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult>;

    /// Set parallel degree
    fn set_parallel_degree(&mut self, degree: usize);

    /// Get current parallel degree
    fn parallel_degree(&self) -> usize;
}

/// ParallelVolcanoExecutor - wrapper for VolcanoExecutor with parallel execution
pub struct ParallelVolcanoExecutor {
    scheduler: Arc<dyn TaskScheduler>,
    parallel_degree: usize,
}

impl ParallelVolcanoExecutor {
    /// Create a new ParallelVolcanoExecutor with default scheduler
    pub fn new() -> Self {
        Self {
            scheduler: Arc::new(create_default_scheduler()),
            parallel_degree: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
        }
    }

    /// Create with custom scheduler
    pub fn with_scheduler(scheduler: Arc<dyn TaskScheduler>) -> Self {
        let parallel_degree = scheduler.current_parallelism();
        Self {
            scheduler,
            parallel_degree,
        }
    }

    /// Create with custom scheduler and parallel degree
    pub fn with_config(scheduler: Arc<dyn TaskScheduler>, parallel_degree: usize) -> Self {
        Self {
            scheduler,
            parallel_degree,
        }
    }

    /// Execute a physical plan in parallel
    pub fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let parallel_degree = self.parallel_degree;

        if parallel_degree <= 1 {
            return self.execute_sequential(plan);
        }

        match plan.name() {
            "SeqScan" => self.execute_parallel_scan(plan),
            "HashJoin" => self.execute_parallel_hash_join(plan),
            _ => self.execute_sequential(plan),
        }
    }

    fn execute_sequential(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        crate::executor::execute_collect(&mut (*self.build_executor(plan)?))
    }

    fn build_executor(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let mut executor_builder = crate::executor::VolcanoExecutorBuilder::new();
        executor_builder.build(plan)
    }

    fn execute_parallel_scan(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let scheduler = self.scheduler.clone();
        let plan = plan.clone_any();
        let parallel_degree = self.parallel_degree;

        let results = Arc::new(std::sync::Mutex::new(Vec::new()));

        for _ in 0..parallel_degree {
            let results_clone = results.clone();
            let plan_clone = plan.clone_any();
            scheduler.submit(move || {
                let mut executor_builder = crate::executor::VolcanoExecutorBuilder::new();
                if let Ok(mut executor) = executor_builder.build(&*plan_clone) {
                    if let Ok(result) = crate::executor::execute_collect(&mut *executor) {
                        let mut guard = results_clone.lock().unwrap();
                        guard.push(result);
                    }
                }
            });
        }

        scheduler.wait();

        let results = results.lock().unwrap();
        let mut combined = ExecutorResult::new(vec![], 0);
        for result in results.iter() {
            combined.rows.extend(result.rows.clone());
            combined.affected_rows += result.affected_rows;
        }

        Ok(combined)
    }

    fn execute_parallel_hash_join(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        self.execute_sequential(plan)
    }

    fn clone_any(&self) -> Box<dyn std::any::Any + Send + Sync> {
        Box::new(())
    }
}

impl Default for ParallelVolcanoExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallelExecutor for ParallelVolcanoExecutor {
    fn execute_parallel(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        self.execute(plan)
    }

    fn set_parallel_degree(&mut self, degree: usize) {
        self.parallel_degree = degree.max(1);
    }

    fn parallel_degree(&self) -> usize {
        self.parallel_degree
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_scheduler::RayonTaskScheduler;

    #[test]
    fn test_parallel_executor_creation() {
        let executor = ParallelVolcanoExecutor::new();
        assert!(executor.parallel_degree() >= 1);
    }

    #[test]
    fn test_parallel_degree_set() {
        let mut executor = ParallelVolcanoExecutor::new();
        executor.set_parallel_degree(8);
        assert_eq!(executor.parallel_degree(), 8);
    }

    #[test]
    fn test_parallel_degree_minimum() {
        let mut executor = ParallelVolcanoExecutor::new();
        executor.set_parallel_degree(0);
        assert_eq!(executor.parallel_degree(), 1);
    }

    #[test]
    fn test_with_custom_scheduler() {
        let scheduler = Arc::new(RayonTaskScheduler::new(4));
        let executor = ParallelVolcanoExecutor::with_scheduler(scheduler);
        assert_eq!(executor.parallel_degree(), 4);
    }

    #[test]
    fn test_with_config() {
        let scheduler = Arc::new(RayonTaskScheduler::new(4));
        let executor = ParallelVolcanoExecutor::with_config(scheduler, 8);
        assert_eq!(executor.parallel_degree(), 8);
    }
}
```

**Step 2: 在 lib.rs 中导出模块**

在 `crates/executor/src/lib.rs` 添加:

```rust
pub mod parallel_executor;
pub use parallel_executor::{ParallelExecutor, ParallelVolcanoExecutor};
```

**Step 3: 验证编译**

Run: `cd crates/executor && cargo test parallel_executor --no-fail-fast`
Expected: 所有测试通过

**Step 4: Commit**

```bash
git add crates/executor/src/parallel_executor.rs crates/executor/src/lib.rs
git commit -m "feat(executor): add ParallelExecutor with parallel execution"
```

---

## Task 2: 实现 Scan 并行化

**Files:**
- Modify: `crates/executor/src/parallel_executor.rs`

扩展 parallel_executor.rs，实现 execute_parallel_scan 方法。

**Step 1: 添加数据分片逻辑**

添加以下功能：
- 从 StorageEngine 获取数据分片信息
- 按分片创建并行任务
- 结果合并

**Step 2: 测试**

Run: `cd crates/executor && cargo test parallel_scan --no-fail-fast`
Expected: 测试通过

**Step 3: Commit**

```bash
git add crates/executor/src/parallel_executor.rs
git commit -m "feat(executor): implement parallel scan execution"
```

---

## Task 3: 实现 HashJoin 并行化

**Files:**
- Modify: `crates/executor/src/parallel_executor.rs`

扩展 parallel_executor.rs，实现 execute_parallel_hash_join 方法。

**Step 1: 添加分桶逻辑**

添加以下功能：
- Hash 分桶策略
- 并行构建阶段
- 并行探测阶段

**Step 2: 测试**

Run: `cd crates/executor && cargo test parallel_hash_join --no-fail-fast`
Expected: 测试通过

**Step 3: Commit**

```bash
git add crates/executor/src/parallel_executor.rs
git commit -m "feat(executor): implement parallel hash join execution"
```

---

## Task 4: 性能验证

**Files:**
- Test: `crates/executor/src/parallel_executor.rs`

**Step 1: 运行性能测试**

Run: `cd crates/executor && cargo test test_parallel_speedup -- --nocapture`
Expected: 加速比 > 2 (4核)

**Step 2: 提交最终版本**

```bash
git add -A && git commit -m "feat: complete ParallelExecutor implementation (#976)"
```

---

## 执行选项

**1. Subagent-Driven (本会话)** - 每个任务由子代理执行，任务间审查，快速迭代

**2. Parallel Session (单独会话)** - 在新会话中使用 executing-plans，批量执行带检查点

选择哪种方式？