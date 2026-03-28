use crate::gid::GlobalTransactionId;
use serde::{Deserialize, Serialize};

/// 分布式事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributedTransactionState {
    Started,
    Preparing,
    Prepared,
    Committing,
    Committed,
    RollingBack,
    RolledBack,
    Terminated,
}

/// 参与者投票
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    VoteCommit,
    VoteAbort,
}

/// 协调消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinatorMessage {
    PrepareRequest {
        gid: GlobalTransactionId,
        changes: Vec<Change>,
    },
    CommitRequest {
        gid: GlobalTransactionId,
    },
    RollbackRequest {
        gid: GlobalTransactionId,
        reason: String,
    },
}

/// 参与者响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantResponse {
    pub gid: GlobalTransactionId,
    pub node_id: u64,
    pub vote: Vote,
    pub reason: Option<String>,
}

/// 数据变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub table: String,
    pub operation: ChangeOperation,
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
}

/// 变更操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeOperation {
    Insert,
    Update,
    Delete,
}

/// 事务上下文
#[derive(Debug, Clone)]
pub struct TransactionContext {
    pub gid: GlobalTransactionId,
    pub state: DistributedTransactionState,
    pub participants: Vec<u64>,
    pub changes: Vec<Change>,
}

impl TransactionContext {
    pub fn new(gid: GlobalTransactionId) -> Self {
        TransactionContext {
            gid,
            state: DistributedTransactionState::Started,
            participants: Vec::new(),
            changes: Vec::new(),
        }
    }

    pub fn add_participant(&mut self, node_id: u64) {
        if !self.participants.contains(&node_id) {
            self.participants.push(node_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeId;

    #[test]
    fn test_transaction_state_transitions() {
        assert_eq!(
            DistributedTransactionState::Started,
            DistributedTransactionState::Started
        );
    }

    #[test]
    fn test_vote_serialization() {
        assert_eq!(Vote::VoteCommit, Vote::VoteCommit);
        assert_eq!(Vote::VoteAbort, Vote::VoteAbort);
    }

    #[test]
    fn test_change_operation() {
        let insert = ChangeOperation::Insert;
        let update = ChangeOperation::Update;
        let delete = ChangeOperation::Delete;

        assert_ne!(insert, update);
        assert_ne!(update, delete);
    }

    #[test]
    fn test_transaction_context_new() {
        let node_id = NodeId(1);
        let gid = GlobalTransactionId::new(node_id);
        let ctx = TransactionContext::new(gid.clone());

        assert_eq!(ctx.gid, gid);
        assert_eq!(ctx.state, DistributedTransactionState::Started);
        assert!(ctx.participants.is_empty());
        assert!(ctx.changes.is_empty());
    }

    #[test]
    fn test_transaction_context_add_participant() {
        let node_id = NodeId(1);
        let gid = GlobalTransactionId::new(node_id);
        let mut ctx = TransactionContext::new(gid);

        ctx.add_participant(2);
        ctx.add_participant(3);
        ctx.add_participant(2); // 重复添加

        assert_eq!(ctx.participants.len(), 2);
        assert!(ctx.participants.contains(&2));
        assert!(ctx.participants.contains(&3));
    }
}