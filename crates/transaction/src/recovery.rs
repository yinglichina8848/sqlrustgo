use crate::gid::GlobalTransactionId;
use serde::{Deserialize, Serialize};

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
    // TODO: 需要 WALManager 的引用
}

impl Recovery {
    pub fn new() -> Self {
        Recovery {}
    }

    /// 扫描未完成的事务
    pub async fn scan_incomplete_transactions(&self) -> Result<Vec<WalEntry>, String> {
        // TODO: 从 WAL 读取所有未 Commit 的条目
        Ok(Vec::new())
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
                WalEntry::TxPrepare { gid, participants, .. } => {
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

    async fn rollback_incomplete_tx(&self, gid: &GlobalTransactionId, reason: &str) -> Result<(), String> {
        // TODO: 向参与者发送 Rollback 请求
        Ok(())
    }

    async fn query_coordinator_for_outcome(&self, gid: &GlobalTransactionId, _participants: &[u64]) -> Result<TxOutcome, String> {
        // TODO: 查询协调者事务状态
        // 暂时返回 Unknown
        Ok(TxOutcome::Unknown)
    }

    async fn mark_committed(&self, gid: &GlobalTransactionId) -> Result<(), String> {
        // TODO: 更新 WAL 条目状态
        Ok(())
    }

    async fn mark_rolled_back(&self, gid: &GlobalTransactionId) -> Result<(), String> {
        // TODO: 更新 WAL 条目状态
        Ok(())
    }

    async fn mark_terminated(&self, gid: &GlobalTransactionId, reason: &str) -> Result<(), String> {
        // TODO: 记录 TxTerminate 日志
        Ok(())
    }
}

impl Default for Recovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_report_default() {
        let report = RecoveryReport::default();
        assert_eq!(report.committed, 0);
        assert_eq!(report.rolled_back, 0);
        assert_eq!(report.terminated, 0);
    }

    #[tokio::test]
    async fn test_scan_incomplete_transactions() {
        let recovery = Recovery::new();
        let incomplete = recovery.scan_incomplete_transactions().await.unwrap();
        // 初始化时应该没有未完成的事务
        assert!(incomplete.is_empty());
    }

    #[tokio::test]
    async fn test_recovery_empty_wal() {
        let recovery = Recovery::new();
        let report = recovery.recover().await.unwrap();
        assert_eq!(report.committed, 0);
        assert_eq!(report.rolled_back, 0);
        assert_eq!(report.terminated, 0);
    }
}