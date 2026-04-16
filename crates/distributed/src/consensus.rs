use crate::error::DistributedError;
use crate::raft::{RaftEntry, RaftEntryData, RaftNode, RaftState};
use crate::shard_manager::ShardId;
use std::collections::HashMap;

pub type NodeId = u64;

#[derive(Debug, Clone)]
pub enum Operation {
    Insert { key: String, value: Vec<u8> },
    Delete { key: String },
    Update { key: String, value: Vec<u8> },
}

impl Operation {
    pub fn to_entry_data(&self, tx_id: u64) -> RaftEntryData {
        match self {
            Operation::Insert { .. } | Operation::Delete { .. } | Operation::Update { .. } => {
                RaftEntryData::Transaction { tx_id }
            }
        }
    }
}

pub struct ShardReplicaManager {
    shard_nodes: HashMap<ShardId, RaftNode>,
    shard_to_primary: HashMap<ShardId, NodeId>,
    shard_replicas: HashMap<ShardId, Vec<NodeId>>,
    node_id: NodeId,
}

impl ShardReplicaManager {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            shard_nodes: HashMap::new(),
            shard_to_primary: HashMap::new(),
            shard_replicas: HashMap::new(),
            node_id,
        }
    }

    pub fn register_shard(&mut self, shard_id: ShardId, replicas: Vec<NodeId>) {
        let peers: Vec<NodeId> = replicas
            .into_iter()
            .filter(|&n| n != self.node_id)
            .collect();
        let raft_node = RaftNode::new(self.node_id, peers.clone());

        self.shard_nodes.insert(shard_id, raft_node);
        self.shard_replicas.insert(shard_id, peers);
    }

    pub fn get_primary(&self, shard_id: ShardId) -> Option<NodeId> {
        self.shard_to_primary.get(&shard_id).copied()
    }

    pub fn is_primary(&self, shard_id: ShardId) -> bool {
        self.shard_to_primary
            .get(&shard_id)
            .map(|&primary| primary == self.node_id)
            .unwrap_or(false)
    }

    pub fn get_shard_state(&self, shard_id: ShardId) -> Option<RaftState> {
        self.shard_nodes.get(&shard_id).map(|n| n.state())
    }

    pub fn is_leader(&self, shard_id: ShardId) -> bool {
        self.shard_nodes
            .get(&shard_id)
            .map(|n| n.is_leader())
            .unwrap_or(false)
    }

    pub fn become_leader(&mut self, shard_id: ShardId) -> Result<(), DistributedError> {
        if let Some(node) = self.shard_nodes.get_mut(&shard_id) {
            node.become_leader();
            self.shard_to_primary.insert(shard_id, self.node_id);
            Ok(())
        } else {
            Err(DistributedError::ShardNotFound(shard_id))
        }
    }

    pub fn replicate_operation(
        &mut self,
        shard_id: ShardId,
        op: Operation,
    ) -> Result<(), DistributedError> {
        let node = self
            .shard_nodes
            .get_mut(&shard_id)
            .ok_or(DistributedError::ShardNotFound(shard_id))?;

        if !node.is_leader() {
            return Err(DistributedError::Consensus(
                "Not the leader for this shard".to_string(),
            ));
        }

        let entry = RaftEntry {
            term: node.term(),
            index: node.last_index() + 1,
            data: op.to_entry_data(node.term()),
        };

        node.append_entry(entry);
        Ok(())
    }

    pub fn handle_vote_response(
        &mut self,
        shard_id: ShardId,
        vote_granted: bool,
    ) -> Result<bool, DistributedError> {
        let node = self
            .shard_nodes
            .get_mut(&shard_id)
            .ok_or(DistributedError::ShardNotFound(shard_id))?;

        if vote_granted {
            if node.become_leader_on_votes(node.count_votes() + 1) {
                self.shard_to_primary.insert(shard_id, self.node_id);
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn get_all_leaders(&self) -> Vec<ShardId> {
        self.shard_nodes
            .iter()
            .filter(|(_, node)| node.is_leader())
            .map(|(&shard_id, _)| shard_id)
            .collect()
    }

    pub fn get_leader_count(&self) -> usize {
        self.shard_nodes.values().filter(|n| n.is_leader()).count()
    }

    pub fn get_follower_count(&self) -> usize {
        self.shard_nodes
            .values()
            .filter(|n| n.state() == RaftState::Follower)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_replica_manager_register() {
        let mut manager = ShardReplicaManager::new(1);
        manager.register_shard(0, vec![1, 2, 3]);

        assert_eq!(manager.get_shard_state(0), Some(RaftState::Follower));
        assert!(!manager.is_leader(0));
    }

    #[test]
    fn test_become_leader() {
        let mut manager = ShardReplicaManager::new(1);
        manager.register_shard(0, vec![1, 2, 3]);

        manager.become_leader(0).unwrap();

        assert!(manager.is_leader(0));
        assert!(manager.is_primary(0));
        assert_eq!(manager.get_primary(0), Some(1));
    }

    #[test]
    fn test_replicate_operation_requires_leader() {
        let mut manager = ShardReplicaManager::new(1);
        manager.register_shard(0, vec![1, 2, 3]);

        let op = Operation::Insert {
            key: "test".to_string(),
            value: vec![1, 2, 3],
        };

        let result = manager.replicate_operation(0, op);
        assert!(result.is_err());
    }

    #[test]
    fn test_shard_not_found() {
        let manager = ShardReplicaManager::new(1);

        assert_eq!(manager.get_primary(999), None);
        assert_eq!(manager.get_shard_state(999), None);
    }
}
