use crate::gid::GlobalTransactionId;
use crate::gid::NodeId;
use serde::{Deserialize, Serialize};
use sqlrustgo_storage::{WalEntryType, WalManager};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// 恢复报告
#[derive(Debug, Default)]
pub struct RecoveryReport {
    pub committed: u32,
    pub rolled_back: u32,
    pub terminated: u32,
}

/// 事务结果
#[derive(Debug)]
pub enum TxOutcome {
    Committed,
    RolledBack,
    Unknown,
}

/// WAL 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalEntry {
    TxBegin {
        gid: GlobalTransactionId,
        timestamp: u64,
    },
    TxPrepare {
        gid: GlobalTransactionId,
        participants: Vec<u64>,
        timestamp: u64,
    },
    TxCommit {
        gid: GlobalTransactionId,
        timestamp: u64,
    },
    TxRollback {
        gid: GlobalTransactionId,
        reason: String,
        timestamp: u64,
    },
    TxTerminate {
        gid: GlobalTransactionId,
        reason: String,
        timestamp: u64,
    },
}

/// 故障恢复组件
pub struct Recovery {
    /// WAL manager for reading transaction logs
    wal_manager: Option<Arc<WalManager>>,
    /// Node ID for this participant
    node_id: NodeId,
    /// In-memory transaction state tracking (tx_id -> state)
    /// Used during recovery scanning
    tx_states: RwLock<HashMap<u64, TxState>>,
}

/// Internal transaction state during recovery
#[derive(Debug, Clone)]
struct TxState {
    gid: GlobalTransactionId,
    has_prepare: bool,
    has_commit: bool,
    has_rollback: bool,
    participants: Vec<u64>,
    timestamp: u64,
}

impl Recovery {
    /// Create a new Recovery without WAL (in-memory only)
    pub fn new(node_id: NodeId) -> Self {
        Recovery {
            wal_manager: None,
            node_id,
            tx_states: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new Recovery with WAL manager for durability
    pub fn with_wal(node_id: NodeId, wal_path: PathBuf) -> Self {
        Recovery {
            wal_manager: Some(Arc::new(WalManager::new(wal_path))),
            node_id,
            tx_states: RwLock::new(HashMap::new()),
        }
    }

    /// Convert GID to transaction ID (hash)
    fn gid_to_tx_id(gid: &GlobalTransactionId) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        gid.to_string().hash(&mut hasher);
        hasher.finish()
    }

    /// Log transaction begin to WAL
    pub fn log_begin(&self, gid: &GlobalTransactionId) -> Result<(), String> {
        if let Some(ref wal) = self.wal_manager {
            let tx_id = Self::gid_to_tx_id(gid);
            wal.log_begin(tx_id)
                .map_err(|e| format!("WAL error: {}", e))?;
        }
        Ok(())
    }

    /// Log transaction prepare to WAL
    pub fn log_prepare(
        &self,
        gid: &GlobalTransactionId,
        participants: &[u64],
    ) -> Result<(), String> {
        if let Some(ref wal) = self.wal_manager {
            let tx_id = Self::gid_to_tx_id(gid);
            wal.log_prepare(tx_id)
                .map_err(|e| format!("WAL error: {}", e))?;
        }
        // Update in-memory state
        let tx_id = Self::gid_to_tx_id(gid);
        let mut states = self
            .tx_states
            .write()
            .map_err(|_| "Lock poisoned".to_string())?;
        states.insert(
            tx_id,
            TxState {
                gid: gid.clone(),
                has_prepare: true,
                has_commit: false,
                has_rollback: false,
                participants: participants.to_vec(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        );
        Ok(())
    }

    /// Log transaction commit to WAL
    pub fn log_commit(&self, gid: &GlobalTransactionId) -> Result<(), String> {
        if let Some(ref wal) = self.wal_manager {
            let tx_id = Self::gid_to_tx_id(gid);
            wal.log_commit(tx_id)
                .map_err(|e| format!("WAL error: {}", e))?;
        }
        // Update in-memory state
        let tx_id = Self::gid_to_tx_id(gid);
        let mut states = self
            .tx_states
            .write()
            .map_err(|_| "Lock poisoned".to_string())?;
        if let Some(state) = states.get_mut(&tx_id) {
            state.has_commit = true;
        }
        Ok(())
    }

    /// Log transaction rollback to WAL
    pub fn log_rollback(&self, gid: &GlobalTransactionId, _reason: &str) -> Result<(), String> {
        if let Some(ref wal) = self.wal_manager {
            let tx_id = Self::gid_to_tx_id(gid);
            wal.log_rollback(tx_id)
                .map_err(|e| format!("WAL error: {}", e))?;
        }
        // Update in-memory state
        let tx_id = Self::gid_to_tx_id(gid);
        let mut states = self
            .tx_states
            .write()
            .map_err(|_| "Lock poisoned".to_string())?;
        if let Some(state) = states.get_mut(&tx_id) {
            state.has_rollback = true;
        }
        Ok(())
    }

    /// Scan WAL for incomplete transactions (those with Prepare but no Commit/Rollback)
    pub async fn scan_incomplete_transactions(&self) -> Result<Vec<WalEntry>, String> {
        let mut incomplete = Vec::new();

        // If we have WAL, recover transactions from it
        if let Some(ref wal) = self.wal_manager {
            // Handle case where WAL file doesn't exist yet (not an error, just no entries)
            let entries = match wal.recover() {
                Ok(entries) => entries,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
                Err(e) => return Err(format!("WAL recovery error: {}", e)),
            };

            // Build transaction state map from WAL entries
            let mut tx_map: HashMap<u64, TxState> = HashMap::new();

            for entry in entries {
                let state = tx_map.entry(entry.tx_id).or_insert_with(|| TxState {
                    gid: GlobalTransactionId {
                        node_id: self.node_id,
                        timestamp: entry.timestamp,
                        txn_id: entry.tx_id,
                    },
                    has_prepare: false,
                    has_commit: false,
                    has_rollback: false,
                    participants: Vec::new(),
                    timestamp: entry.timestamp,
                });

                match entry.entry_type {
                    WalEntryType::Begin => {
                        // Transaction began - will be incomplete unless we see prepare/commit/rollback
                        state.timestamp = entry.timestamp;
                    }
                    WalEntryType::Prepare => {
                        state.has_prepare = true;
                    }
                    WalEntryType::Commit => {
                        state.has_commit = true;
                    }
                    WalEntryType::Rollback => {
                        state.has_rollback = true;
                    }
                    _ => {}
                }
            }

            // Collect incomplete transactions
            for (_tx_id, state) in tx_map {
                // Transaction is incomplete if:
                // 1. It has Prepare but no Commit and no Rollback (prepared but not decided)
                // 2. It has no Prepare and no Commit and no Rollback (started but never prepared)
                if state.has_prepare && !state.has_commit && !state.has_rollback {
                    incomplete.push(WalEntry::TxPrepare {
                        gid: state.gid,
                        participants: state.participants,
                        timestamp: state.timestamp,
                    });
                } else if !state.has_prepare && !state.has_commit && !state.has_rollback {
                    incomplete.push(WalEntry::TxBegin {
                        gid: state.gid,
                        timestamp: state.timestamp,
                    });
                }
            }
        }

        // Also check in-memory state for any transactions that were logged but not yet committed
        {
            let states = self
                .tx_states
                .read()
                .map_err(|_| "Lock poisoned".to_string())?;
            for (tx_id, state) in states.iter() {
                if state.has_prepare && !state.has_commit && !state.has_rollback {
                    // Check if not already in incomplete list
                    if !incomplete.iter().any(|e| {
                        if let WalEntry::TxPrepare { gid, .. } = e {
                            Self::gid_to_tx_id(gid) == *tx_id
                        } else {
                            false
                        }
                    }) {
                        incomplete.push(WalEntry::TxPrepare {
                            gid: state.gid.clone(),
                            participants: state.participants.clone(),
                            timestamp: state.timestamp,
                        });
                    }
                }
            }
        }

        Ok(incomplete)
    }

    /// 执行恢复
    pub async fn recover(&self) -> Result<RecoveryReport, String> {
        let mut report = RecoveryReport::default();

        // 扫描未完成的事务
        let incomplete_txs = self.scan_incomplete_transactions().await?;

        for entry in incomplete_txs {
            match entry {
                WalEntry::TxBegin { gid, .. } => {
                    // 从未完成 Phase 1，回滚
                    self.rollback_incomplete_tx(&gid, "Node crash before prepare")
                        .await?;
                    report.rolled_back += 1;
                }
                WalEntry::TxPrepare {
                    gid, participants, ..
                } => {
                    // 等待协调者指令或主动查询
                    let outcome = self
                        .query_coordinator_for_outcome(&gid, &participants)
                        .await?;
                    match outcome {
                        TxOutcome::Committed => {
                            self.mark_committed(&gid).await?;
                            report.committed += 1;
                        }
                        TxOutcome::RolledBack => {
                            self.mark_rolled_back(&gid).await?;
                            report.rolled_back += 1;
                        }
                        TxOutcome::Unknown => {
                            self.mark_terminated(&gid, "Coordinator uncertain").await?;
                            report.terminated += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(report)
    }

    /// 向参与者发送 Rollback 请求
    async fn rollback_incomplete_tx(
        &self,
        gid: &GlobalTransactionId,
        reason: &str,
    ) -> Result<(), String> {
        log::info!("Rolling back incomplete transaction {}: {}", gid, reason);

        // 记录回滚日志
        self.log_rollback(gid, reason)?;

        // 在实际实现中，这里会向参与者发送 Rollback 请求
        // 目前只是记录日志
        log::debug!("Rollback request sent for GID: {}", gid);
        Ok(())
    }

    /// 查询协调者事务状态
    async fn query_coordinator_for_outcome(
        &self,
        gid: &GlobalTransactionId,
        _participants: &[u64],
    ) -> Result<TxOutcome, String> {
        log::debug!("Querying coordinator for transaction outcome: {}", gid);

        // 在实际实现中，这里会查询协调者获取事务状态
        // 暂时返回 Unknown，表示无法确定
        Ok(TxOutcome::Unknown)
    }

    /// 标记事务为已提交
    async fn mark_committed(&self, gid: &GlobalTransactionId) -> Result<(), String> {
        log::info!("Marking transaction as committed: {}", gid);

        // 记录提交日志
        self.log_commit(gid)?;

        // 清理内存中的状态
        let tx_id = Self::gid_to_tx_id(gid);
        let mut states = self
            .tx_states
            .write()
            .map_err(|_| "Lock poisoned".to_string())?;
        states.remove(&tx_id);

        Ok(())
    }

    /// 标记事务为已回滚
    async fn mark_rolled_back(&self, gid: &GlobalTransactionId) -> Result<(), String> {
        log::info!("Marking transaction as rolled back: {}", gid);

        // 记录回滚日志
        self.log_rollback(gid, "Recovery rollback")?;

        // 清理内存中的状态
        let tx_id = Self::gid_to_tx_id(gid);
        let mut states = self
            .tx_states
            .write()
            .map_err(|_| "Lock poisoned".to_string())?;
        states.remove(&tx_id);

        Ok(())
    }

    /// 标记事务为已终止
    async fn mark_terminated(&self, gid: &GlobalTransactionId, reason: &str) -> Result<(), String> {
        log::warn!("Marking transaction as terminated: {} - {}", gid, reason);

        // 在实际实现中，应该记录 TxTerminate 日志到 WAL
        // 目前只是记录日志
        log::debug!("Transaction terminated: {} - {}", gid, reason);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_recovery_report_default() {
        let report = RecoveryReport::default();
        assert_eq!(report.committed, 0);
        assert_eq!(report.rolled_back, 0);
        assert_eq!(report.terminated, 0);
    }

    #[tokio::test]
    async fn test_scan_incomplete_transactions() {
        let node_id = NodeId(1);
        let recovery = Recovery::new(node_id);
        let incomplete = recovery.scan_incomplete_transactions().await.unwrap();
        // 初始化时应该没有未完成的事务
        assert!(incomplete.is_empty());
    }

    #[tokio::test]
    async fn test_recovery_empty_wal() {
        let node_id = NodeId(1);
        let recovery = Recovery::new(node_id);
        let report = recovery.recover().await.unwrap();
        assert_eq!(report.committed, 0);
        assert_eq!(report.rolled_back, 0);
        assert_eq!(report.terminated, 0);
    }

    #[tokio::test]
    async fn test_recovery_with_wal() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("recovery.wal");
        let node_id = NodeId(1);
        let recovery = Recovery::with_wal(node_id, wal_path);

        // Recovery should be created with WAL manager
        let incomplete = recovery.scan_incomplete_transactions().await.unwrap();
        assert!(incomplete.is_empty());
    }

    #[tokio::test]
    async fn test_recovery_log_begin() {
        let node_id = NodeId(1);
        let recovery = Recovery::new(node_id);
        let gid = GlobalTransactionId::new(node_id);

        let result = recovery.log_begin(&gid);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_recovery_log_prepare_and_commit() {
        let node_id = NodeId(1);
        let recovery = Recovery::new(node_id);
        let gid = GlobalTransactionId::new(node_id);

        // Log prepare
        let result = recovery.log_prepare(&gid, &[2, 3]);
        assert!(result.is_ok());

        // Log commit
        let result = recovery.log_commit(&gid);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_recovery_log_rollback() {
        let node_id = NodeId(1);
        let recovery = Recovery::new(node_id);
        let gid = GlobalTransactionId::new(node_id);

        // Log rollback
        let result = recovery.log_rollback(&gid, "User requested");
        assert!(result.is_ok());
    }
}
