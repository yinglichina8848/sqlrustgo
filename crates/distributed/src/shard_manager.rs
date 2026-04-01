//! Shard Manager - manages shard metadata and routing
//!
//! Provides shard lifecycle management and routing.

use crate::partition::{PartitionKey, PartitionStrategy, PartitionValue};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type NodeId = u64;
pub type ShardId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShardStatus {
    Active,
    Migrating,
    Readonly,
    Offline,
}

impl Default for ShardStatus {
    fn default() -> Self {
        ShardStatus::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    pub shard_id: ShardId,
    pub replica_nodes: Vec<NodeId>,
    pub partition_key: Option<PartitionKey>,
    pub status: ShardStatus,
}

impl ShardInfo {
    pub fn new(shard_id: ShardId, primary_node: NodeId) -> Self {
        Self {
            shard_id,
            replica_nodes: vec![primary_node],
            partition_key: None,
            status: ShardStatus::Active,
        }
    }

    pub fn with_partition(mut self, partition_key: PartitionKey) -> Self {
        self.partition_key = Some(partition_key);
        self
    }

    pub fn add_replica(&mut self, node: NodeId) {
        if !self.replica_nodes.contains(&node) {
            self.replica_nodes.push(node);
        }
    }

    pub fn primary_node(&self) -> Option<NodeId> {
        self.replica_nodes.first().copied()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionRule {
    pub table: String,
    pub partition_key: PartitionKey,
}

impl PartitionRule {
    pub fn new(table: &str, partition_key: PartitionKey) -> Self {
        Self {
            table: table.to_string(),
            partition_key,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShardManager {
    shards: HashMap<ShardId, ShardInfo>,
    node_shards: HashMap<NodeId, HashSet<ShardId>>,
    partition_rules: HashMap<String, PartitionKey>,
    default_num_shards: u64,
}

impl ShardManager {
    pub fn new() -> Self {
        Self {
            shards: HashMap::new(),
            node_shards: HashMap::new(),
            partition_rules: HashMap::new(),
            default_num_shards: 4,
        }
    }

    pub fn with_default_shards(mut self, num: u64) -> Self {
        self.default_num_shards = num;
        self
    }

    pub fn create_shard(&mut self, info: ShardInfo) {
        let shard_id = info.shard_id;
        self.shards.insert(shard_id, info.clone());

        if let Some(primary) = info.primary_node() {
            self.node_shards
                .entry(primary)
                .or_default()
                .insert(shard_id);
        }
    }

    pub fn get_shard(&self, shard_id: ShardId) -> Option<&ShardInfo> {
        self.shards.get(&shard_id)
    }

    pub fn get_shard_mut(&mut self, shard_id: ShardId) -> Option<&mut ShardInfo> {
        self.shards.get_mut(&shard_id)
    }

    pub fn get_shards(&self) -> &HashMap<ShardId, ShardInfo> {
        &self.shards
    }

    pub fn get_node_shards(&self, node_id: NodeId) -> Option<&HashSet<ShardId>> {
        self.node_shards.get(&node_id)
    }

    pub fn add_partition_rule(&mut self, rule: PartitionRule) {
        self.partition_rules.insert(rule.table, rule.partition_key);
    }

    pub fn get_partition_key(&self, table: &str) -> Option<&PartitionKey> {
        self.partition_rules.get(table)
    }

    pub fn set_shard_status(&mut self, shard_id: ShardId, status: ShardStatus) {
        if let Some(shard) = self.shards.get_mut(&shard_id) {
            shard.status = status;
        }
    }

    pub fn get_active_shards(&self) -> Vec<&ShardInfo> {
        self.shards
            .values()
            .filter(|s| s.status == ShardStatus::Active)
            .collect()
    }

    pub fn get_shards_by_node(&self, node_id: NodeId) -> Vec<&ShardInfo> {
        self.node_shards
            .get(&node_id)
            .map(|shard_ids| {
                shard_ids
                    .iter()
                    .filter_map(|id| self.shards.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn remove_node_from_shard(&mut self, shard_id: ShardId, node_id: NodeId) -> bool {
        if let Some(shard) = self.shards.get_mut(&shard_id) {
            shard.replica_nodes.retain(|&n| n != node_id);
            if let Some(shards) = self.node_shards.get_mut(&node_id) {
                shards.remove(&shard_id);
            }
            return true;
        }
        false
    }

    pub fn num_shards(&self) -> usize {
        self.shards.len()
    }

    pub fn num_nodes(&self) -> usize {
        self.node_shards.len()
    }

    pub fn initialize_table_shards(&mut self, table: &str, num_shards: u64, nodes: &[NodeId]) {
        let partition_key = PartitionKey::new_hash("id", num_shards);
        self.add_partition_rule(PartitionRule::new(table, partition_key.clone()));

        for i in 0..num_shards {
            let primary = nodes[(i as usize) % nodes.len()];
            let mut shard = ShardInfo::new(i, primary).with_partition(partition_key.clone());

            for node in nodes {
                if *node != primary {
                    shard.add_replica(*node);
                }
            }

            self.create_shard(shard);
        }
    }
}

impl Default for ShardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_shard() {
        let mut manager = ShardManager::new();
        let shard = ShardInfo::new(0, 1);
        manager.create_shard(shard);

        assert_eq!(manager.num_shards(), 1);
        assert_eq!(manager.get_shard(0).unwrap().primary_node(), Some(1));
    }

    #[test]
    fn test_shard_replicas() {
        let mut manager = ShardManager::new();
        let mut shard = ShardInfo::new(0, 1);
        shard.add_replica(2);
        shard.add_replica(3);
        manager.create_shard(shard);

        let info = manager.get_shard(0).unwrap();
        assert_eq!(info.replica_nodes, vec![1, 2, 3]);
    }

    #[test]
    fn test_partition_rule() {
        let mut manager = ShardManager::new();
        let rule = PartitionRule::new("users", PartitionKey::new_hash("user_id", 8));
        manager.add_partition_rule(rule);

        let pk = manager.get_partition_key("users").unwrap();
        let shard = pk.partition(&PartitionValue::Integer(15)).unwrap();
        assert_eq!(shard, 7); // 15 % 8 = 7
    }

    #[test]
    fn test_initialize_table_shards() {
        let mut manager = ShardManager::new();
        let nodes = vec![1, 2, 3];
        manager.initialize_table_shards("orders", 6, &nodes);

        assert_eq!(manager.num_shards(), 6);

        let pk = manager.get_partition_key("orders").unwrap();
        let shard = pk.partition(&PartitionValue::Integer(7)).unwrap();
        assert_eq!(shard, 1); // 7 % 6 = 1
    }

    #[test]
    fn test_shard_status() {
        let mut manager = ShardManager::new();
        manager.create_shard(ShardInfo::new(0, 1));

        manager.set_shard_status(0, ShardStatus::Migrating);
        assert_eq!(manager.get_shard(0).unwrap().status, ShardStatus::Migrating);

        manager.set_shard_status(0, ShardStatus::Readonly);
        assert_eq!(manager.get_shard(0).unwrap().status, ShardStatus::Readonly);
    }
}
