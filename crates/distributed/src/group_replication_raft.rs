//! Group Replication with Raft Integration
//!
//! Implements virtual synchronous replication by integrating:
//! - Group Replication (certification, conflict detection)
//! - Raft consensus (transaction ordering, log replication)

use std::collections::HashSet;

use crate::group_membership::{GroupMembership, View};
use crate::group_replication::{
    CertificationResult, GroupReplication, GroupReplicationConfig, TransactionContext, TransactionId,
};
use crate::raft::{RaftEntry, RaftEntryData, RaftNode};

/// Group Replication with Raft integration state
pub struct GroupReplicationRaft {
    group_rep: GroupReplication,
    raft: RaftNode,
    index_to_tx: Vec<(u64, TransactionId)>,
    is_leader: bool,
}

impl GroupReplicationRaft {
    pub fn new(membership: GroupMembership, config: GroupReplicationConfig, node_id: u64) -> Self {
        let local_id = membership.get_local_node_id();
        let peers: Vec<u64> = membership
            .get_all_members()
            .iter()
            .filter(|m| m.node_id != local_id)
            .map(|m| m.node_id)
            .collect();

        Self {
            group_rep: GroupReplication::new(membership, config),
            raft: RaftNode::new(node_id, peers),
            index_to_tx: Vec::new(),
            is_leader: false,
        }
    }

    pub fn with_defaults(membership: GroupMembership, node_id: u64) -> Self {
        Self::new(membership, GroupReplicationConfig::default(), node_id)
    }

    /// Check if current node is the Raft leader
    pub fn is_raft_leader(&self) -> bool {
        self.is_leader
    }

    /// Check if can accept writes (must be leader in Raft + certified)
    pub fn can_accept_writes(&self) -> bool {
        self.is_leader && self.group_rep.can_accept_writes()
    }

    /// Submit a transaction for certification and Raft replication
    pub fn submit_transaction(
        &mut self,
        write_set: HashSet<Vec<u8>>,
    ) -> (CertificationResult, Option<u64>) {
        if !self.can_accept_writes() {
            return (CertificationResult::Aborted, None);
        }

        let tx_id = self.group_rep.new_transaction_id();
        let view = self.group_rep.get_view();

        let ctx = TransactionContext {
            tx_id: tx_id.clone(),
            write_set: write_set.clone(),
            snapshot_version: view.view_id.id,
        };

        let cert_result = self.group_rep.certify(ctx);

        if cert_result == CertificationResult::Certified {
            let tx_sequence = tx_id.sequence_number;
            let raft_entry = RaftEntry {
                term: self.raft.term(),
                index: 0,
                data: RaftEntryData::Transaction { tx_id: tx_sequence },
            };

            self.raft.append_entry(raft_entry);
            let index = self.raft.last_index();
            self.index_to_tx.push((index, tx_id));

            (cert_result, Some(index))
        } else {
            (cert_result, None)
        }
    }

    /// Handle a view change from group membership
    pub fn on_view_change(&mut self, new_view: View) {
        self.group_rep.on_view_change(new_view.clone());

        let raft_entry = RaftEntry {
            term: self.raft.term(),
            index: 0,
            data: RaftEntryData::ConfigChange {
                node_id: new_view.view_id.leader_id,
                add: true,
            },
        };
        self.raft.append_entry(raft_entry);
    }

    /// Check and update leader status
    pub fn check_leader_status(&mut self) {
        self.raft.check_election_timeout();
        self.is_leader = self.raft.is_leader();
    }

    /// Get applied entries count
    pub fn applied_count(&self) -> usize {
        self.index_to_tx.len()
    }

    /// Get underlying group replication
    pub fn group_replication(&self) -> &GroupReplication {
        &self.group_rep
    }

    /// Get underlying Raft node
    pub fn raft_node(&self) -> &RaftNode {
        &self.raft
    }

    /// Get certified transaction count
    pub fn certified_count(&self) -> usize {
        self.group_rep.certified_count()
    }

    /// Get certification window size
    pub fn window_size(&self) -> usize {
        self.group_rep.window_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group_membership::MemberState;

    #[test]
    fn test_raft_leader_write() {
        let gm = GroupMembership::new(1);
        gm.add_member(2, "node-2".to_string()).unwrap();
        gm.update_member_state(2, MemberState::Online).unwrap();

        let mut grr = GroupReplicationRaft::with_defaults(gm, 1);

        assert!(!grr.is_raft_leader());
        assert!(!grr.can_accept_writes());
    }

    #[test]
    fn test_certification_integration() {
        let gm = GroupMembership::new(1);
        let mut grr = GroupReplicationRaft::with_defaults(gm, 1);

        grr.is_leader = true;

        let mut write_set = HashSet::new();
        write_set.insert(b"key1".to_vec());

        let (result, index) = grr.submit_transaction(write_set);
        assert_eq!(result, CertificationResult::Certified);
        assert!(index.is_some());
    }
}