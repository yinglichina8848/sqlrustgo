use crate::dtc::{DistributedTransactionState, TransactionContext};
use crate::gid::{GlobalTransactionId, NodeId};
use std::collections::HashMap;
use std::sync::RwLock;

/// 事务协调者
pub struct Coordinator {
    node_id: NodeId,
    pending_transactions: RwLock<HashMap<GlobalTransactionId, TransactionContext>>,
}

impl Coordinator {
    pub fn new(node_id: NodeId) -> Self {
        Coordinator {
            node_id,
            pending_transactions: RwLock::new(HashMap::new()),
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn generate_gid(&self) -> GlobalTransactionId {
        GlobalTransactionId::new(self.node_id)
    }

    pub fn begin_transaction(
        &self,
        gid: GlobalTransactionId,
    ) -> Result<(), String> {
        let ctx = TransactionContext::new(gid.clone());
        self.pending_transactions
            .write()
            .map_err(|_| "Lock poisoned")?
            .insert(gid, ctx);
        Ok(())
    }

    pub fn get_state(&self, gid: &GlobalTransactionId) -> Option<DistributedTransactionState> {
        self.pending_transactions
            .read()
            .ok()
            .and_then(|map| map.get(gid).map(|ctx| ctx.state))
    }

    pub async fn prepare(
        &self,
        gid: &GlobalTransactionId,
        participants: &[u64],
    ) -> Result<PrepareResult, String> {
        // 更新状态为 Preparing
        {
            let mut map = self.pending_transactions.write().map_err(|_| "Lock poisoned")?;
            if let Some(ctx) = map.get_mut(gid) {
                ctx.state = DistributedTransactionState::Preparing;
                for &p in participants {
                    ctx.add_participant(p);
                }
            }
        }

        // TODO: 发送 Prepare 请求给所有参与者
        // 暂时返回 AllCommitted 以便编译通过
        let all_commit = true;

        let mut map = self.pending_transactions.write().map_err(|_| "Lock poisoned")?;
        if let Some(ctx) = map.get_mut(gid) {
            if all_commit {
                ctx.state = DistributedTransactionState::Prepared;
                Ok(PrepareResult::AllCommitted)
            } else {
                ctx.state = DistributedTransactionState::RollingBack;
                Ok(PrepareResult::NeedsRollback)
            }
        } else {
            Err("Transaction not found".to_string())
        }
    }

    pub async fn commit(&self, gid: &GlobalTransactionId) -> Result<CommitResult, String> {
        let mut map = self.pending_transactions.write().map_err(|_| "Lock poisoned")?;
        if let Some(ctx) = map.get_mut(gid) {
            ctx.state = DistributedTransactionState::Committing;

            // TODO: 发送 Commit 请求给所有参与者

            ctx.state = DistributedTransactionState::Committed;
            map.remove(gid);
            Ok(CommitResult { success: true })
        } else {
            Err("Transaction not found".to_string())
        }
    }

    pub async fn rollback(&self, gid: &GlobalTransactionId, reason: &str) -> Result<(), String> {
        let mut map = self.pending_transactions.write().map_err(|_| "Lock poisoned")?;
        if let Some(ctx) = map.get_mut(gid) {
            ctx.state = DistributedTransactionState::RollingBack;

            // TODO: 发送 Rollback 请求给所有参与者

            ctx.state = DistributedTransactionState::RolledBack;
            map.remove(gid);
            Ok(())
        } else {
            Err("Transaction not found".to_string())
        }
    }
}

pub enum PrepareResult {
    AllCommitted,
    NeedsRollback,
}

pub struct CommitResult {
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_initialization() {
        let node_id = NodeId(1);
        let coordinator = Coordinator::new(node_id);
        assert_eq!(coordinator.node_id(), node_id);
    }

    #[tokio::test]
    async fn test_begin_distributed_transaction() {
        let node_id = NodeId(1);
        let coordinator = Coordinator::new(node_id);
        let gid = coordinator.generate_gid();
        assert_eq!(gid.node_id, node_id);
    }

    #[tokio::test]
    async fn test_coordinator_state_transitions() {
        let coordinator = Coordinator::new(NodeId(1));
        let gid = coordinator.generate_gid();

        coordinator.begin_transaction(gid.clone()).unwrap();
        assert_eq!(coordinator.get_state(&gid), Some(DistributedTransactionState::Started));
    }

    #[tokio::test]
    async fn test_coordinator_prepare_success() {
        let coordinator = Coordinator::new(NodeId(1));
        let gid = coordinator.generate_gid();

        coordinator.begin_transaction(gid.clone()).unwrap();
        let participants = vec![2, 3];

        let result = coordinator.prepare(&gid, &participants).await.unwrap();
        assert!(matches!(result, PrepareResult::AllCommitted));
        assert_eq!(coordinator.get_state(&gid), Some(DistributedTransactionState::Prepared));
    }

    #[tokio::test]
    async fn test_coordinator_commit() {
        let coordinator = Coordinator::new(NodeId(1));
        let gid = coordinator.generate_gid();

        coordinator.begin_transaction(gid.clone()).unwrap();
        let participants = vec![2];
        coordinator.prepare(&gid, &participants).await.unwrap();

        let result = coordinator.commit(&gid).await.unwrap();
        assert!(result.success);
        assert_eq!(coordinator.get_state(&gid), None); // 已移除
    }

    #[tokio::test]
    async fn test_coordinator_rollback() {
        let coordinator = Coordinator::new(NodeId(1));
        let gid = coordinator.generate_gid();

        coordinator.begin_transaction(gid.clone()).unwrap();
        let participants = vec![2];
        coordinator.prepare(&gid, &participants).await.unwrap();

        coordinator.rollback(&gid, "User requested").await.unwrap();
        assert_eq!(coordinator.get_state(&gid), None); // 已移除
    }
}