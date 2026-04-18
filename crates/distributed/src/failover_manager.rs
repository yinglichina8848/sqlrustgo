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

        // Register a shard and make it a leader so health check passes
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
}
