//! Worker Pool for MTS (Multi-Threaded Slave)
//!
//! Provides a pool of workers for parallel transaction execution.

use tokio::sync::{mpsc, RwLock};

pub struct WorkerPool {
    workers: Vec<Worker>,
    config: WorkerPoolConfig,
    state: RwLock<WorkerPoolState>,
}

#[derive(Debug, Clone)]
pub struct WorkerPoolConfig {
    pub worker_count: usize,
    pub queue_capacity: usize,
    pub worker_thread_name_prefix: String,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            queue_capacity: 1000,
            worker_thread_name_prefix: "mts-worker".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum WorkerPoolState {
    #[default]
    Stopped,
    Running,
    Paused,
}

pub struct Worker {
    id: usize,
    tx: mpsc::Sender<WorkerMessage>,
}

#[derive(Debug, Clone)]
pub enum WorkerMessage {
    Execute(u64),
    Commit(u64),
    Rollback(u64),
    Stop,
}

impl Worker {
    pub fn id(&self) -> usize {
        self.id
    }

    pub async fn send(
        &self,
        msg: WorkerMessage,
    ) -> Result<(), mpsc::error::SendError<WorkerMessage>> {
        self.tx.send(msg).await
    }
}

impl WorkerPool {
    pub fn new(config: WorkerPoolConfig) -> Self {
        let workers = (0..config.worker_count)
            .map(|id| {
                let (tx, _rx) = mpsc::channel(config.queue_capacity);
                Worker { id, tx }
            })
            .collect();

        Self {
            workers,
            config,
            state: RwLock::new(WorkerPoolState::Stopped),
        }
    }

    pub fn worker_count(&self) -> usize {
        self.config.worker_count
    }

    pub async fn start(&self) {
        *self.state.write().await = WorkerPoolState::Running;
    }

    pub async fn stop(&self) {
        for worker in &self.workers {
            let _ = worker.send(WorkerMessage::Stop).await;
        }
        *self.state.write().await = WorkerPoolState::Stopped;
    }

    pub async fn pause(&self) {
        *self.state.write().await = WorkerPoolState::Paused;
    }

    pub async fn resume(&self) {
        *self.state.write().await = WorkerPoolState::Running;
    }

    pub async fn get_state(&self) -> WorkerPoolState {
        self.state.read().await.clone()
    }

    pub fn get_worker(&self, id: usize) -> Option<&Worker> {
        self.workers.get(id)
    }

    pub fn get_all_workers(&self) -> &[Worker] {
        &self.workers
    }

    pub async fn broadcast(
        &self,
        msg: WorkerMessage,
    ) -> Vec<Result<(), mpsc::error::SendError<WorkerMessage>>> {
        let mut results = Vec::with_capacity(self.workers.len());
        for worker in &self.workers {
            results.push(worker.send(msg.clone()).await);
        }
        results
    }

    pub fn num_workers(&self) -> usize {
        self.workers.len()
    }

    pub fn config(&self) -> &WorkerPoolConfig {
        &self.config
    }
}

#[derive(Debug, Clone, Default)]
pub struct WorkerPoolStats {
    pub active_workers: usize,
    pub total_executed: u64,
    pub total_committed: u64,
    pub total_rolled_back: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_pool_config_default() {
        let config = WorkerPoolConfig::default();
        assert_eq!(config.worker_count, 4);
        assert_eq!(config.queue_capacity, 1000);
    }

    #[test]
    fn test_worker_pool_new() {
        let config = WorkerPoolConfig::default();
        let pool = WorkerPool::new(config);

        assert_eq!(pool.num_workers(), 4);
        assert_eq!(pool.worker_count(), 4);
    }

    #[tokio::test]
    async fn test_worker_pool_start_stop() {
        let config = WorkerPoolConfig::default();
        let pool = WorkerPool::new(config);

        pool.start().await;
        assert_eq!(pool.get_state().await, WorkerPoolState::Running);

        pool.stop().await;
        assert_eq!(pool.get_state().await, WorkerPoolState::Stopped);
    }

    #[tokio::test]
    async fn test_worker_pool_pause_resume() {
        let config = WorkerPoolConfig::default();
        let pool = WorkerPool::new(config);

        pool.start().await;
        pool.pause().await;
        assert_eq!(pool.get_state().await, WorkerPoolState::Paused);

        pool.resume().await;
        assert_eq!(pool.get_state().await, WorkerPoolState::Running);
    }

    #[test]
    fn test_worker_id() {
        let config = WorkerPoolConfig::default();
        let pool = WorkerPool::new(config);

        for i in 0..pool.num_workers() {
            let worker = pool.get_worker(i).unwrap();
            assert_eq!(worker.id(), i);
        }
    }

    #[test]
    fn test_worker_pool_get_all_workers() {
        let config = WorkerPoolConfig {
            worker_count: 8,
            ..Default::default()
        };
        let pool = WorkerPool::new(config);

        assert_eq!(pool.get_all_workers().len(), 8);
    }

    #[tokio::test]
    async fn test_worker_send() {
        let config = WorkerPoolConfig::default();
        let pool = WorkerPool::new(config);

        let worker = pool.get_worker(0).unwrap();
        let result = worker.send(WorkerMessage::Execute(123)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_worker_message_variants() {
        let msg1 = WorkerMessage::Execute(1);
        let msg2 = WorkerMessage::Commit(2);
        let msg3 = WorkerMessage::Rollback(3);
        let msg4 = WorkerMessage::Stop;

        assert!(matches!(msg1, WorkerMessage::Execute(1)));
        assert!(matches!(msg2, WorkerMessage::Commit(2)));
        assert!(matches!(msg3, WorkerMessage::Rollback(3)));
        assert!(matches!(msg4, WorkerMessage::Stop));
    }

    #[tokio::test]
    async fn test_worker_pool_state_default() {
        let config = WorkerPoolConfig::default();
        let pool = WorkerPool::new(config);
        assert_eq!(pool.get_state().await, WorkerPoolState::Stopped);
    }

    #[test]
    fn test_worker_pool_config_custom() {
        let config = WorkerPoolConfig {
            worker_count: 16,
            queue_capacity: 5000,
            worker_thread_name_prefix: "custom-worker".to_string(),
        };
        assert_eq!(config.worker_count, 16);
        assert_eq!(config.queue_capacity, 5000);
    }

    #[test]
    fn test_worker_pool_stats_default() {
        let stats = WorkerPoolStats::default();
        assert_eq!(stats.active_workers, 0);
        assert_eq!(stats.total_executed, 0);
        assert_eq!(stats.total_committed, 0);
        assert_eq!(stats.total_rolled_back, 0);
    }

    #[test]
    fn test_worker_pool_state_variants() {
        assert_eq!(format!("{:?}", WorkerPoolState::Stopped), "Stopped");
        assert_eq!(format!("{:?}", WorkerPoolState::Running), "Running");
        assert_eq!(format!("{:?}", WorkerPoolState::Paused), "Paused");
    }
}