//! Row-level locking implementation
//!
//! This module provides row-level locking for concurrent transaction control,
//! including shared locks (read) and exclusive locks (write).

use crate::deadlock::DeadlockDetector;
use crate::mvcc::TxId;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockMode {
    Shared,
    Exclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockGrantMode {
    Granted,
    Waiting,
}

#[derive(Debug, Clone)]
pub struct LockRequest {
    pub tx_id: TxId,
    pub key: Vec<u8>,
    pub mode: LockMode,
    pub granted: bool,
}

impl LockRequest {
    pub fn new(tx_id: TxId, key: Vec<u8>, mode: LockMode) -> Self {
        Self {
            tx_id,
            key,
            mode,
            granted: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LockInfo {
    pub key: Vec<u8>,
    pub mode: LockMode,
    pub holders: HashSet<TxId>,
    pub waiters: Vec<(TxId, LockMode)>,
}

impl LockInfo {
    pub fn new(key: Vec<u8>, mode: LockMode) -> Self {
        Self {
            key,
            mode,
            holders: HashSet::new(),
            waiters: Vec::new(),
        }
    }

    pub fn can_grant(&self, mode: LockMode, _requester: TxId) -> bool {
        match mode {
            LockMode::Shared => {
                self.holders.is_empty()
                    || (self.mode == LockMode::Shared && self.waiters.is_empty())
            }
            LockMode::Exclusive => self.holders.is_empty() && self.waiters.is_empty(),
        }
    }

    pub fn add_holder(&mut self, tx_id: TxId) {
        self.holders.insert(tx_id);
    }

    pub fn remove_holder(&mut self, tx_id: TxId) {
        self.holders.remove(&tx_id);
    }

    pub fn add_waiter(&mut self, tx_id: TxId, mode: LockMode) {
        if !self.waiters.iter().any(|(t, _)| *t == tx_id) {
            self.waiters.push((tx_id, mode));
        }
    }

    pub fn remove_waiter(&mut self, tx_id: TxId) {
        self.waiters.retain(|(t, _)| *t != tx_id);
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct LockManager {
    locks: HashMap<Vec<u8>, LockInfo>,
    tx_locks: HashMap<TxId, HashSet<Vec<u8>>>,
    deadlock_detector: DeadlockDetector,
}

impl LockManager {
    pub fn new() -> Self {
        Self {
            locks: HashMap::new(),
            tx_locks: HashMap::new(),
            deadlock_detector: DeadlockDetector::new(),
        }
    }

    pub fn detect_deadlock(&mut self, tx_id: TxId) -> Option<Vec<TxId>> {
        self.deadlock_detector.detect_cycle(tx_id)
    }

    pub fn clear_deadlock_edges(&mut self, tx_id: TxId) {
        self.deadlock_detector.remove_edges_for(tx_id);
    }

    pub fn acquire_lock(
        &mut self,
        tx_id: TxId,
        key: Vec<u8>,
        mode: LockMode,
    ) -> Result<LockGrantMode, LockError> {
        let lock = self
            .locks
            .entry(key.clone())
            .or_insert_with(|| LockInfo::new(key.clone(), mode));

        if lock.can_grant(mode, tx_id) {
            lock.add_holder(tx_id);

            let tx_locks = self.tx_locks.entry(tx_id).or_default();
            tx_locks.insert(key);

            Ok(LockGrantMode::Granted)
        } else {
            lock.add_waiter(tx_id, mode);

            if let Some(holders) = self.locks.get(&key).map(|l| &l.holders) {
                for holder in holders {
                    self.deadlock_detector.add_edge(tx_id, *holder);
                }
            }

            if let Some(_cycle) = self.deadlock_detector.detect_cycle(tx_id) {
                return Err(LockError::Deadlock);
            }

            Ok(LockGrantMode::Waiting)
        }
    }

    pub fn upgrade_lock(&mut self, tx_id: TxId, key: Vec<u8>) -> Result<LockGrantMode, LockError> {
        if let Some(lock) = self.locks.get_mut(&key) {
            #[allow(clippy::collapsible_if)]
            if lock.holders.contains(&tx_id) && lock.mode == LockMode::Shared {
                if lock.holders.len() == 1 && lock.waiters.is_empty() {
                    lock.holders.clear();
                    lock.holders.insert(tx_id);
                    lock.mode = LockMode::Exclusive;
                    return Ok(LockGrantMode::Granted);
                }
            }
        }
        Err(LockError::LockUpgradeFailed)
    }

    pub fn release_lock(&mut self, tx_id: TxId, key: &Vec<u8>) -> Result<(), LockError> {
        let should_remove = {
            if let Some(lock) = self.locks.get_mut(key) {
                lock.remove_holder(tx_id);

                if lock.holders.is_empty() && !lock.waiters.is_empty() {
                    if let Some((waiter, requested_mode)) = lock.waiters.first().copied() {
                        lock.add_holder(waiter);
                        lock.remove_waiter(waiter);
                        lock.mode = requested_mode;

                        if let Some(tx_locks) = self.tx_locks.get_mut(&waiter) {
                            tx_locks.insert(key.clone());
                        }
                    }
                }

                lock.holders.is_empty() && lock.waiters.is_empty()
            } else {
                false
            }
        };

        if should_remove {
            self.locks.remove(key);
        }

        if let Some(tx_locks) = self.tx_locks.get_mut(&tx_id) {
            tx_locks.remove(key);
        }

        Ok(())
    }

    pub fn release_all_locks(&mut self, tx_id: TxId) -> Result<Vec<Vec<u8>>, LockError> {
        let mut released = Vec::new();

        if let Some(keys) = self.tx_locks.remove(&tx_id) {
            let keys: Vec<_> = keys.into_iter().collect();
            for key in keys {
                let should_remove = {
                    if let Some(lock) = self.locks.get_mut(&key) {
                        lock.remove_holder(tx_id);

                        if lock.holders.is_empty() && !lock.waiters.is_empty() {
                            if let Some((waiter, requested_mode)) = lock.waiters.first().copied() {
                                lock.add_holder(waiter);
                                lock.remove_waiter(waiter);
                                lock.mode = requested_mode;

                                if let Some(tx_locks) = self.tx_locks.get_mut(&waiter) {
                                    tx_locks.insert(key.clone());
                                }
                            }
                        }

                        lock.holders.is_empty() && lock.waiters.is_empty()
                    } else {
                        false
                    }
                };

                if should_remove {
                    self.locks.remove(&key);
                }
                released.push(key);
            }
        }

        Ok(released)
    }

    pub fn is_locked(&self, key: &Vec<u8>) -> bool {
        self.locks.contains_key(key)
    }

    pub fn is_locked_by_tx(&self, key: &Vec<u8>, tx_id: TxId) -> bool {
        self.locks
            .get(key)
            .map(|lock| lock.holders.contains(&tx_id))
            .unwrap_or(false)
    }

    pub fn has_exclusive_lock(&self, key: &Vec<u8>, tx_id: TxId) -> bool {
        self.locks
            .get(key)
            .map(|lock| lock.mode == LockMode::Exclusive && lock.holders.contains(&tx_id))
            .unwrap_or(false)
    }

    pub fn get_lock_count(&self) -> usize {
        self.locks.len()
    }

    pub fn get_tx_lock_count(&self, tx_id: TxId) -> usize {
        self.tx_locks.get(&tx_id).map(|l| l.len()).unwrap_or(0)
    }
}

impl Default for LockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum LockError {
    Deadlock,
    LockUpgradeFailed,
    LockNotHeld,
    LockTimeout,
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockError::Deadlock => write!(f, "deadlock detected"),
            LockError::LockUpgradeFailed => write!(f, "lock upgrade failed"),
            LockError::LockNotHeld => write!(f, "lock not held by transaction"),
            LockError::LockTimeout => write!(f, "lock acquisition timeout"),
        }
    }
}

impl std::error::Error for LockError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_lock() {
        let mut manager = LockManager::new();
        let tx_id = TxId::new(1);
        let key = vec![1, 2, 3];

        let result = manager.acquire_lock(tx_id, key.clone(), LockMode::Shared);
        assert!(matches!(result, Ok(LockGrantMode::Granted)));
        assert!(manager.is_locked(&key));
    }

    #[test]
    fn test_exclusive_lock() {
        let mut manager = LockManager::new();
        let tx_id = TxId::new(1);
        let key = vec![1, 2, 3];

        let result = manager.acquire_lock(tx_id, key.clone(), LockMode::Exclusive);
        assert!(matches!(result, Ok(LockGrantMode::Granted)));
        assert!(manager.has_exclusive_lock(&key, tx_id));
    }

    #[test]
    fn test_shared_lock_conflict() {
        let mut manager = LockManager::new();
        let key = vec![1, 2, 3];

        manager
            .acquire_lock(TxId::new(1), key.clone(), LockMode::Shared)
            .unwrap();
        let result = manager.acquire_lock(TxId::new(2), key.clone(), LockMode::Exclusive);

        assert!(matches!(result, Ok(LockGrantMode::Waiting)));
    }

    #[test]
    fn test_lock_upgrade() {
        let mut manager = LockManager::new();
        let tx_id = TxId::new(1);
        let key = vec![1, 2, 3];

        manager
            .acquire_lock(tx_id, key.clone(), LockMode::Shared)
            .unwrap();
        let result = manager.upgrade_lock(tx_id, key.clone());

        assert!(matches!(result, Ok(LockGrantMode::Granted)));
        assert!(manager.has_exclusive_lock(&key, tx_id));
    }

    #[test]
    fn test_lock_release() {
        let mut manager = LockManager::new();
        let tx_id = TxId::new(1);
        let key = vec![1, 2, 3];

        manager
            .acquire_lock(tx_id, key.clone(), LockMode::Shared)
            .unwrap();
        manager.release_lock(tx_id, &key).unwrap();

        assert!(!manager.is_locked(&key));
    }

    #[test]
    fn test_lock_release_grants_to_waiter() {
        let mut manager = LockManager::new();
        let key = vec![1, 2, 3];

        manager
            .acquire_lock(TxId::new(1), key.clone(), LockMode::Shared)
            .unwrap();
        manager
            .acquire_lock(TxId::new(2), key.clone(), LockMode::Exclusive)
            .unwrap();

        manager.release_lock(TxId::new(1), &key).unwrap();

        assert!(manager.has_exclusive_lock(&key, TxId::new(2)));
    }

    #[test]
    fn test_release_all_locks() {
        let mut manager = LockManager::new();
        let tx_id = TxId::new(1);
        let key1 = vec![1, 2, 3];
        let key2 = vec![4, 5, 6];

        manager
            .acquire_lock(tx_id, key1.clone(), LockMode::Shared)
            .unwrap();
        manager
            .acquire_lock(tx_id, key2.clone(), LockMode::Shared)
            .unwrap();

        let released = manager.release_all_locks(tx_id).unwrap();

        assert_eq!(released.len(), 2);
        assert!(!manager.is_locked(&key1));
        assert!(!manager.is_locked(&key2));
    }

    #[test]
    fn test_multiple_shared_locks() {
        let mut manager = LockManager::new();
        let key = vec![1, 2, 3];

        manager
            .acquire_lock(TxId::new(1), key.clone(), LockMode::Shared)
            .unwrap();
        let result = manager.acquire_lock(TxId::new(2), key.clone(), LockMode::Shared);

        assert!(matches!(result, Ok(LockGrantMode::Granted)));
    }

    #[test]
    fn test_lock_manager_with_deadlock_detector() {
        let mut manager = LockManager::new();

        let key1 = vec![1];
        let key2 = vec![2];

        manager
            .acquire_lock(TxId::new(1), key1.clone(), LockMode::Exclusive)
            .unwrap();

        manager
            .acquire_lock(TxId::new(1), key2.clone(), LockMode::Shared)
            .unwrap();

        let result = manager.acquire_lock(TxId::new(2), key2.clone(), LockMode::Exclusive);
        assert!(matches!(result, Ok(LockGrantMode::Waiting)));

        let _ = manager.detect_deadlock(TxId::new(2));
    }
}
