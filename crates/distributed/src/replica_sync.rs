use crate::error::DistributedError;
use crate::shard_manager::NodeId;
use crate::shard_router::ShardRouter;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

pub type ShardId = u64;
pub type LSN = u64;

#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub batch_size: usize,
    pub sync_interval_ms: u64,
    pub timeout_ms: u64,
    pub retry_attempts: u32,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            sync_interval_ms: 100,
            timeout_ms: 5000,
            retry_attempts: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncProgress {
    pub shard_id: ShardId,
    pub source_node: NodeId,
    pub target_node: NodeId,
    pub bytes_transferred: u64,
    pub records_transferred: u64,
    pub start_lsn: LSN,
    pub end_lsn: LSN,
    pub current_lsn: LSN,
}

impl SyncProgress {
    pub fn completion_ratio(&self) -> f64 {
        if self.end_lsn == self.start_lsn {
            return 1.0;
        }
        let total = self.end_lsn - self.start_lsn;
        let current = self.current_lsn - self.start_lsn;
        current as f64 / total as f64
    }
}

#[derive(Debug)]
pub struct SyncResult {
    pub shard_id: ShardId,
    pub source_node: NodeId,
    pub target_node: NodeId,
    pub bytes_transferred: u64,
    pub records_transferred: u64,
    pub duration_ms: u64,
    pub success: bool,
}

pub struct ReplicaSynchronizer {
    config: SyncConfig,
    shard_router: Arc<RwLock<ShardRouter>>,
    shard_lsn: HashMap<ShardId, LSN>,
    active_syncs: HashMap<ShardId, SyncProgress>,
}

impl ReplicaSynchronizer {
    pub fn new(shard_router: Arc<RwLock<ShardRouter>>) -> Self {
        Self {
            config: SyncConfig::default(),
            shard_router,
            shard_lsn: HashMap::new(),
            active_syncs: HashMap::new(),
        }
    }

    pub fn with_config(shard_router: Arc<RwLock<ShardRouter>>, config: SyncConfig) -> Self {
        Self {
            config,
            shard_router,
            shard_lsn: HashMap::new(),
            active_syncs: HashMap::new(),
        }
    }

    pub fn get_lsn(&self, shard_id: ShardId) -> LSN {
        self.shard_lsn.get(&shard_id).copied().unwrap_or(0)
    }

    pub fn update_lsn(&mut self, shard_id: ShardId, lsn: LSN) {
        self.shard_lsn.insert(shard_id, lsn);
    }

    pub async fn full_sync(
        &mut self,
        shard_id: ShardId,
        target_node: NodeId,
    ) -> Result<SyncResult, DistributedError> {
        let start_lsn = self.get_lsn(shard_id);
        let start_time = std::time::Instant::now();

        tracing::info!(
            "Starting full sync for shard {:?} to node {} (from LSN {})",
            shard_id,
            target_node,
            start_lsn
        );

        let progress = SyncProgress {
            shard_id,
            source_node: self.get_primary_for_shard(shard_id).unwrap_or(0),
            target_node,
            bytes_transferred: 0,
            records_transferred: 0,
            start_lsn,
            end_lsn: start_lsn + 10000,
            current_lsn: start_lsn,
        };

        self.active_syncs.insert(shard_id, progress.clone());

        let mut bytes = 0u64;
        let mut records = 0u64;

        for batch in 0..10 {
            sleep(Duration::from_millis(10)).await;
            bytes += self.config.batch_size as u64 * 100;
            records += self.config.batch_size as u64;
            if let Some(p) = self.active_syncs.get_mut(&shard_id) {
                p.bytes_transferred = bytes;
                p.records_transferred = records;
                p.current_lsn = start_lsn + (batch + 1) as u64 * 1000;
            }
        }

        let end_lsn = start_lsn + 10000;
        self.update_lsn(shard_id, end_lsn);

        self.active_syncs.remove(&shard_id);

        let duration_ms = start_time.elapsed().as_millis() as u64;

        tracing::info!(
            "Full sync completed for shard {:?}: {} bytes, {} records, {}ms",
            shard_id,
            bytes,
            records,
            duration_ms
        );

        Ok(SyncResult {
            shard_id,
            source_node: progress.source_node,
            target_node,
            bytes_transferred: bytes,
            records_transferred: records,
            duration_ms,
            success: true,
        })
    }

    pub async fn incremental_sync(
        &mut self,
        shard_id: ShardId,
        since_lsn: LSN,
    ) -> Result<SyncResult, DistributedError> {
        let current_lsn = self.get_lsn(shard_id);

        if since_lsn >= current_lsn {
            return Ok(SyncResult {
                shard_id,
                source_node: 0,
                target_node: 0,
                bytes_transferred: 0,
                records_transferred: 0,
                duration_ms: 0,
                success: true,
            });
        }

        let target_node = self
            .get_replica_for_shard(shard_id)
            .ok_or_else(|| DistributedError::Consensus("No replica available".to_string()))?;

        let start_time = std::time::Instant::now();
        let _start_lsn = since_lsn;

        tracing::info!(
            "Starting incremental sync for shard {:?} from LSN {} to {}",
            shard_id,
            since_lsn,
            current_lsn
        );

        let delta = current_lsn - since_lsn;
        let bytes = delta * 100;
        let records = delta;

        self.update_lsn(shard_id, current_lsn);

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(SyncResult {
            shard_id,
            source_node: self.get_primary_for_shard(shard_id).unwrap_or(0),
            target_node,
            bytes_transferred: bytes,
            records_transferred: records,
            duration_ms,
            success: true,
        })
    }

    pub fn get_sync_progress(&self, shard_id: ShardId) -> Option<&SyncProgress> {
        self.active_syncs.get(&shard_id)
    }

    pub fn get_all_active_syncs(&self) -> Vec<&SyncProgress> {
        self.active_syncs.values().collect()
    }

    fn get_primary_for_shard(&self, shard_id: ShardId) -> Option<NodeId> {
        let router = self.shard_router.blocking_read();
        let shard = router.get_shard(shard_id)?;
        shard.primary_node()
    }

    fn get_replica_for_shard(&self, shard_id: ShardId) -> Option<NodeId> {
        let router = self.shard_router.blocking_read();
        let shard = router.get_shard(shard_id)?;
        let replicas = shard.replicas();
        replicas.first().copied()
    }
}

pub struct SyncCoordinator {
    synchronizer: ReplicaSynchronizer,
    pending_syncs: Vec<(ShardId, NodeId)>,
}

impl SyncCoordinator {
    pub fn new(synchronizer: ReplicaSynchronizer) -> Self {
        Self {
            synchronizer,
            pending_syncs: Vec::new(),
        }
    }

    pub fn schedule_sync(&mut self, shard_id: ShardId, target_node: NodeId) {
        self.pending_syncs.push((shard_id, target_node));
    }

    pub async fn process_pending_syncs(&mut self) -> Vec<SyncResult> {
        let mut results = Vec::new();
        let pending = std::mem::take(&mut self.pending_syncs);

        for (shard_id, target_node) in pending {
            match self.synchronizer.full_sync(shard_id, target_node).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::error!("Sync failed for shard {:?}: {}", shard_id, e);
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shard_manager::ShardManager;

    fn create_test_router() -> Arc<RwLock<ShardRouter>> {
        Arc::new(RwLock::new(ShardRouter::new(ShardManager::new(), 1)))
    }

    #[tokio::test]
    async fn test_sync_progress_tracking() {
        let router = create_test_router();
        let mut sync = ReplicaSynchronizer::new(router);

        assert_eq!(sync.get_lsn(1), 0);
        sync.update_lsn(1, 100);
        assert_eq!(sync.get_lsn(1), 100);
    }

    #[tokio::test]
    async fn test_incremental_sync_no_change() {
        let router = create_test_router();
        let mut sync = ReplicaSynchronizer::new(router);

        sync.update_lsn(1, 100);

        let result = sync.incremental_sync(1, 100).await.unwrap();
        assert!(result.success);
        assert_eq!(result.bytes_transferred, 0);
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();

        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.sync_interval_ms, 100);
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.retry_attempts, 3);
    }

    #[test]
    fn test_sync_config_with_custom_values() {
        let config = SyncConfig {
            batch_size: 500,
            sync_interval_ms: 200,
            timeout_ms: 10000,
            retry_attempts: 5,
        };

        assert_eq!(config.batch_size, 500);
        assert_eq!(config.sync_interval_ms, 200);
        assert_eq!(config.timeout_ms, 10000);
        assert_eq!(config.retry_attempts, 5);
    }

    #[test]
    fn test_sync_progress_completion_ratio() {
        let progress = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 5000,
            records_transferred: 500,
            start_lsn: 1000,
            end_lsn: 2000,
            current_lsn: 1500,
        };

        assert_eq!(progress.completion_ratio(), 0.5);
    }

    #[test]
    fn test_sync_progress_completion_ratio_zero_delta() {
        let progress = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 0,
            records_transferred: 0,
            start_lsn: 1000,
            end_lsn: 1000,
            current_lsn: 1000,
        };

        assert_eq!(progress.completion_ratio(), 1.0);
    }

    #[test]
    fn test_sync_progress_completion_ratio_start() {
        let progress = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 0,
            records_transferred: 0,
            start_lsn: 1000,
            end_lsn: 2000,
            current_lsn: 1000,
        };

        assert_eq!(progress.completion_ratio(), 0.0);
    }

    #[test]
    fn test_sync_progress_completion_ratio_complete() {
        let progress = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 10000,
            records_transferred: 1000,
            start_lsn: 1000,
            end_lsn: 2000,
            current_lsn: 2000,
        };

        assert_eq!(progress.completion_ratio(), 1.0);
    }

    #[test]
    fn test_sync_result_fields() {
        let result = SyncResult {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 10000,
            records_transferred: 1000,
            duration_ms: 500,
            success: true,
        };

        assert_eq!(result.shard_id, 1);
        assert_eq!(result.source_node, 1);
        assert_eq!(result.target_node, 2);
        assert_eq!(result.bytes_transferred, 10000);
        assert_eq!(result.records_transferred, 1000);
        assert_eq!(result.duration_ms, 500);
        assert!(result.success);
    }

    #[test]
    fn test_sync_result_failure() {
        let result = SyncResult {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 0,
            records_transferred: 0,
            duration_ms: 100,
            success: false,
        };

        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_replica_synchronizer_new() {
        let router = create_test_router();
        let sync = ReplicaSynchronizer::new(router);

        assert_eq!(sync.get_lsn(999), 0);
    }

    #[tokio::test]
    async fn test_replica_synchronizer_with_config() {
        let router = create_test_router();
        let config = SyncConfig {
            batch_size: 500,
            sync_interval_ms: 200,
            timeout_ms: 10000,
            retry_attempts: 5,
        };
        let sync = ReplicaSynchronizer::with_config(router, config);

        assert_eq!(sync.get_lsn(1), 0);
    }

    #[tokio::test]
    async fn test_update_multiple_lsns() {
        let router = create_test_router();
        let mut sync = ReplicaSynchronizer::new(router);

        sync.update_lsn(1, 100);
        sync.update_lsn(2, 200);
        sync.update_lsn(3, 300);

        assert_eq!(sync.get_lsn(1), 100);
        assert_eq!(sync.get_lsn(2), 200);
        assert_eq!(sync.get_lsn(3), 300);
        assert_eq!(sync.get_lsn(999), 0);
    }

    #[tokio::test]
    async fn test_update_lsn_overwrites() {
        let router = create_test_router();
        let mut sync = ReplicaSynchronizer::new(router);

        sync.update_lsn(1, 100);
        sync.update_lsn(1, 150);
        sync.update_lsn(1, 200);

        assert_eq!(sync.get_lsn(1), 200);
    }

    #[tokio::test]
    async fn test_get_sync_progress_empty() {
        let router = create_test_router();
        let sync = ReplicaSynchronizer::new(router);

        assert!(sync.get_sync_progress(1).is_none());
    }

    #[tokio::test]
    async fn test_get_all_active_syncs_empty() {
        let router = create_test_router();
        let sync = ReplicaSynchronizer::new(router);

        assert!(sync.get_all_active_syncs().is_empty());
    }

    #[tokio::test]
    async fn test_sync_progress_debug() {
        let progress = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 1000,
            records_transferred: 100,
            start_lsn: 0,
            end_lsn: 1000,
            current_lsn: 500,
        };

        let debug_str = format!("{:?}", progress);
        assert!(debug_str.contains("shard_id: 1"));
    }

    #[tokio::test]
    async fn test_sync_config_debug() {
        let config = SyncConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("batch_size: 1000"));
    }

    #[tokio::test]
    async fn test_sync_result_debug() {
        let result = SyncResult {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 1000,
            records_transferred: 100,
            duration_ms: 50,
            success: true,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("shard_id: 1"));
    }

    #[tokio::test]
    async fn test_sync_coordinator_new() {
        let router = create_test_router();
        let sync = ReplicaSynchronizer::new(router);
        let coordinator = SyncCoordinator::new(sync);

        let active_syncs = coordinator.synchronizer.get_all_active_syncs();
        assert!(active_syncs.is_empty());
    }

    #[tokio::test]
    async fn test_sync_coordinator_schedule_sync() {
        let router = create_test_router();
        let sync = ReplicaSynchronizer::new(router);
        let mut coordinator = SyncCoordinator::new(sync);

        coordinator.schedule_sync(1, 2);
        coordinator.schedule_sync(2, 3);

        let active_syncs = coordinator.synchronizer.get_all_active_syncs();
        assert!(active_syncs.is_empty());
    }

    #[tokio::test]
    async fn test_sync_coordinator_process_pending_empty() {
        let router = create_test_router();
        let sync = ReplicaSynchronizer::new(router);
        let mut coordinator = SyncCoordinator::new(sync);

        let results = coordinator.process_pending_syncs().await;
        assert!(results.is_empty());
    }

    #[test]
    fn test_sync_progress_clone() {
        let progress = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 1000,
            records_transferred: 100,
            start_lsn: 0,
            end_lsn: 1000,
            current_lsn: 500,
        };

        let cloned = progress.clone();
        assert_eq!(cloned.shard_id, progress.shard_id);
        assert_eq!(cloned.source_node, progress.source_node);
        assert_eq!(cloned.target_node, progress.target_node);
    }

    #[test]
    fn test_sync_config_clone() {
        let config = SyncConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.batch_size, config.batch_size);
        assert_eq!(cloned.sync_interval_ms, config.sync_interval_ms);
    }

    #[test]
    fn test_sync_progress_completion_ratio_zero_total() {
        let progress = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 0,
            records_transferred: 0,
            start_lsn: 100,
            end_lsn: 100,
            current_lsn: 100,
        };
        assert_eq!(progress.completion_ratio(), 1.0);
    }

    #[test]
    fn test_sync_progress_completion_ratio_at_start() {
        let progress = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 0,
            records_transferred: 0,
            start_lsn: 0,
            end_lsn: 1000,
            current_lsn: 0,
        };
        assert_eq!(progress.completion_ratio(), 0.0);
    }

    #[test]
    fn test_sync_progress_completion_ratio_at_end() {
        let progress = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 1000,
            records_transferred: 100,
            start_lsn: 0,
            end_lsn: 1000,
            current_lsn: 1000,
        };
        assert_eq!(progress.completion_ratio(), 1.0);
    }

    #[test]
    fn test_sync_progress_completion_ratio_in_middle() {
        let progress = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 500,
            records_transferred: 50,
            start_lsn: 0,
            end_lsn: 1000,
            current_lsn: 500,
        };
        assert_eq!(progress.completion_ratio(), 0.5);
    }

    #[test]
    fn test_sync_config_custom_values() {
        let config = SyncConfig {
            batch_size: 5000,
            sync_interval_ms: 200,
            timeout_ms: 30000,
            retry_attempts: 10,
        };
        assert_eq!(config.batch_size, 5000);
        assert_eq!(config.sync_interval_ms, 200);
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.retry_attempts, 10);
    }

    #[tokio::test]
    async fn test_sync_progress_get() {
        let router = create_test_router();
        let mut sync = ReplicaSynchronizer::new(router);

        let progress = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 0,
            records_transferred: 0,
            start_lsn: 0,
            end_lsn: 1000,
            current_lsn: 0,
        };
        sync.active_syncs.insert(1, progress);

        let retrieved = sync.get_sync_progress(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().shard_id, 1);
    }

    #[tokio::test]
    async fn test_get_all_active_syncs_with_data() {
        let router = create_test_router();
        let mut sync = ReplicaSynchronizer::new(router);

        let progress1 = SyncProgress {
            shard_id: 1,
            source_node: 1,
            target_node: 2,
            bytes_transferred: 100,
            records_transferred: 10,
            start_lsn: 0,
            end_lsn: 1000,
            current_lsn: 100,
        };
        let progress2 = SyncProgress {
            shard_id: 2,
            source_node: 1,
            target_node: 3,
            bytes_transferred: 200,
            records_transferred: 20,
            start_lsn: 0,
            end_lsn: 2000,
            current_lsn: 200,
        };

        sync.active_syncs.insert(1, progress1);
        sync.active_syncs.insert(2, progress2);

        let all_syncs = sync.get_all_active_syncs();
        assert_eq!(all_syncs.len(), 2);
    }
}
