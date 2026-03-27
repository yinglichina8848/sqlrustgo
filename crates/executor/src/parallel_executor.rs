//! ParallelExecutor Module
//!
//! Provides parallel execution wrapper for VolcanoExecutor.

use crate::task_scheduler::{RayonTaskScheduler, TaskScheduler};
use crate::ExecutorResult;
use sqlrustgo_types::SqlResult;
use std::sync::Arc;

/// ParallelExecutor trait - unified interface for parallel execution
pub trait ParallelExecutor: Send + Sync {
    /// Execute a plan in parallel
    fn execute_parallel(
        &self,
        plan: &dyn sqlrustgo_planner::PhysicalPlan,
    ) -> SqlResult<ExecutorResult>;

    /// Set parallel degree
    fn set_parallel_degree(&mut self, degree: usize);

    /// Get current parallel degree
    fn parallel_degree(&self) -> usize;
}

/// ParallelVolcanoExecutor - wrapper for VolcanoExecutor with parallel execution
pub struct ParallelVolcanoExecutor {
    scheduler: Arc<RayonTaskScheduler>,
    parallel_degree: usize,
}

impl ParallelVolcanoExecutor {
    /// Create a new ParallelVolcanoExecutor with default scheduler
    pub fn new() -> Self {
        let scheduler = RayonTaskScheduler::new(
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
        );
        let parallel_degree = scheduler.current_parallelism();
        Self {
            scheduler: Arc::new(scheduler),
            parallel_degree,
        }
    }

    /// Create with custom scheduler
    pub fn with_scheduler(scheduler: Arc<RayonTaskScheduler>) -> Self {
        let parallel_degree = scheduler.current_parallelism();
        Self {
            scheduler,
            parallel_degree,
        }
    }

    /// Create with custom scheduler and parallel degree
    pub fn with_config(scheduler: Arc<RayonTaskScheduler>, parallel_degree: usize) -> Self {
        Self {
            scheduler,
            parallel_degree,
        }
    }

    /// Get the scheduler
    pub fn scheduler(&self) -> &Arc<RayonTaskScheduler> {
        &self.scheduler
    }

    /// Get parallel degree
    pub fn degree(&self) -> usize {
        self.parallel_degree
    }
}

impl Default for ParallelVolcanoExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallelExecutor for ParallelVolcanoExecutor {
    fn execute_parallel(
        &self,
        _plan: &dyn sqlrustgo_planner::PhysicalPlan,
    ) -> SqlResult<ExecutorResult> {
        Ok(ExecutorResult::new(vec![], 0))
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

    #[test]
    fn test_scheduler_access() {
        let scheduler = Arc::new(RayonTaskScheduler::new(4));
        let executor = ParallelVolcanoExecutor::with_scheduler(scheduler);
        let _ = executor.scheduler();
    }
}
