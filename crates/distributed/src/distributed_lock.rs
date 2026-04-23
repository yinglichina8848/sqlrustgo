//! Distributed Lock Manager
//!
//! Provides distributed locking for coordination across nodes.

use crate::raft::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

pub type TransactionId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    pub key: String,
    pub owner: NodeId,
    pub tx_id: TransactionId,
    pub acquired_at: u64,
    pub expires_at: Option<u64>,
}

impl LockEntry {
    pub fn new(key: String, owner: NodeId, tx_id: TransactionId) -> Self {
        Self {
            key,
            owner,
            tx_id,
            acquired_at: current_timestamp(),
            expires_at: None,
        }
    }

    pub fn with_expiry(mut self, ttl_ms: u64) -> Self {
        self.expires_at = Some(current_timestamp() + ttl_ms);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            return current_timestamp() > expires_at;
        }
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockResult {
    Acquired,
    AlreadyLocked,
    NotOwner,
}

pub struct DistributedLockManager {
    locks: HashMap<String, LockEntry>,
    lock_timeout: Duration,
    pending_requests: HashMap<String, Vec<LockRequest>>,
}

#[derive(Debug, Clone)]
pub struct LockRequest {
    pub node_id: NodeId,
    pub tx_id: TransactionId,
    pub timestamp: u64,
}

impl DistributedLockManager {
    pub fn new() -> Self {
        Self {
            locks: HashMap::new(),
            lock_timeout: Duration::from_secs(30),
            pending_requests: HashMap::new(),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.lock_timeout = timeout;
        self
    }

    pub fn try_lock(&mut self, key: &str, node_id: NodeId, tx_id: TransactionId) -> LockResult {
        if let Some(entry) = self.locks.get(key) {
            if entry.is_expired() {
                self.locks.remove(key);
            } else if entry.owner == node_id && entry.tx_id == tx_id {
                return LockResult::Acquired;
            } else {
                return LockResult::AlreadyLocked;
            }
        }

        let entry = LockEntry::new(key.to_string(), node_id, tx_id);
        self.locks.insert(key.to_string(), entry);
        LockResult::Acquired
    }

    pub fn try_lock_with_ttl(
        &mut self,
        key: &str,
        node_id: NodeId,
        tx_id: TransactionId,
        ttl_ms: u64,
    ) -> LockResult {
        if let Some(entry) = self.locks.get(key) {
            if entry.is_expired() {
                self.locks.remove(key);
            } else if entry.owner == node_id && entry.tx_id == tx_id {
                return LockResult::Acquired;
            } else {
                return LockResult::AlreadyLocked;
            }
        }

        let entry = LockEntry::new(key.to_string(), node_id, tx_id).with_expiry(ttl_ms);
        self.locks.insert(key.to_string(), entry);
        LockResult::Acquired
    }

    pub fn unlock(&mut self, key: &str, node_id: NodeId, tx_id: TransactionId) -> LockResult {
        if let Some(entry) = self.locks.get(key) {
            if entry.owner == node_id && entry.tx_id == tx_id {
                self.locks.remove(key);
                self.process_pending_requests(key);
                LockResult::Acquired
            } else {
                LockResult::NotOwner
            }
        } else {
            LockResult::Acquired
        }
    }

    pub fn is_locked(&self, key: &str) -> bool {
        if let Some(entry) = self.locks.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    pub fn get_lock_owner(&self, key: &str) -> Option<(NodeId, TransactionId)> {
        self.locks.get(key).map(|e| (e.owner, e.tx_id))
    }

    pub fn extend_lock(
        &mut self,
        key: &str,
        node_id: NodeId,
        tx_id: TransactionId,
        ttl_ms: u64,
    ) -> LockResult {
        if let Some(entry) = self.locks.get_mut(key) {
            if entry.owner == node_id && entry.tx_id == tx_id {
                entry.expires_at = Some(current_timestamp() + ttl_ms);
                return LockResult::Acquired;
            } else {
                return LockResult::NotOwner;
            }
        }
        LockResult::AlreadyLocked
    }

    fn process_pending_requests(&mut self, key: &str) {
        if let Some(requests) = self.pending_requests.remove(key) {
            for request in requests {
                if !self.is_locked(key) {
                    let entry = LockEntry::new(key.to_string(), request.node_id, request.tx_id);
                    self.locks.insert(key.to_string(), entry);
                    break;
                }
            }
        }
    }

    pub fn add_pending_request(&mut self, key: &str, node_id: NodeId, tx_id: TransactionId) {
        let request = LockRequest {
            node_id,
            tx_id,
            timestamp: current_timestamp(),
        };

        self.pending_requests
            .entry(key.to_string())
            .or_default()
            .push(request);
    }

    pub fn check_timeouts(&mut self) -> Vec<String> {
        let mut expired_keys = Vec::new();

        for (key, entry) in &self.locks {
            if entry.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        for key in &expired_keys {
            self.locks.remove(key);
            self.process_pending_requests(key);
        }

        expired_keys
    }

    pub fn release_all_for_transaction(&mut self, node_id: NodeId, tx_id: TransactionId) -> usize {
        let keys: Vec<String> = self
            .locks
            .iter()
            .filter(|(_, e)| e.owner == node_id && e.tx_id == tx_id)
            .map(|(k, _)| k.clone())
            .collect();

        for key in &keys {
            self.locks.remove(key);
            self.process_pending_requests(key);
        }

        keys.len()
    }

    pub fn num_locks(&self) -> usize {
        self.locks.len()
    }

    pub fn get_lock_info(&self, key: &str) -> Option<LockInfo> {
        self.locks.get(key).map(|e| LockInfo {
            key: e.key.clone(),
            owner: e.owner,
            tx_id: e.tx_id,
            acquired_at: e.acquired_at,
            expires_at: e.expires_at,
            is_expired: e.is_expired(),
        })
    }

    pub fn list_all_locks(&self) -> Vec<LockInfo> {
        self.locks
            .values()
            .map(|e| LockInfo {
                key: e.key.clone(),
                owner: e.owner,
                tx_id: e.tx_id,
                acquired_at: e.acquired_at,
                expires_at: e.expires_at,
                is_expired: e.is_expired(),
            })
            .collect()
    }
}

impl Default for DistributedLockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct LockInfo {
    pub key: String,
    pub owner: NodeId,
    pub tx_id: TransactionId,
    pub acquired_at: u64,
    pub expires_at: Option<u64>,
    pub is_expired: bool,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_unlock() {
        let mut manager = DistributedLockManager::new();

        assert_eq!(manager.try_lock("key1", 1, 100), LockResult::Acquired);
        assert!(manager.is_locked("key1"));

        assert_eq!(manager.unlock("key1", 1, 100), LockResult::Acquired);
        assert!(!manager.is_locked("key1"));
    }

    #[test]
    fn test_lock_already_locked() {
        let mut manager = DistributedLockManager::new();

        assert_eq!(manager.try_lock("key1", 1, 100), LockResult::Acquired);
        assert_eq!(manager.try_lock("key1", 2, 200), LockResult::AlreadyLocked);
    }

    #[test]
    fn test_unlock_not_owner() {
        let mut manager = DistributedLockManager::new();

        assert_eq!(manager.try_lock("key1", 1, 100), LockResult::Acquired);
        assert_eq!(manager.unlock("key1", 2, 200), LockResult::NotOwner);
    }

    #[test]
    fn test_lock_expiry() {
        let mut manager = DistributedLockManager::new();

        manager.try_lock_with_ttl("key1", 1, 100, 50);
        assert!(manager.is_locked("key1"));

        std::thread::sleep(Duration::from_millis(100));

        assert!(!manager.is_locked("key1"));
    }

    #[test]
    fn test_extend_lock() {
        let mut manager = DistributedLockManager::new();

        manager.try_lock_with_ttl("key1", 1, 100, 50);

        std::thread::sleep(Duration::from_millis(100));

        assert!(!manager.is_locked("key1"));

        manager.try_lock_with_ttl("key1", 1, 100, 100);
        let result = manager.extend_lock("key1", 1, 100, 100);
        assert_eq!(result, LockResult::Acquired);
    }

    #[test]
    fn test_release_all_for_transaction() {
        let mut manager = DistributedLockManager::new();

        manager.try_lock("key1", 1, 100);
        manager.try_lock("key2", 1, 100);
        manager.try_lock("key3", 2, 200);

        assert_eq!(manager.release_all_for_transaction(1, 100), 2);
        assert!(!manager.is_locked("key1"));
        assert!(!manager.is_locked("key2"));
        assert!(manager.is_locked("key3"));
    }

    #[test]
    fn test_lock_entry_new() {
        let entry = LockEntry::new("key1".to_string(), 1, 100);
        assert_eq!(entry.key, "key1");
        assert_eq!(entry.owner, 1);
        assert_eq!(entry.tx_id, 100);
        assert!(entry.expires_at.is_none());
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_lock_entry_with_expiry() {
        let entry = LockEntry::new("key1".to_string(), 1, 100).with_expiry(1000);
        assert!(entry.expires_at.is_some());
    }

    #[test]
    fn test_lock_entry_is_expired() {
        let entry = LockEntry::new("key1".to_string(), 1, 100).with_expiry(0);
        std::thread::sleep(Duration::from_millis(1));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_lock_entry_no_expiry() {
        let entry = LockEntry::new("key1".to_string(), 1, 100);
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_try_lock_same_owner_reentrant() {
        let mut manager = DistributedLockManager::new();

        assert_eq!(manager.try_lock("key1", 1, 100), LockResult::Acquired);
        assert_eq!(manager.try_lock("key1", 1, 100), LockResult::Acquired);
    }

    #[test]
    fn test_lock_manager_unlock() {
        let mut manager = DistributedLockManager::new();
        assert_eq!(manager.try_lock("key1", 1, 100), LockResult::Acquired);
        assert_eq!(manager.unlock("key1", 1, 100), LockResult::Acquired);
        assert_eq!(manager.try_lock("key1", 2, 100), LockResult::Acquired);
    }

    #[test]
    fn test_lock_manager_unlock_wrong_owner() {
        let mut manager = DistributedLockManager::new();
        assert_eq!(manager.try_lock("key1", 1, 100), LockResult::Acquired);
        assert_eq!(manager.unlock("key1", 2, 100), LockResult::NotOwner);
    }

    #[test]
    fn test_lock_manager_is_locked() {
        let mut manager = DistributedLockManager::new();
        assert!(!manager.is_locked("key1"));
        manager.try_lock("key1", 1, 100);
        assert!(manager.is_locked("key1"));
    }

    #[test]
    fn test_lock_manager_get_lock_owner() {
        let mut manager = DistributedLockManager::new();
        assert_eq!(manager.get_lock_owner("key1"), None);
        manager.try_lock("key1", 1, 100);
        assert_eq!(manager.get_lock_owner("key1"), Some((1, 100)));
    }

    #[test]
    fn test_try_lock_with_ttl_basic() {
        let mut manager = DistributedLockManager::new();
        let result = manager.try_lock_with_ttl("key1", 1, 100, 100);
        assert_eq!(result, LockResult::Acquired);
    }

    #[test]
    fn test_try_lock_with_ttl_already_locked() {
        let mut manager = DistributedLockManager::new();
        manager.try_lock_with_ttl("key1", 1, 100, 100);
        let result = manager.try_lock_with_ttl("key1", 2, 200, 100);
        assert_eq!(result, LockResult::AlreadyLocked);
    }

    #[test]
    fn test_extend_lock_not_found() {
        let mut manager = DistributedLockManager::new();
        let result = manager.extend_lock("nonexistent", 1, 100, 100);
        assert_eq!(result, LockResult::AlreadyLocked);
    }

    #[test]
    fn test_extend_lock_not_owner() {
        let mut manager = DistributedLockManager::new();
        manager.try_lock_with_ttl("key1", 1, 100, 100);
        let result = manager.extend_lock("key1", 2, 200, 100);
        assert_eq!(result, LockResult::NotOwner);
    }

    #[test]
    fn test_release_all_for_transaction_none() {
        let mut manager = DistributedLockManager::new();
        let count = manager.release_all_for_transaction(999, 999);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_release_all_for_transaction_partial() {
        let mut manager = DistributedLockManager::new();
        manager.try_lock("key1", 1, 100);
        manager.try_lock("key2", 1, 100);
        manager.try_lock("key3", 2, 200);
        manager.try_lock("key4", 2, 200);
        let count = manager.release_all_for_transaction(1, 100);
        assert_eq!(count, 2);
        assert!(!manager.is_locked("key1"));
        assert!(!manager.is_locked("key2"));
        assert!(manager.is_locked("key3"));
        assert!(manager.is_locked("key4"));
    }

    #[test]
    fn test_lock_manager_with_timeout() {
        let manager = DistributedLockManager::new().with_timeout(Duration::from_secs(60));
        assert!(!manager.is_locked("key1"));
    }

    #[test]
    fn test_lock_entry_debug() {
        let entry = LockEntry::new("test_key".to_string(), 42, 999);
        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("test_key"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_lock_result_debug() {
        assert!(format!("{:?}", LockResult::Acquired).contains("Acquired"));
        assert!(format!("{:?}", LockResult::AlreadyLocked).contains("AlreadyLocked"));
        assert!(format!("{:?}", LockResult::NotOwner).contains("NotOwner"));
    }

    #[test]
    fn test_lock_request_clone() {
        let request = LockRequest {
            node_id: 1,
            tx_id: 100,
            timestamp: 12345,
        };
        let cloned = request.clone();
        assert_eq!(cloned.node_id, request.node_id);
        assert_eq!(cloned.tx_id, request.tx_id);
        assert_eq!(cloned.timestamp, request.timestamp);
    }

    #[test]
    fn test_multiple_locks_different_keys() {
        let mut manager = DistributedLockManager::new();
        assert_eq!(manager.try_lock("key1", 1, 100), LockResult::Acquired);
        assert_eq!(manager.try_lock("key2", 1, 100), LockResult::Acquired);
        assert_eq!(manager.try_lock("key3", 2, 200), LockResult::Acquired);
        assert!(manager.is_locked("key1"));
        assert!(manager.is_locked("key2"));
        assert!(manager.is_locked("key3"));
    }

    #[test]
    fn test_unlock_after_expiry() {
        let mut manager = DistributedLockManager::new();
        manager.try_lock_with_ttl("key1", 1, 100, 10);
        std::thread::sleep(Duration::from_millis(20));
        assert!(!manager.is_locked("key1"));
        assert_eq!(manager.try_lock("key1", 2, 200), LockResult::Acquired);
    }

    #[test]
    fn test_lock_entry_acquired_at() {
        let entry = LockEntry::new("key1".to_string(), 1, 100);
        assert!(entry.acquired_at > 0);
    }
}
