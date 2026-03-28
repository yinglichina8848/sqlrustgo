use crate::dtc::{Change, DistributedTransactionState, TransactionContext};
use crate::gid::{GlobalTransactionId, NodeId};
use std::collections::HashMap;
use std::sync::RwLock;

/// 事务参与者
pub struct Participant {
    node_id: NodeId,
    local_transactions: RwLock<HashMap<GlobalTransactionId, TransactionContext>>,
}

impl Participant {
    pub fn new(node_id: NodeId) -> Self {
        Participant {
            node_id,
            local_transactions: RwLock::new(HashMap::new()),
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub async fn handle_prepare(
        &self,
        request: PrepareRequest,
    ) -> Result<VoteResponse, String> {
        let gid = GlobalTransactionId::parse(&request.gid)
            .map_err(|e| e.to_string())?;

        // 创建本地事务上下文
        let mut ctx = TransactionContext::new(gid.clone());
        ctx.state = DistributedTransactionState::Preparing;

        // TODO: 尝试获取锁
        // TODO: 记录 Prepare 日志到 WAL

        self.local_transactions
            .write()
            .map_err(|_| "Lock poisoned")?
            .insert(gid.clone(), ctx);

        Ok(VoteResponse {
            gid: request.gid,
            node_id: self.node_id.0.to_string(),
            vote: VoteType::VoteCommit as i32,
            reason: None,
        })
    }

    pub async fn handle_commit(
        &self,
        request: CommitRequest,
    ) -> Result<ExecutionResponse, String> {
        let gid = GlobalTransactionId::parse(&request.gid)
            .map_err(|e| e.to_string())?;

        // TODO: 从 WAL 恢复事务状态
        // TODO: 执行本地提交
        // TODO: 释放锁
        // TODO: 清理 WAL

        self.local_transactions
            .write()
            .map_err(|_| "Lock poisoned")?
            .remove(&gid);

        Ok(ExecutionResponse {
            gid: request.gid,
            node_id: self.node_id.0.to_string(),
            success: true,
            affected_rows: 0,
            error: None,
        })
    }

    pub async fn handle_rollback(
        &self,
        request: RollbackRequest,
    ) -> Result<ExecutionResponse, String> {
        let gid = GlobalTransactionId::parse(&request.gid)
            .map_err(|e| e.to_string())?;

        // TODO: 执行本地回滚
        // TODO: 释放锁
        // TODO: 清理 WAL

        self.local_transactions
            .write()
            .map_err(|_| "Lock poisoned")?
            .remove(&gid);

        Ok(ExecutionResponse {
            gid: request.gid,
            node_id: self.node_id.0.to_string(),
            success: true,
            affected_rows: 0,
            error: None,
        })
    }
}

// gRPC 请求/响应类型
#[derive(Debug, Clone)]
pub struct PrepareRequest {
    pub gid: String,
    pub coordinator_node_id: String,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone)]
pub struct VoteResponse {
    pub gid: String,
    pub node_id: String,
    pub vote: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CommitRequest {
    pub gid: String,
}

#[derive(Debug, Clone)]
pub struct RollbackRequest {
    pub gid: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ExecutionResponse {
    pub gid: String,
    pub node_id: String,
    pub success: bool,
    pub affected_rows: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum VoteType {
    VoteCommit = 0,
    VoteAbort = 1,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_participant_initialization() {
        let node_id = NodeId(1);
        let participant = Participant::new(node_id);
        assert_eq!(participant.node_id(), node_id);
    }

    #[tokio::test]
    async fn test_participant_handle_prepare_commit() {
        let node_id = NodeId(1);
        let participant = Participant::new(node_id);

        let request = PrepareRequest {
            gid: "1:1:1000".to_string(),
            coordinator_node_id: "2".to_string(),
            changes: vec![],
        };

        let response = participant.handle_prepare(request).await.unwrap();
        assert_eq!(response.vote, VoteType::VoteCommit as i32);
    }

    #[tokio::test]
    async fn test_participant_handle_commit() {
        let node_id = NodeId(1);
        let participant = Participant::new(node_id);

        let request = CommitRequest {
            gid: "1:1:1000".to_string(),
        };

        let response = participant.handle_commit(request).await.unwrap();
        assert!(response.success);
    }

    #[tokio::test]
    async fn test_participant_handle_rollback() {
        let node_id = NodeId(1);
        let participant = Participant::new(node_id);

        let request = RollbackRequest {
            gid: "1:1:1000".to_string(),
            reason: "User requested".to_string(),
        };

        let response = participant.handle_rollback(request).await.unwrap();
        assert!(response.success);
    }
}