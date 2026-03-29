use crate::dtc::{Change, DistributedTransactionState, TransactionContext};
use crate::gid::{GlobalTransactionId, NodeId};
use sqlrustgo_storage::{WalEntryType, WalManager};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Participant with WAL-based transaction management
pub struct Participant {
    node_id: NodeId,
    local_transactions: RwLock<HashMap<GlobalTransactionId, TransactionContext>>,
    /// WAL manager for durability
    wal_manager: Option<Arc<WalManager>>,
    /// Locks for distributed transactions (GID -> lock)
    locks: RwLock<HashMap<GlobalTransactionId, ()>>,
}

impl Participant {
    /// Create a new Participant without WAL (in-memory only)
    pub fn new(node_id: NodeId) -> Self {
        Participant {
            node_id,
            local_transactions: RwLock::new(HashMap::new()),
            wal_manager: None,
            locks: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new Participant with WAL manager for durability
    pub fn with_wal(node_id: NodeId, wal_path: PathBuf) -> Self {
        Participant {
            node_id,
            local_transactions: RwLock::new(HashMap::new()),
            wal_manager: Some(Arc::new(WalManager::new(wal_path))),
            locks: RwLock::new(HashMap::new()),
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Get a unique transaction ID from GID
    fn gid_to_tx_id(gid: &GlobalTransactionId) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        gid.to_string().hash(&mut hasher);
        hasher.finish()
    }

    pub async fn handle_prepare(&self, request: PrepareRequest) -> Result<VoteResponse, String> {
        let gid = GlobalTransactionId::parse(&request.gid).map_err(|e| e.to_string())?;

        // 尝试获取锁 - 如果已有锁则失败
        {
            let mut locks = self.locks.write().map_err(|_| "Lock poisoned")?;
            if locks.contains_key(&gid) {
                return Ok(VoteResponse {
                    gid: request.gid.clone(),
                    node_id: self.node_id.0.to_string(),
                    vote: VoteType::VoteAbort as i32,
                    reason: Some("Transaction already in progress".to_string()),
                });
            }
            locks.insert(gid.clone(), ());
        }

        // 创建本地事务上下文
        let mut ctx = TransactionContext::new(gid.clone());
        ctx.state = DistributedTransactionState::Preparing;
        ctx.changes = request.changes.clone();

        // 记录 Prepare 日志到 WAL
        if let Some(ref wal) = self.wal_manager {
            let tx_id = Self::gid_to_tx_id(&gid);
            wal.log_prepare(tx_id)
                .map_err(|e| format!("WAL error: {}", e))?;
        }

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

    pub async fn handle_commit(&self, request: CommitRequest) -> Result<ExecutionResponse, String> {
        let gid = GlobalTransactionId::parse(&request.gid).map_err(|e| e.to_string())?;

        // 从 WAL 恢复事务状态
        let mut recovered_ctx = None;
        if let Some(ref wal) = self.wal_manager {
            let tx_id = Self::gid_to_tx_id(&gid);
            let entries = wal
                .recover()
                .map_err(|e| format!("WAL recovery error: {}", e))?;
            // 查找该事务的 Prepare 条目
            for entry in entries {
                if entry.tx_id == tx_id && entry.entry_type == WalEntryType::Prepare {
                    // 找到 Prepare 条目，可以提交
                    recovered_ctx = Some(());
                    break;
                }
            }
        }

        // 如果没有 WAL 或没有找到恢复状态，使用内存中的上下文
        if recovered_ctx.is_none() {
            let ctx = self
                .local_transactions
                .read()
                .map_err(|_| "Lock poisoned")?
                .get(&gid)
                .cloned();
            if ctx.is_none() {
                return Err("Transaction not found".to_string());
            }
        }

        // 执行本地提交 (在实际实现中，这里会提交数据变更)
        log::debug!("Executing local commit for GID: {}", gid);

        // 记录提交日志到 WAL
        if let Some(ref wal) = self.wal_manager {
            let tx_id = Self::gid_to_tx_id(&gid);
            wal.log_commit(tx_id)
                .map_err(|e| format!("WAL error: {}", e))?;
        }

        // 释放锁
        {
            let mut locks = self.locks.write().map_err(|_| "Lock poisoned")?;
            locks.remove(&gid);
        }

        // 清理 WAL (在实际实现中，可能需要归档或截断 WAL)
        log::debug!("Cleaning up WAL for GID: {}", gid);

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
        let gid = GlobalTransactionId::parse(&request.gid).map_err(|e| e.to_string())?;

        // 检查事务是否存在
        {
            let ctx = self
                .local_transactions
                .read()
                .map_err(|_| "Lock poisoned")?
                .get(&gid)
                .cloned();
            if ctx.is_none() {
                return Err("Transaction not found".to_string());
            }
        }

        // 执行本地回滚 (在实际实现中，这里会撤销数据变更)
        log::debug!(
            "Executing local rollback for GID: {}, reason: {}",
            gid,
            request.reason
        );

        // 记录回滚日志到 WAL
        if let Some(ref wal) = self.wal_manager {
            let tx_id = Self::gid_to_tx_id(&gid);
            wal.log_rollback(tx_id)
                .map_err(|e| format!("WAL error: {}", e))?;
        }

        // 释放锁
        {
            let mut locks = self.locks.write().map_err(|_| "Lock poisoned")?;
            locks.remove(&gid);
        }

        // 清理 WAL
        log::debug!("Cleaning up WAL after rollback for GID: {}", gid);

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
    use tempfile::tempdir;

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

        // First prepare the transaction
        let prepare_request = PrepareRequest {
            gid: "1:1:1000".to_string(),
            coordinator_node_id: "2".to_string(),
            changes: vec![],
        };
        participant.handle_prepare(prepare_request).await.unwrap();

        // Then commit
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

        // First prepare the transaction
        let prepare_request = PrepareRequest {
            gid: "1:1:1000".to_string(),
            coordinator_node_id: "2".to_string(),
            changes: vec![],
        };
        participant.handle_prepare(prepare_request).await.unwrap();

        // Then rollback
        let request = RollbackRequest {
            gid: "1:1:1000".to_string(),
            reason: "User requested".to_string(),
        };

        let response = participant.handle_rollback(request).await.unwrap();
        assert!(response.success);
    }

    #[tokio::test]
    async fn test_participant_with_wal() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("test.wal");
        let node_id = NodeId(1);
        let participant = Participant::with_wal(node_id, wal_path);

        let request = PrepareRequest {
            gid: "1:1:2000".to_string(),
            coordinator_node_id: "2".to_string(),
            changes: vec![],
        };

        let response = participant.handle_prepare(request).await.unwrap();
        assert_eq!(response.vote, VoteType::VoteCommit as i32);
    }

    #[tokio::test]
    async fn test_participant_lock_contention() {
        let node_id = NodeId(1);
        let participant = Participant::new(node_id);

        let request = PrepareRequest {
            gid: "1:1:3000".to_string(),
            coordinator_node_id: "2".to_string(),
            changes: vec![],
        };

        // 第一次 prepare 应该成功
        let response1 = participant.handle_prepare(request.clone()).await.unwrap();
        assert_eq!(response1.vote, VoteType::VoteCommit as i32);

        // 第二次 prepare 同一事务应该被拒绝
        let response2 = participant.handle_prepare(request).await.unwrap();
        assert_eq!(response2.vote, VoteType::VoteAbort as i32);
    }

    #[tokio::test]
    async fn test_participant_rollback_unknown_tx() {
        let node_id = NodeId(1);
        let participant = Participant::new(node_id);

        let request = RollbackRequest {
            gid: "1:1:9999".to_string(),
            reason: "Test".to_string(),
        };

        let response = participant.handle_rollback(request).await;
        assert!(response.is_err());
    }
}
