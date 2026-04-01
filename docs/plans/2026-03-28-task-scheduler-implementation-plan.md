# TaskScheduler 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 TaskScheduler trait 和 RayonTaskScheduler，为并行查询框架提供任务调度能力

**Architecture:** 基于 rayon 库实现 work-stealing 线程池，支持任务提交、批量提交、等待、并行度控制

**Tech Stack:** Rust, rayon, executor crate

---

## Task 1: 添加 rayon 依赖

**Files:**
- Modify: `crates/executor/Cargo.toml:17`

**Step 1: 添加 rayon 依赖**

```toml
rayon = "1.10"
```

**Step 2: 验证依赖**

Run: `cd crates/executor && cargo check`
Expected: 成功解析依赖

**Step 3: Commit**

```bash
git add crates/executor/Cargo.toml
git commit -m "chore(executor): add rayon dependency for TaskScheduler"
```

---

## Task 2: 创建 task_scheduler 模块

**Files:**
- Create: `crates/executor/src/task_scheduler.rs`
- Modify: `crates/executor/src/lib.rs:6`

**Step 1: 创建 task_scheduler.rs 文件**

```rust
//! TaskScheduler Module
//! 
//! Provides task scheduling and thread pool management for parallel query execution.

use rayon::ThreadPool;
use std::sync::Arc;

/// TaskScheduler trait - unified interface for task scheduling
pub trait TaskScheduler: Send + Sync {
    /// Submit a single task for execution
    fn submit<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static;

    /// Submit multiple tasks as a batch
    fn submit_batch<I>(&self, tasks: I)
    where
        I: IntoIterator<Item = Box<dyn FnOnce() + Send + 'static>>;

    /// Wait for all submitted tasks to complete
    fn wait(&self);

    /// Set the parallelism degree (number of threads)
    fn set_parallelism(&self, n: usize);

    /// Get current parallelism degree
    fn current_parallelism(&self) -> usize;
}

/// Rayon-based implementation of TaskScheduler
pub struct RayonTaskScheduler {
    pool: Arc<ThreadPool>,
}

impl RayonTaskScheduler {
    /// Create a new RayonTaskScheduler with specified parallelism
    pub fn new(parallelism: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(parallelism)
            .build()
            .expect("Failed to create thread pool");
        
        Self { pool: Arc::new(pool) }
    }

    /// Create with custom rayon configuration
    pub fn with_config(parallelism: usize, stack_size: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(parallelism)
            .stack_size(stack_size)
            .build()
            .expect("Failed to create thread pool");
        
        Self { pool: Arc::new(pool) }
    }
}

impl TaskScheduler for RayonTaskScheduler {
    fn submit<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.spawn(task);
    }

    fn submit_batch<I>(&self, tasks: I)
    where
        I: IntoIterator<Item = Box<dyn FnOnce() + Send + 'static>>,
    {
        for task in tasks {
            self.pool.spawn(task);
        }
    }

    fn wait(&self) {
        self.pool.wait();
    }

    fn set_parallelism(&self, n: usize) {
        // Rayon doesn't support runtime parallelism change directly
        // Log a warning or use a wrapper pattern
        log::warn!("Rayon does not support runtime parallelism change");
    }

    fn current_parallelism(&self) -> usize {
        self.pool.current_num_threads()
    }
}

/// Create a default TaskScheduler with optimal parallelism
pub fn create_default_scheduler() -> impl TaskScheduler {
    let parallelism = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    RayonTaskScheduler::new(parallelism)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_scheduler_creation() {
        let scheduler = RayonTaskScheduler::new(4);
        assert_eq!(scheduler.current_parallelism(), 4);
    }

    #[test]
    fn test_task_submission() {
        let scheduler = RayonTaskScheduler::new(2);
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        scheduler.submit(move || {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        
        scheduler.wait();
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn test_batch_submission() {
        let scheduler = RayonTaskScheduler::new(4);
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        
        let tasks: Vec<Box<dyn FnOnce() + Send + 'static>> = (0..10)
            .map(|_| {
                let c = counter.clone();
                Box::new(move || {
                    c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }) as Box<dyn FnOnce() + Send + 'static>
            })
            .collect();
        
        scheduler.submit_batch(tasks);
        scheduler.wait();
        
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 10);
    }

    #[test]
    fn test_create_default_scheduler() {
        let scheduler = create_default_scheduler();
        assert!(scheduler.current_parallelism() >= 1);
    }

    #[test]
    fn test_parallel_execution() {
        use std::time::Instant;
        
        let scheduler = RayonTaskScheduler::new(4);
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        
        let start = Instant::now();
        
        for _ in 0..1000 {
            let c = counter.clone();
            scheduler.submit(move || {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            });
        }
        
        scheduler.wait();
        let elapsed = start.elapsed();
        
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1000);
        // Should complete in reasonable time (5s target)
        assert!(elapsed.as_secs() < 5, "Took {}s, should be < 5s", elapsed.as_secs());
    }
}
```

**Step 2: 在 lib.rs 中导出模块**

在 `crates/executor/src/lib.rs` 添加:

```rust
pub mod task_scheduler;
pub use task_scheduler::{create_default_scheduler, RayonTaskScheduler, TaskScheduler};
```

**Step 3: 验证编译**

Run: `cd crates/executor && cargo test task_scheduler --no-fail-fast`
Expected: 所有测试通过

**Step 4: Commit**

```bash
git add crates/executor/src/task_scheduler.rs crates/executor/src/lib.rs
git commit -m "feat(executor): add TaskScheduler with Rayon implementation"
```

---

## Task 3: 更新 MockStorage (可选)

**Files:**
- Modify: `crates/executor/src/mock_storage.rs` (如需要)

如果 StorageEngine trait 需要添加 TaskScheduler 相关的默认实现，此任务为可选。

---

## Task 4: 验证 Issue #975 验收标准

**Files:**
- Test: `crates/executor/src/task_scheduler.rs`

**Step 1: 运行性能测试**

Run: `cd crates/executor && cargo test test_parallel_execution -- --nocapture`
Expected: 1000 任务在 5 秒内完成

**Step 2: 提交最终版本**

```bash
git add -A && git commit -m "feat: complete TaskScheduler implementation (#975)"
```

---

## 执行选项

**1. Subagent-Driven (本会话)** - 每个任务由子代理执行，任务间审查，快速迭代

**2. Parallel Session (单独会话)** - 在新会话中使用 executing-plans，批量执行带检查点

选择哪种方式？