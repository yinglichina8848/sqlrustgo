//! Cancel Token - 协作式取消机制
//!
//! 参考 MySQL 和 PostgreSQL 的 KILL 实现设计
//! - MySQL: killed_state (NOT_KILLED, KILL_QUERY, KILL_CONNECTION)
//! - PostgreSQL: QueryCancelPending, ProcDiePending
//!
//! 核心原则：协作式取消，而非强制终止

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// CancelToken - 协作式取消标志
///
/// 用于在 Session 层面管理查询取消状态
///
/// # 与 PostgreSQL 对比
/// - QueryCancelPending → cancel_query()
/// - ProcDiePending → kill_connection()
#[derive(Debug)]
pub struct CancelToken {
    /// 取消当前查询（KILL QUERY）
    /// 设置后，查询应在下一个检查点中止
    query_cancelled: Arc<AtomicBool>,

    /// 终止会话（KILL CONNECTION）
    /// 设置后，整个会话应立即终止
    connection_killed: Arc<AtomicBool>,
}

impl CancelToken {
    /// 创建新的 CancelToken
    pub fn new() -> Self {
        Self {
            query_cancelled: Arc::new(AtomicBool::new(false)),
            connection_killed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 取消当前查询（KILL QUERY）
    ///
    /// 设置后，查询应在下一个检查点中止
    /// 连接保持开放，可接受新查询
    pub fn cancel_query(&self) {
        self.query_cancelled.store(true, Ordering::SeqCst);
    }

    /// 检查查询是否被取消
    ///
    /// 由查询执行循环在检查点调用
    pub fn is_query_cancelled(&self) -> bool {
        self.query_cancelled.load(Ordering::SeqCst)
    }

    /// 终止连接（KILL CONNECTION）
    ///
    /// 设置后，整个会话应立即终止
    /// 所有正在执行的查询都应中止
    pub fn kill_connection(&self) {
        self.connection_killed.store(true, Ordering::SeqCst);
    }

    /// 检查连接是否被终止
    pub fn is_connection_killed(&self) -> bool {
        self.connection_killed.load(Ordering::SeqCst)
    }

    /// 重置查询取消标志（用于新的查询）
    ///
    /// 在会话开始执行新查询时调用
    pub fn reset_query_cancelled(&self) {
        self.query_cancelled.store(false, Ordering::SeqCst);
    }

    /// 检查是否正在执行查询
    ///
    /// 即查询取消标志未设置且连接未终止
    pub fn is_active(&self) -> bool {
        !self.query_cancelled.load(Ordering::SeqCst)
            && !self.connection_killed.load(Ordering::SeqCst)
    }

    pub fn query_cancelled_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.query_cancelled)
    }

    pub fn connection_killed_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.connection_killed)
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

/// CancelGuard - 临界区取消保护
///
/// 参考 PostgreSQL CancelGuard 设计
///
/// 用于在关键操作期间（如 metadata swap、table rewrite）临时禁用取消
///
/// # 警告
/// 临界区操作必须是原子的，不能在中间被取消
pub struct CancelGuard {
    token: Arc<CancelToken>,
    previous_state: bool,
}

impl CancelGuard {
    /// 禁用取消（进入临界区）
    ///
    /// 保存当前取消状态，设置取消标志
    /// 这样即使收到 KILL 信号，也不会在临界区期间实际取消
    pub fn disable(token: Arc<CancelToken>) -> Self {
        let previous_state = token.query_cancelled.swap(true, Ordering::SeqCst);
        Self {
            token,
            previous_state,
        }
    }

    /// 重新启用取消（离开临界区）
    ///
    /// 恢复之前的取消状态
    pub fn enable(self) {
        self.token
            .query_cancelled
            .store(self.previous_state, Ordering::SeqCst);
    }

    /// 检查之前的取消状态
    pub fn was_cancelled(&self) -> bool {
        self.previous_state
    }
}

impl Drop for CancelGuard {
    fn drop(&mut self) {
        // 确保离开临界区时恢复取消状态
        self.token
            .query_cancelled
            .store(self.previous_state, Ordering::SeqCst);
    }
}

/// CheckCancel - 查询执行中的取消检查点
///
/// 在长时间运行的查询操作中插入检查点
#[derive(Debug, Clone)]
pub struct SqlError {
    pub message: String,
}

impl SqlError {
    pub fn query_cancelled() -> Self {
        Self {
            message: "Query cancelled".to_string(),
        }
    }

    pub fn connection_killed() -> Self {
        Self {
            message: "Connection killed".to_string(),
        }
    }
}

/// 检查取消状态，返回错误如果已被取消
#[inline]
pub fn check_cancel(cancel_token: &Arc<CancelToken>) -> Result<(), SqlError> {
    if cancel_token.is_connection_killed() {
        return Err(SqlError::connection_killed());
    }
    if cancel_token.is_query_cancelled() {
        return Err(SqlError::query_cancelled());
    }
    Ok(())
}

/// 检查取消状态，但允许在临界区中被覆盖
#[inline]
pub fn check_cancel_guarded(guard: &CancelGuard) -> Result<(), SqlError> {
    if guard.was_cancelled() {
        return Err(SqlError::query_cancelled());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancel_token_new() {
        let token = CancelToken::new();
        assert!(!token.is_query_cancelled());
        assert!(!token.is_connection_killed());
        assert!(token.is_active());
    }

    #[test]
    fn test_cancel_query() {
        let token = CancelToken::new();
        token.cancel_query();
        assert!(token.is_query_cancelled());
        assert!(!token.is_connection_killed());
        assert!(!token.is_active());
    }

    #[test]
    fn test_kill_connection() {
        let token = CancelToken::new();
        token.kill_connection();
        assert!(!token.is_query_cancelled());
        assert!(token.is_connection_killed());
        assert!(!token.is_active());
    }

    #[test]
    fn test_reset_query_cancelled() {
        let token = CancelToken::new();
        token.cancel_query();
        assert!(token.is_query_cancelled());
        token.reset_query_cancelled();
        assert!(!token.is_query_cancelled());
        assert!(token.is_active());
    }

    #[test]
    fn test_cancel_guard() {
        let token = Arc::new(CancelToken::new());
        assert!(token.is_active());

        // 进入临界区
        let guard = CancelGuard::disable(token.clone());
        assert!(!token.is_active()); // 已被禁用

        // 离开临界区
        drop(guard);
        assert!(token.is_active()); // 恢复
    }

    #[test]
    fn test_cancel_guard_preserves_previous_state() {
        let token = Arc::new(CancelToken::new());

        // 先取消查询
        token.cancel_query();
        assert!(token.is_query_cancelled());

        // 进入临界区
        let guard = CancelGuard::disable(token.clone());

        // 离开临界区 - 应该恢复为已取消状态
        drop(guard);
        assert!(token.is_query_cancelled());
    }

    #[test]
    fn test_check_cancel() {
        let token = Arc::new(CancelToken::new());
        assert!(check_cancel(&token).is_ok());

        token.cancel_query();
        assert!(check_cancel(&token).is_err());

        let err = check_cancel(&token).unwrap_err();
        assert_eq!(err.message, "Query cancelled");
    }

    #[test]
    fn test_check_cancel_connection_killed() {
        let token = Arc::new(CancelToken::new());
        token.kill_connection();

        let err = check_cancel(&token).unwrap_err();
        assert_eq!(err.message, "Connection killed");
    }
}
