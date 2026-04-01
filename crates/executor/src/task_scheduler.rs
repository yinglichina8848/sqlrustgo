//! TaskScheduler Module
//!
//! Provides task scheduling and thread pool management for parallel query execution.

use rayon::ThreadPool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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
    active_tasks: Arc<AtomicUsize>,
}

impl RayonTaskScheduler {
    /// Create a new RayonTaskScheduler with specified parallelism
    pub fn new(parallelism: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(parallelism)
            .build()
            .expect("Failed to create thread pool");

        Self {
            pool: Arc::new(pool),
            active_tasks: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create with custom rayon configuration
    pub fn with_config(parallelism: usize, stack_size: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(parallelism)
            .stack_size(stack_size)
            .build()
            .expect("Failed to create thread pool");

        Self {
            pool: Arc::new(pool),
            active_tasks: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl TaskScheduler for RayonTaskScheduler {
    fn submit<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        let active = self.active_tasks.clone();
        self.pool.spawn(move || {
            task();
            active.fetch_sub(1, Ordering::SeqCst);
        });
    }

    fn submit_batch<I>(&self, tasks: I)
    where
        I: IntoIterator<Item = Box<dyn FnOnce() + Send + 'static>>,
    {
        for task in tasks {
            self.active_tasks.fetch_add(1, Ordering::SeqCst);
            let active = self.active_tasks.clone();
            self.pool.spawn(move || {
                task();
                active.fetch_sub(1, Ordering::SeqCst);
            });
        }
    }

    fn wait(&self) {
        while self.active_tasks.load(Ordering::SeqCst) > 0 {
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn set_parallelism(&self, _n: usize) {
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
        assert!(
            elapsed.as_secs() < 5,
            "Took {}s, should be < 5s",
            elapsed.as_secs()
        );
    }
}
