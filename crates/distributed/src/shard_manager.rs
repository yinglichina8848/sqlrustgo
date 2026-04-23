//! Shard Manager - manages shard metadata and routing
//!
//! Provides shard lifecycle management and routing.

use crate::partition::PartitionKey;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type NodeId = u64;
pub type ShardId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ShardStatus {
    #[default]
    Active,
    Migrating,
    Readonly,
    Offline,
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

    pub fn replicas(&self) -> &[NodeId] {
        &self.replica_nodes
    }

    pub fn promote_replica(&mut self, node: NodeId) {
        if let Some(pos) = self.replica_nodes.iter().position(|&n| n == node) {
            self.replica_nodes.remove(pos);
            self.replica_nodes.insert(0, node);
        }
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
    use crate::partition::PartitionValue;

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

    #[test]
    fn test_shard_info_new() {
        let shard = ShardInfo::new(5, 10);
        assert_eq!(shard.shard_id, 5);
        assert_eq!(shard.primary_node(), Some(10));
        assert_eq!(shard.status, ShardStatus::Active);
    }

    #[test]
    fn test_shard_info_with_partition() {
        let shard = ShardInfo::new(0, 1).with_partition(PartitionKey::new_hash("id", 4));
        assert!(shard.partition_key.is_some());
    }

    #[test]
    fn test_shard_info_add_replica() {
        let mut shard = ShardInfo::new(0, 1);
        shard.add_replica(2);
        shard.add_replica(2); // duplicate - should not add again
        assert_eq!(shard.replicas().len(), 2);
        assert_eq!(shard.replicas(), &[1, 2]);
    }

    #[test]
    fn test_shard_info_promote_replica() {
        let mut shard = ShardInfo::new(0, 1);
        shard.add_replica(2);
        shard.add_replica(3);
        shard.promote_replica(3);
        assert_eq!(shard.primary_node(), Some(3));
        assert_eq!(shard.replicas(), &[3, 1, 2]);
    }

    #[test]
    fn test_partition_rule_new() {
        let rule = PartitionRule::new("users", PartitionKey::new_hash("user_id", 8));
        assert_eq!(rule.table, "users");
    }

    #[test]
    fn test_shard_manager_num_shards() {
        let manager = ShardManager::new();
        assert_eq!(manager.num_shards(), 0);
    }

    #[test]
    fn test_shard_manager_get_shard_none() {
        let manager = ShardManager::new();
        assert!(manager.get_shard(999).is_none());
    }

    #[test]
    fn test_shard_manager_remove_shard() {
        let mut manager = ShardManager::new();
        manager.create_shard(ShardInfo::new(0, 1));
        assert_eq!(manager.num_shards(), 1);
        manager.remove_node_from_shard(0, 1);
        assert_eq!(manager.num_shards(), 1);
    }

    #[test]
    fn test_shard_info_fields() {
        let shard = ShardInfo::new(5, 10);
        assert_eq!(shard.shard_id, 5);
        assert_eq!(shard.primary_node(), Some(10));
        assert_eq!(shard.replicas().len(), 1);
        assert_eq!(shard.status, ShardStatus::Active);
    }

    #[test]
    fn test_shard_info_status() {
        let mut shard = ShardInfo::new(0, 1);
        assert_eq!(shard.status, ShardStatus::Active);

        shard.status = ShardStatus::Migrating;
        assert_eq!(shard.status, ShardStatus::Migrating);

        shard.status = ShardStatus::Readonly;
        assert_eq!(shard.status, ShardStatus::Readonly);

        shard.status = ShardStatus::Offline;
        assert_eq!(shard.status, ShardStatus::Offline);
    }

    #[test]
    fn test_shard_status_debug() {
        assert_eq!(format!("{:?}", ShardStatus::Active), "Active");
        assert_eq!(format!("{:?}", ShardStatus::Migrating), "Migrating");
        assert_eq!(format!("{:?}", ShardStatus::Readonly), "Readonly");
        assert_eq!(format!("{:?}", ShardStatus::Offline), "Offline");
    }

    #[test]
    fn test_shard_info_debug() {
        let shard = ShardInfo::new(1, 2);
        let debug_str = format!("{:?}", shard);
        assert!(debug_str.contains("shard_id: 1"));
    }

    #[test]
    fn test_shard_info_promote_first_replica() {
        let mut shard = ShardInfo::new(0, 1);
        shard.add_replica(2);
        shard.add_replica(3);
        shard.promote_replica(1);
        assert_eq!(shard.primary_node(), Some(1));
    }

    #[test]
    fn test_shard_manager_get_shards_by_node() {
        let mut manager = ShardManager::new();
        manager.create_shard(ShardInfo::new(0, 1));
        manager.create_shard(ShardInfo::new(1, 2));

        let shards = manager.get_shards_by_node(999);
        assert!(shards.is_empty());
    }

    #[test]
    fn test_partition_rule_debug() {
        let rule = PartitionRule::new("users", PartitionKey::new_hash("user_id", 8));
        let debug_str = format!("{:?}", rule);
        assert!(debug_str.contains("users"));
    }

    #[test]
    fn test_get_active_shards_filters_by_status() {
        let mut manager = ShardManager::new();
        manager.create_shard(ShardInfo::new(0, 1));
        manager.create_shard(ShardInfo::new(1, 2));
        manager.create_shard(ShardInfo::new(2, 3));
        manager.set_shard_status(1, ShardStatus::Offline);

        let active = manager.get_active_shards();
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|s| s.status == ShardStatus::Active));
    }

    #[test]
    fn test_get_active_shards_all_offline() {
        let mut manager = ShardManager::new();
        let mut shard = ShardInfo::new(0, 1);
        shard.status = ShardStatus::Offline;
        manager.create_shard(shard);

        assert!(manager.get_active_shards().is_empty());
    }

    #[test]
    fn test_get_node_shards_returns_correct_shards() {
        let mut manager = ShardManager::new();
        manager.create_shard(ShardInfo::new(0, 1));
        manager.create_shard(ShardInfo::new(1, 1));
        manager.create_shard(ShardInfo::new(2, 2));

        let node1_shards = manager.get_node_shards(1).unwrap();
        assert_eq!(node1_shards.len(), 2);
        assert!(node1_shards.contains(&0));
        assert!(node1_shards.contains(&1));

        let node2_shards = manager.get_node_shards(2).unwrap();
        assert_eq!(node2_shards.len(), 1);
        assert!(node2_shards.contains(&2));
    }

    #[test]
    fn test_get_node_shards_nonexistent_node() {
        let manager = ShardManager::new();
        assert!(manager.get_node_shards(999).is_none());
    }

    #[test]
    fn test_remove_node_from_shard_updates_replica_list() {
        let mut manager = ShardManager::new();
        let mut shard = ShardInfo::new(0, 1);
        shard.add_replica(2);
        shard.add_replica(3);
        manager.create_shard(shard);

        manager.remove_node_from_shard(0, 2);

        let shard_info = manager.get_shard(0).unwrap();
        assert_eq!(shard_info.replica_nodes, vec![1, 3]);
    }

    #[test]
    fn test_remove_node_from_shard_nonexistent_shard() {
        let mut manager = ShardManager::new();
        assert!(!manager.remove_node_from_shard(999, 1));
    }

    #[test]
    fn test_remove_node_from_shard_nonexistent_node() {
        let mut manager = ShardManager::new();
        manager.create_shard(ShardInfo::new(0, 1));
        manager.remove_node_from_shard(0, 999);
        assert_eq!(manager.get_shard(0).unwrap().replica_nodes, vec![1]);
    }

    #[test]
    fn test_set_shard_status_nonexistent_shard() {
        let mut manager = ShardManager::new();
        manager.set_shard_status(999, ShardStatus::Offline);
        assert!(manager.get_shard(999).is_none());
    }

    #[test]
    fn test_initialize_table_shards_distributes_replicas() {
        let mut manager = ShardManager::new();
        let nodes = vec![1, 2, 3];
        manager.initialize_table_shards("orders", 3, &nodes);

        let shard0 = manager.get_shard(0).unwrap();
        assert_eq!(shard0.primary_node(), Some(1));
        assert_eq!(shard0.replicas(), &[1, 2, 3]);

        let shard1 = manager.get_shard(1).unwrap();
        assert_eq!(shard1.primary_node(), Some(2));
        assert_eq!(shard1.replicas(), &[2, 1, 3]);

        let shard2 = manager.get_shard(2).unwrap();
        assert_eq!(shard2.primary_node(), Some(3));
        assert_eq!(shard2.replicas(), &[3, 1, 2]);
    }

    #[test]
    fn test_initialize_table_shards_single_node() {
        let mut manager = ShardManager::new();
        manager.initialize_table_shards("users", 4, &[1]);

        for i in 0..4 {
            let shard = manager.get_shard(i).unwrap();
            assert_eq!(shard.primary_node(), Some(1));
            assert_eq!(shard.replicas().len(), 1);
        }
    }

    #[test]
    fn test_num_nodes_counts_distinct_nodes() {
        let mut manager = ShardManager::new();
        manager.create_shard(ShardInfo::new(0, 1));
        manager.create_shard(ShardInfo::new(1, 1));
        manager.create_shard(ShardInfo::new(2, 2));
        assert_eq!(manager.num_nodes(), 2);
    }

    #[test]
    fn test_shard_info_promote_nonexistent_replica() {
        let mut shard = ShardInfo::new(0, 1);
        shard.add_replica(2);
        shard.promote_replica(999);
        assert_eq!(shard.primary_node(), Some(1));
        assert_eq!(shard.replicas(), &[1, 2]);
    }

    #[test]
    fn test_with_default_shards() {
        let manager = ShardManager::new().with_default_shards(8);
        assert_eq!(manager.num_shards(), 0);
    }
}
