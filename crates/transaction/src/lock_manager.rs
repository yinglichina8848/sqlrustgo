use crate::gid::GlobalTransactionId;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// 锁键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LockKey {
    Table(String),
    Row { table: String, row_key: Vec<u8> },
}

/// 锁值 - 持有事务的 GID
#[derive(Debug, Clone)]
pub struct LockValue {
    pub gid: GlobalTransactionId,
    pub lock_mode: LockMode,
}

/// 锁模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    Shared,
    Exclusive,
}

/// 锁错误
#[derive(Debug, Clone)]
pub enum LockError {
    Conflict { held_by: GlobalTransactionId },
    Timeout,
}

/// 分布式锁管理器
pub struct DistributedLockManager {
    locks: RwLock<HashMap<LockKey, LockValue>>,
}

impl DistributedLockManager {
    pub fn new() -> Self {
        DistributedLockManager {
            locks: RwLock::new(HashMap::new()),
        }
    }

    /// 尝试获取锁
    pub async fn try_lock(
        &self,
        gid: &GlobalTransactionId,
        key: &LockKey,
    ) -> Result<(), LockError> {
        let mut locks = self.locks.write().await;

        if let Some(existing) = locks.get(key) {
            if existing.gid != *gid {
                return Err(LockError::Conflict {
                    held_by: existing.gid.clone(),
                });
            }
        }

        locks.insert(
            key.clone(),
            LockValue {
                gid: gid.clone(),
                lock_mode: LockMode::Exclusive,
            },
        );

        Ok(())
    }

    /// 释放锁
    pub async fn unlock(&self, gid: &GlobalTransactionId) -> Result<(), String> {
        let mut locks = self.locks.write().await;

        // 移除该事务持有的所有锁
        locks.retain(|_, v| v.gid != *gid);

        Ok(())
    }

    /// 检查锁是否被持有
    pub async fn is_locked(&self, key: &LockKey) -> bool {
        let locks = self.locks.read().await;
        locks.contains_key(key)
    }

    /// 获取锁的持有者
    pub async fn get_holder(&self, key: &LockKey) -> Option<GlobalTransactionId> {
        let locks = self.locks.read().await;
        locks.get(key).map(|v| v.gid.clone())
    }
}

impl Default for DistributedLockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lock_acquire_release() {
        let lock_mgr = DistributedLockManager::new();
        let gid = GlobalTransactionId::new(NodeId(1));

        let lock_key = LockKey::Table("users".to_string());
        let result = lock_mgr.try_lock(&gid, &lock_key).await;
        assert!(result.is_ok());

        lock_mgr.unlock(&gid).await.unwrap();
    }

    #[tokio::test]
    async fn test_lock_conflict() {
        let lock_mgr = DistributedLockManager::new();
        let gid1 = GlobalTransactionId::new(NodeId(1));
        let gid2 = GlobalTransactionId::new(NodeId(2));

        let lock_key = LockKey::Table("users".to_string());

        // 第一个事务获取锁
        let result1 = lock_mgr.try_lock(&gid1, &lock_key).await;
        assert!(result1.is_ok());

        // 第二个事务尝试获取同一把锁
        let result2 = lock_mgr.try_lock(&gid2, &lock_key).await;
        assert!(result2.is_err()); // 应该失败
    }

    #[tokio::test]
    async fn test_lock_same_gid_succeeds() {
        let lock_mgr = DistributedLockManager::new();
        let gid = GlobalTransactionId::new(NodeId(1));

        let lock_key = LockKey::Table("users".to_string());

        // 同一个事务获取同一把锁应该成功
        let result1 = lock_mgr.try_lock(&gid, &lock_key).await;
        assert!(result1.is_ok());

        let result2 = lock_mgr.try_lock(&gid, &lock_key).await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_is_locked() {
        let lock_mgr = DistributedLockManager::new();
        let gid = GlobalTransactionId::new(NodeId(1));

        let lock_key = LockKey::Table("users".to_string());
        assert!(!lock_mgr.is_locked(&lock_key).await);

        lock_mgr.try_lock(&gid, &lock_key).await.unwrap();
        assert!(lock_mgr.is_locked(&lock_key).await);
    }

    #[tokio::test]
    async fn test_get_holder() {
        let lock_mgr = DistributedLockManager::new();
        let gid = GlobalTransactionId::new(NodeId(1));

        let lock_key = LockKey::Table("users".to_string());
        assert!(lock_mgr.get_holder(&lock_key).await.is_none());

        lock_mgr.try_lock(&gid, &lock_key).await.unwrap();
        assert_eq!(lock_mgr.get_holder(&lock_key).await, Some(gid));
    }

    #[tokio::test]
    async fn test_row_lock() {
        let lock_mgr = DistributedLockManager::new();
        let gid = GlobalTransactionId::new(NodeId(1));

        let lock_key = LockKey::Row {
            table: "users".to_string(),
            row_key: vec![1, 2, 3],
        };

        let result = lock_mgr.try_lock(&gid, &lock_key).await;
        assert!(result.is_ok());
    }
}
