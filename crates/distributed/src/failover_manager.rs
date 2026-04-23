use crate::consensus::ShardReplicaManager;
use crate::error::DistributedError;
use crate::shard_manager::{NodeId, ShardId, ShardManager};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

pub struct FailoverConfig {
    pub election_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub max_replication_lag_ms: u64,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            election_timeout: Duration::from_millis(300),
            heartbeat_interval: Duration::from_millis(50),
            max_replication_lag_ms: 100,
        }
    }
}

pub struct FailoverManager {
    node_id: NodeId,
    shard_manager: Arc<RwLock<ShardManager>>,
    replica_manager: Arc<RwLock<ShardReplicaManager>>,
    config: FailoverConfig,
    dead_nodes: HashSet<NodeId>,
    last_heartbeat: HashMap<NodeId, Instant>,
}

impl FailoverManager {
    pub fn new(
        node_id: NodeId,
        shard_manager: Arc<RwLock<ShardManager>>,
        replica_manager: Arc<RwLock<ShardReplicaManager>>,
    ) -> Self {
        Self {
            node_id,
            shard_manager,
            replica_manager,
            config: FailoverConfig::default(),
            dead_nodes: HashSet::new(),
            last_heartbeat: HashMap::new(),
        }
    }

    pub fn with_config(
        node_id: NodeId,
        shard_manager: Arc<RwLock<ShardManager>>,
        replica_manager: Arc<RwLock<ShardReplicaManager>>,
        config: FailoverConfig,
    ) -> Self {
        Self {
            node_id,
            shard_manager,
            replica_manager,
            config,
            dead_nodes: HashSet::new(),
            last_heartbeat: HashMap::new(),
        }
    }

    pub async fn handle_node_failure(&mut self, node_id: NodeId) -> Result<(), DistributedError> {
        if node_id == self.node_id {
            return Err(DistributedError::Consensus(
                "Cannot handle own failure".to_string(),
            ));
        }

        tracing::info!("Handling node failure for node {}", node_id);
        self.dead_nodes.insert(node_id);

        let affected_shard_ids: Vec<ShardId> = {
            let shard_manager = self.shard_manager.read().await;
            shard_manager
                .get_shards_by_node(node_id)
                .iter()
                .map(|s| s.shard_id)
                .collect()
        };

        for shard_id in affected_shard_ids {
            self.promote_replica_for_shard(shard_id).await?;
        }

        Ok(())
    }

    async fn promote_replica_for_shard(
        &mut self,
        shard_id: ShardId,
    ) -> Result<(), DistributedError> {
        let (current_primary, replicas) = {
            let shard_manager = self.shard_manager.read().await;
            let shard = shard_manager
                .get_shard(shard_id)
                .ok_or(DistributedError::ShardNotFound(shard_id))?;

            let replicas = shard.replica_nodes.clone();
            let current_primary = shard.primary_node();
            (current_primary, replicas)
        };

        if let Some(primary) = current_primary {
            if primary == self.node_id {
                return Ok(());
            }
            if self.dead_nodes.contains(&primary) {
                if let Some(new_primary) = replicas.iter().find(|&&n| !self.dead_nodes.contains(&n))
                {
                    self.do_promote(shard_id, *new_primary).await?;
                }
            }
        }

        Ok(())
    }

    async fn do_promote(
        &mut self,
        shard_id: ShardId,
        new_primary: NodeId,
    ) -> Result<(), DistributedError> {
        tracing::info!(
            "Promoting node {} as new primary for shard {:?}",
            new_primary,
            shard_id
        );

        {
            let mut shard_manager = self.shard_manager.write().await;
            if let Some(shard) = shard_manager.get_shard_mut(shard_id) {
                shard.promote_replica(new_primary);
            }
        }

        {
            let mut replica_manager = self.replica_manager.write().await;
            if new_primary == self.node_id {
                replica_manager.become_leader(shard_id)?;
            }
        }

        Ok(())
    }

    pub async fn check_node_health(&mut self, node_id: NodeId) -> bool {
        if self.dead_nodes.contains(&node_id) {
            return false;
        }

        if let Some(last) = self.last_heartbeat.get(&node_id) {
            return last.elapsed() < self.config.election_timeout;
        }

        true
    }

    pub async fn record_heartbeat(&mut self, node_id: NodeId) {
        self.last_heartbeat.insert(node_id, Instant::now());
    }

    pub fn is_node_dead(&self, node_id: NodeId) -> bool {
        self.dead_nodes.contains(&node_id)
    }

    pub fn get_dead_nodes(&self) -> Vec<NodeId> {
        self.dead_nodes.iter().copied().collect()
    }

    pub fn recover_node(&mut self, node_id: NodeId) {
        self.dead_nodes.remove(&node_id);
        self.last_heartbeat.remove(&node_id);
        tracing::info!("Node {} has been recovered", node_id);
    }

    pub fn get_failed_shards(&self) -> Vec<ShardId> {
        let failed = Vec::new();
        for node_id in &self.dead_nodes {
            tracing::debug!("Node {} is dead", node_id);
        }
        failed
    }

    pub async fn get_cluster_health(&self) -> ClusterHealth {
        let shard_manager = self.shard_manager.read().await;
        let total_shards = shard_manager.num_shards();
        let total_nodes = shard_manager.num_nodes();
        drop(shard_manager);

        let replica_manager = self.replica_manager.read().await;
        let leader_count = replica_manager.get_leader_count();
        drop(replica_manager);

        ClusterHealth {
            total_nodes,
            dead_nodes: self.dead_nodes.len(),
            total_shards,
            leader_count,
            healthy: self.dead_nodes.is_empty() && leader_count > 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClusterHealth {
    pub total_nodes: usize,
    pub dead_nodes: usize,
    pub total_shards: usize,
    pub leader_count: usize,
    pub healthy: bool,
}

// ============================================================================
// Automatic Failure Detection
// ============================================================================

#[derive(Debug, Clone)]
pub struct FailureEvent {
    pub node_id: NodeId,
    pub detected_at: Instant,
    pub reason: FailureReason,
}

#[derive(Debug, Clone)]
pub enum FailureReason {
    HeartbeatTimeout,
    ReplicationLag,
    NetworkError,
    Manual,
}

#[derive(Debug, Clone)]
pub struct FailureDetector {
    node_id: NodeId,
    last_heartbeat: HashMap<NodeId, Instant>,
    config: FailureDetectorConfig,
}

#[derive(Debug, Clone)]
pub struct FailureDetectorConfig {
    pub check_interval: Duration,
    pub heartbeat_timeout: Duration,
    pub max_replication_lag_ms: u64,
    pub failure_threshold: u32,
}

impl Default for FailureDetectorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_millis(100),
            heartbeat_timeout: Duration::from_millis(500),
            max_replication_lag_ms: 1000,
            failure_threshold: 3,
        }
    }
}

impl FailureDetector {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            last_heartbeat: HashMap::new(),
            config: FailureDetectorConfig::default(),
        }
    }

    pub fn with_config(node_id: NodeId, config: FailureDetectorConfig) -> Self {
        Self {
            node_id,
            last_heartbeat: HashMap::new(),
            config,
        }
    }

    pub fn record_heartbeat(&mut self, node_id: NodeId) {
        self.last_heartbeat.insert(node_id, Instant::now());
    }

    pub fn is_node_alive(&self, node_id: NodeId) -> bool {
        if node_id == self.node_id {
            return true;
        }
        if let Some(last) = self.last_heartbeat.get(&node_id) {
            last.elapsed() < self.config.heartbeat_timeout
        } else {
            false
        }
    }

    pub fn is_node_dead(&self, node_id: NodeId) -> bool {
        !self.is_node_alive(node_id)
    }

    pub fn check_all_nodes(&self) -> Vec<FailureEvent> {
        let mut failures = Vec::new();
        let now = Instant::now();

        for (&node_id, &last) in &self.last_heartbeat {
            if node_id == self.node_id {
                continue;
            }
            if now.duration_since(last) > self.config.heartbeat_timeout {
                failures.push(FailureEvent {
                    node_id,
                    detected_at: now,
                    reason: FailureReason::HeartbeatTimeout,
                });
            }
        }
        failures
    }

    pub fn get_config(&self) -> &FailureDetectorConfig {
        &self.config
    }

    pub fn set_check_interval(&mut self, interval: Duration) {
        self.config.check_interval = interval;
    }

    pub fn set_heartbeat_timeout(&mut self, timeout: Duration) {
        self.config.heartbeat_timeout = timeout;
    }
}

#[derive(Debug, Clone)]
pub struct FailoverTrigger {
    pub node_id: NodeId,
    pub shard_id: ShardId,
    pub triggered_at: Instant,
    pub new_primary: NodeId,
}

#[derive(Debug, Clone)]
pub struct FailoverNotifier {
    subscribers: Arc<RwLock<Vec<tokio::sync::mpsc::Sender<FailureEvent>>>>,
}

impl FailoverNotifier {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn subscribe(
        &self,
    ) -> tokio::sync::mpsc::Receiver<FailureEvent> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        self.subscribers.write().await.push(tx);
        rx
    }

    pub async fn notify_failure(&self, event: &FailureEvent) {
        let mut dead_subscribers = Vec::new();
        let subscribers = self.subscribers.read().await;
        for (i, sub) in subscribers.iter().enumerate() {
            if sub.send(event.clone()).await.is_err() {
                dead_subscribers.push(i);
            }
        }
        drop(subscribers);

        let mut subs = self.subscribers.write().await;
        for i in dead_subscribers.into_iter().rev() {
            subs.remove(i);
        }
    }
}

impl Default for FailoverNotifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_shard_manager() -> Arc<RwLock<ShardManager>> {
        let manager = ShardManager::new();
        Arc::new(RwLock::new(manager))
    }

    fn create_test_replica_manager() -> Arc<RwLock<ShardReplicaManager>> {
        Arc::new(RwLock::new(ShardReplicaManager::new(1)))
    }

    #[tokio::test]
    async fn test_failover_manager_creation() {
        let shard_manager = create_test_shard_manager();
        let replica_manager = create_test_replica_manager();
        let manager = FailoverManager::new(1, shard_manager, replica_manager);

        assert!(manager.get_dead_nodes().is_empty());
        assert!(!manager.is_node_dead(1));
    }

    #[tokio::test]
    async fn test_record_heartbeat() {
        let shard_manager = create_test_shard_manager();
        let replica_manager = create_test_replica_manager();
        let mut manager = FailoverManager::new(1, shard_manager, replica_manager);

        manager.record_heartbeat(2).await;
        assert!(manager.check_node_health(2).await);
    }

    #[tokio::test]
    async fn test_recover_node() {
        let shard_manager = create_test_shard_manager();
        let replica_manager = create_test_replica_manager();
        let mut manager = FailoverManager::new(1, shard_manager, replica_manager);

        manager.dead_nodes.insert(2);
        manager.recover_node(2);

        assert!(!manager.is_node_dead(2));
        assert!(manager.get_dead_nodes().is_empty());
    }

    #[tokio::test]
    async fn test_cluster_health() {
        let shard_manager = create_test_shard_manager();
        let replica_manager = create_test_replica_manager();

        {
            let mut rm = replica_manager.write().await;
            rm.register_shard(0, vec![1, 2, 3]);
            rm.become_leader(0).unwrap();
        }

        let manager = FailoverManager::new(1, shard_manager, replica_manager);

        let health = manager.get_cluster_health().await;
        assert!(health.healthy);
        assert_eq!(health.dead_nodes, 0);
    }

    // FailureDetector tests
    #[tokio::test]
    async fn test_failure_detector_creation() {
        let detector = FailureDetector::new(1);
        assert!(detector.is_node_dead(2));
    }

    #[tokio::test]
    async fn test_failure_detector_record_heartbeat() {
        let mut detector = FailureDetector::new(1);
        detector.record_heartbeat(2);
        assert!(detector.is_node_alive(2));
    }

    #[tokio::test]
    async fn test_failure_detector_config() {
        let config = FailureDetectorConfig::default();
        assert_eq!(config.heartbeat_timeout, Duration::from_millis(500));

        let mut detector = FailureDetector::with_config(
            1,
            FailureDetectorConfig {
                check_interval: Duration::from_millis(200),
                heartbeat_timeout: Duration::from_millis(1000),
                max_replication_lag_ms: 500,
                failure_threshold: 5,
            },
        );
        assert_eq!(detector.get_config().heartbeat_timeout, Duration::from_millis(1000));
    }

    #[test]
    fn test_failure_event_reason() {
        let event = FailureEvent {
            node_id: 1,
            detected_at: Instant::now(),
            reason: FailureReason::HeartbeatTimeout,
        };
        assert!(matches!(event.reason, FailureReason::HeartbeatTimeout));
    }

    #[test]
    fn test_failure_detector_self_node_alive() {
        let detector = FailureDetector::new(1);
        assert!(detector.is_node_alive(1));
        assert!(!detector.is_node_dead(1));
    }

    #[test]
    fn test_failure_detector_unknown_node_dead() {
        let detector = FailureDetector::new(1);
        assert!(detector.is_node_dead(999));
    }

    // FailoverNotifier tests
    #[tokio::test]
    async fn test_failover_notifier_subscribe() {
        let notifier = FailoverNotifier::new();
        let _rx = notifier.subscribe().await;
    }

    #[tokio::test]
    async fn test_failover_notifier_notify() {
        let notifier = FailoverNotifier::new();
        let mut rx = notifier.subscribe().await;

        let event = FailureEvent {
            node_id: 2,
            detected_at: Instant::now(),
            reason: FailureReason::Manual,
        };
        notifier.notify_failure(&event).await;

        let received = rx.recv().await.unwrap();
        assert_eq!(received.node_id, 2);
    }
}
