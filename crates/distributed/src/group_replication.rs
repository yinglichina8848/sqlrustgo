//! Group Replication Module
//!
//! Implements MySQL Group Replication compatible virtual synchronous replication:
//! - Single-Primary mode: all writes go through a single primary
//! - Certification-based conflict detection
//! - Automatic primary election on failure
//! - Transaction ordering via globally ordered view changes

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use crate::group_membership::{GroupMembership, MemberState, View};

/// Transaction certification result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificationResult {
    Certified,
    Aborted,
}

/// Transaction identifier with sequence number for ordering
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId {
    pub node_id: u64,
    pub sequence_number: u64,
}

/// Transaction context for certification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionContext {
    pub tx_id: TransactionId,
    pub write_set: HashSet<Vec<u8>>,
    pub snapshot_version: u64,
}

/// Certification information for a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationInfo {
    pub tx_id: TransactionId,
    pub view_id: u64,
    pub write_set: HashSet<Vec<u8>>,
    pub result: CertificationResult,
}

/// Group Replication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupReplicationConfig {
    pub single_primary: bool,
    pub auto_rejoin: bool,
    pub certification_window_size: usize,
    pub max_memory_size: usize,
}

impl Default for GroupReplicationConfig {
    fn default() -> Self {
        Self {
            single_primary: true,
            auto_rejoin: true,
            certification_window_size: 1000,
            max_memory_size: 128 * 1024 * 1024, // 128MB
        }
    }
}

/// Group Replication state machine
pub struct GroupReplication {
    membership: GroupMembership,
    config: GroupReplicationConfig,
    /// Sequence number for local transactions
    local_sequence: u64,
    /// Certified transactions in current view
    certified: HashMap<TransactionId, CertificationInfo>,
    /// Certification window
    certification_window: Vec<CertificationInfo>,
    /// Current primary node
    primary: Option<u64>,
    /// Whether this node is in recovery mode
    recovery_mode: bool,
}

impl GroupReplication {
    pub fn new(membership: GroupMembership, config: GroupReplicationConfig) -> Self {
        let primary = membership.elect_primary();
        Self {
            membership,
            config,
            local_sequence: 0,
            certified: HashMap::new(),
            certification_window: Vec::new(),
            primary,
            recovery_mode: false,
        }
    }

    pub fn with_defaults(membership: GroupMembership) -> Self {
        Self::new(membership, GroupReplicationConfig::default())
    }

    /// Get the current view
    pub fn get_view(&self) -> View {
        self.membership.get_current_view()
    }

    /// Check if current node is primary
    pub fn is_primary(&self) -> bool {
        if let Some(primary) = self.primary {
            primary == self.membership.get_local_node_id()
        } else {
            false
        }
    }

    /// Check if current node can accept writes
    pub fn can_accept_writes(&self) -> bool {
        self.is_primary() && !self.recovery_mode
    }

    /// Generate a new transaction ID
    pub fn new_transaction_id(&mut self) -> TransactionId {
        let node_id = self.membership.get_local_node_id();
        self.local_sequence += 1;
        TransactionId {
            node_id,
            sequence_number: self.local_sequence,
        }
    }

    /// Certify a transaction for commit
    pub fn certify(&mut self, ctx: TransactionContext) -> CertificationResult {
        let view = self.get_view();
        let view_id = view.view_id.id;

        // Check if transaction is from a valid member
        if !view.members.contains(&ctx.tx_id.node_id) {
            return CertificationResult::Aborted;
        }

        // Check write-write conflicts using write set
        for info in self.certified.values() {
            if info.view_id == view_id {
                // Check for intersection of write sets
                let intersection: HashSet<_> = ctx
                    .write_set
                    .intersection(&info.write_set)
                    .collect();
                if !intersection.is_empty() {
                    // Conflict detected
                    return CertificationResult::Aborted;
                }
            }
        }

        let result = CertificationResult::Certified;
        let cert_info = CertificationInfo {
            tx_id: ctx.tx_id.clone(),
            view_id,
            write_set: ctx.write_set.clone(),
            result,
        };

        // Add to certified set
        self.certified.insert(ctx.tx_id.clone(), cert_info.clone());

        // Add to certification window
        self.certification_window.push(cert_info);

        // Prune old entries from certification window
        while self.certification_window.len() > self.config.certification_window_size {
            if let Some(old) = self.certification_window.first() {
                self.certified.remove(&old.tx_id);
                self.certification_window.remove(0);
            }
        }

        result
    }

    /// Handle a view change
    pub fn on_view_change(&mut self, new_view: View) {
        if self.config.single_primary && new_view.primary_member.is_some() {
            self.primary = new_view.primary_member;
        }
    }

    /// Get current primary
    pub fn get_primary(&self) -> Option<u64> {
        self.primary
    }

    /// Check if in recovery mode
    pub fn is_recovering(&self) -> bool {
        self.recovery_mode
    }

    /// Set recovery mode
    pub fn set_recovery_mode(&mut self, recovering: bool) {
        self.recovery_mode = recovering;
    }

    /// Get the underlying membership
    pub fn get_membership(&self) -> &GroupMembership {
        &self.membership
    }

    /// Get membership mut
    pub fn get_membership_mut(&mut self) -> &mut GroupMembership {
        &mut self.membership
    }

    /// Replicate a transaction to group (returns true if certified)
    pub fn replicate(&mut self, write_set: HashSet<Vec<u8>>) -> CertificationResult {
        if !self.can_accept_writes() {
            return CertificationResult::Aborted;
        }

        let tx_id = self.new_transaction_id();
        let view = self.get_view();

        let ctx = TransactionContext {
            tx_id,
            write_set,
            snapshot_version: view.view_id.id,
        };

        self.certify(ctx)
    }

    /// Get certified transaction count
    pub fn certified_count(&self) -> usize {
        self.certified.len()
    }

    /// Get certification window size
    pub fn window_size(&self) -> usize {
        self.certification_window.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_primary_write() {
        let gm = GroupMembership::new(1);
        gm.add_member(2, "node-2".to_string()).unwrap();
        gm.add_member(3, "node-3".to_string()).unwrap();
        gm.update_member_state(2, MemberState::Online).unwrap();
        gm.update_member_state(3, MemberState::Online).unwrap();

        let mut gr = GroupReplication::with_defaults(gm);
        gr.primary = Some(1);

        assert!(gr.is_primary());
        assert!(gr.can_accept_writes());
    }

    #[test]
    fn test_non_primary_cannot_write() {
        let gm = GroupMembership::new(1);
        let mut gr = GroupReplication::with_defaults(gm);
        gr.primary = Some(2); // Node 2 is primary, not us

        assert!(!gr.is_primary());
        assert!(!gr.can_accept_writes());
    }

    #[test]
    fn test_certification() {
        let gm = GroupMembership::new(1);
        let mut gr = GroupReplication::with_defaults(gm);

        let mut write_set1 = HashSet::new();
        write_set1.insert(b"key1".to_vec());

        let tx_id = gr.new_transaction_id();
        let ctx = TransactionContext {
            tx_id,
            write_set: write_set1,
            snapshot_version: 1,
        };

        let result = gr.certify(ctx);
        assert_eq!(result, CertificationResult::Certified);
        assert_eq!(gr.certified_count(), 1);
    }

    #[test]
    fn test_conflict_detection() {
        let gm = GroupMembership::new(1);
        let mut gr = GroupReplication::with_defaults(gm);

        // First transaction writes "key1"
        let tx_id1 = gr.new_transaction_id();
        let mut write_set1 = HashSet::new();
        write_set1.insert(b"key1".to_vec());

        let ctx1 = TransactionContext {
            tx_id: tx_id1,
            write_set: write_set1,
            snapshot_version: 1,
        };
        assert_eq!(gr.certify(ctx1), CertificationResult::Certified);

        // Second transaction also writes "key1" - should abort
        let tx_id2 = gr.new_transaction_id();
        let mut write_set2 = HashSet::new();
        write_set2.insert(b"key1".to_vec());

        let ctx2 = TransactionContext {
            tx_id: tx_id2,
            write_set: write_set2,
            snapshot_version: 1,
        };
        assert_eq!(gr.certify(ctx2), CertificationResult::Aborted);
    }

    #[test]
    fn test_no_conflict_different_keys() {
        let gm = GroupMembership::new(1);
        let mut gr = GroupReplication::with_defaults(gm);

        // First transaction writes "key1"
        let tx_id1 = gr.new_transaction_id();
        let mut write_set1 = HashSet::new();
        write_set1.insert(b"key1".to_vec());

        let ctx1 = TransactionContext {
            tx_id: tx_id1,
            write_set: write_set1,
            snapshot_version: 1,
        };
        assert_eq!(gr.certify(ctx1), CertificationResult::Certified);

        // Second transaction writes "key2" - should succeed
        let tx_id2 = gr.new_transaction_id();
        let mut write_set2 = HashSet::new();
        write_set2.insert(b"key2".to_vec());

        let ctx2 = TransactionContext {
            tx_id: tx_id2,
            write_set: write_set2,
            snapshot_version: 1,
        };
        assert_eq!(gr.certify(ctx2), CertificationResult::Certified);
    }

    #[test]
    fn test_recovery_mode() {
        let gm = GroupMembership::new(1);
        let mut gr = GroupReplication::with_defaults(gm);
        gr.primary = Some(1);

        assert!(gr.can_accept_writes());

        gr.set_recovery_mode(true);
        assert!(!gr.can_accept_writes());

        gr.set_recovery_mode(false);
        assert!(gr.can_accept_writes());
    }
}