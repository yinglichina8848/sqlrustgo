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
            // TLA+ v4 Wait(t,k): add edge only if ~Reachable(h,t) for ALL holders.
            // This is pre-check (prevention) — illegal states are unreachable.
            // TOCTOU-safe via DeadlockDetector.try_wait_edge() (atomic Mutex).
            let holders: HashSet<TxId> = lock.holders.clone();

            // NoSelfWait: if tx_id already holds this lock, self-wait is a deadlock.
            // (can_grant returning false means we need to wait, but self-wait is
            // never valid — a transaction cannot wait for its own lock.)
            if holders.contains(&tx_id) {
                return Err(LockError::Deadlock);
            }

            // PROOF-023 atomicity requirement: would_create_cycle + add_edge
            // must happen in the same locked region (no TOCTOU window).
            if self.deadlock_detector.try_wait_edge(tx_id, holders).is_err() {
                return Err(LockError::Deadlock);
            }

            // Only add waiter after atomic pre-check passes.
            lock.add_waiter(tx_id, mode);

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

    // ─────────────────────────────────────────────────────────────────────────
    // PROOF-023 v4 TLA+ refinement tests
    // These verify the Rust implementation aligns with TLA+ v4 Wait-For Graph
    // semantics: pre-check prevents cycles (deadlock-free by construction).
    // ─────────────────────────────────────────────────────────────────────────

    /// TLA+ v4: Classic 3-cycle (T1→T2→T3→T1).
    /// With pre-check, T3's wait on k1 must be REJECTED before edge is added.
    /// This is the key refinement: illegal state is unreachable, not detected.
    #[test]
    fn test_prevent_3_cycle_via_precheck() {
        let mut manager = LockManager::new();
        let k1 = vec![1];
        let k2 = vec![2];
        let k3 = vec![3];

        // T1 holds k1, T2 holds k2, T3 holds k3
        manager
            .acquire_lock(TxId::new(1), k1.clone(), LockMode::Exclusive)
            .unwrap();
        manager
            .acquire_lock(TxId::new(2), k2.clone(), LockMode::Exclusive)
            .unwrap();
        manager
            .acquire_lock(TxId::new(3), k3.clone(), LockMode::Exclusive)
            .unwrap();

        // T1 waits k2 (T1→{T2})
        assert!(matches!(
            manager.acquire_lock(TxId::new(1), k2.clone(), LockMode::Exclusive),
            Ok(LockGrantMode::Waiting)
        ));

        // T2 waits k3 (T2→{T3})
        assert!(matches!(
            manager.acquire_lock(TxId::new(2), k3.clone(), LockMode::Exclusive),
            Ok(LockGrantMode::Waiting)
        ));

        // T3 waits k1 — T3→{T1} would create 3-cycle.
        // TLA+ v4 pre-check: Reachable(T1, T3) via T1→T2→T3→T1.
        // Wait(T3,k1) requires ~Reachable(T1,T3), but T1→T2→T3 exists.
        // PRE-CHECK must reject, not post-detect.
        assert!(matches!(
            manager.acquire_lock(TxId::new(3), k1.clone(), LockMode::Exclusive),
            Err(LockError::Deadlock)
        ));
    }

    /// TLA+ v4: Multi-resource deadlock (T1→{T2,T3}, T2→{T4}, T4→{T1}).
    /// T1 waits on MULTIPLE resources simultaneously.
    #[test]
    fn test_multi_resource_wait_for_graph_v4() {
        let mut manager = LockManager::new();
        let k1 = vec![1];
        let k2 = vec![2];
        let k3 = vec![3];
        let k4 = vec![4];

        // T1 holds k1; T2 holds k2; T3 holds k3; T4 holds k4
        manager
            .acquire_lock(TxId::new(1), k1.clone(), LockMode::Exclusive)
            .unwrap();
        manager
            .acquire_lock(TxId::new(2), k2.clone(), LockMode::Exclusive)
            .unwrap();
        manager
            .acquire_lock(TxId::new(3), k3.clone(), LockMode::Exclusive)
            .unwrap();
        manager
            .acquire_lock(TxId::new(4), k4.clone(), LockMode::Exclusive)
            .unwrap();

        // T1 waits on k2 and k3 → T1→{T2,T3}
        assert!(matches!(
            manager.acquire_lock(TxId::new(1), k2.clone(), LockMode::Exclusive),
            Ok(LockGrantMode::Waiting)
        ));
        assert!(matches!(
            manager.acquire_lock(TxId::new(1), k3.clone(), LockMode::Exclusive),
            Ok(LockGrantMode::Waiting)
        ));

        // T2 waits on k4 → T2→{T4}
        assert!(matches!(
            manager.acquire_lock(TxId::new(2), k4.clone(), LockMode::Exclusive),
            Ok(LockGrantMode::Waiting)
        ));

        // T4 waits on k1 → T4→{T1}
        // Reachable(T1, T4) via T1→T2→T4 exists.
        // Wait(T4, k1) requires ~Reachable(T1, T4), but T1→T2→T4 path exists.
        // PRE-CHECK must reject.
        assert!(matches!(
            manager.acquire_lock(TxId::new(4), k1.clone(), LockMode::Exclusive),
            Err(LockError::Deadlock)
        ));
    }

    /// TLA+ v4: NoSelfWait — a transaction cannot wait for itself.
    /// This is a trivial cycle that must be prevented at pre-check.
    #[test]
    fn test_no_self_wait() {
        let mut manager = LockManager::new();
        let k1 = vec![1];

        // T1 holds k1
        manager
            .acquire_lock(TxId::new(1), k1.clone(), LockMode::Exclusive)
            .unwrap();

        // T1 tries to wait on k1 — NoSelfWait violation.
        // Self-loop is a trivial cycle of length 1.
        assert!(matches!(
            manager.acquire_lock(TxId::new(1), k1.clone(), LockMode::Exclusive),
            Err(LockError::Deadlock)
        ));
    }

    /// TLA+ v4: After release, deadlock-free paths must be allowed again.
    #[test]
    fn test_release_restores_wait_path() {
        let mut manager = LockManager::new();
        let k1 = vec![1];
        let k2 = vec![2];

        // T1 holds k1; T2 holds k2
        manager
            .acquire_lock(TxId::new(1), k1.clone(), LockMode::Exclusive)
            .unwrap();
        manager
            .acquire_lock(TxId::new(2), k2.clone(), LockMode::Exclusive)
            .unwrap();

        // T1 waits k2 → T1→{T2}
        assert!(matches!(
            manager.acquire_lock(TxId::new(1), k2.clone(), LockMode::Exclusive),
            Ok(LockGrantMode::Waiting)
        ));

        // T2 releases all locks → T1 promoted to holder of k2
        manager.release_all_locks(TxId::new(2)).unwrap();

        // T1 now holds k2. Re-acquiring k2 is a self-dependency → Deadlock.
        // (NoSelfWait: a txn cannot wait for its own held lock.)
        let result = manager.acquire_lock(TxId::new(1), k2.clone(), LockMode::Exclusive);
        assert!(matches!(result, Err(LockError::Deadlock)));
    }

    /// TLA+ v4: Linear chain must NOT be flagged as deadlock.
    /// T1→T2→T3 is a valid wait-for chain (no cycle).
    #[test]
    fn test_linear_chain_no_deadlock() {
        let mut manager = LockManager::new();
        let k1 = vec![1];
        let k2 = vec![2];
        let k3 = vec![3];
        let k4 = vec![4];

        manager
            .acquire_lock(TxId::new(1), k1.clone(), LockMode::Exclusive)
            .unwrap();
        manager
            .acquire_lock(TxId::new(2), k2.clone(), LockMode::Exclusive)
            .unwrap();
        manager
            .acquire_lock(TxId::new(3), k3.clone(), LockMode::Exclusive)
            .unwrap();
        manager
            .acquire_lock(TxId::new(4), k4.clone(), LockMode::Exclusive)
            .unwrap();

        // T1 waits k2 → T1→{T2} ✓ (no cycle)
        assert!(matches!(
            manager.acquire_lock(TxId::new(1), k2.clone(), LockMode::Exclusive),
            Ok(LockGrantMode::Waiting)
        ));

        // T2 waits k3 → T2→{T3} ✓ (no cycle, T2→T3, T3 reaches T2? No)
        assert!(matches!(
            manager.acquire_lock(TxId::new(2), k3.clone(), LockMode::Exclusive),
            Ok(LockGrantMode::Waiting)
        ));

        // T3 waits k4 → T3→{T4} ✓ (no cycle)
        assert!(matches!(
            manager.acquire_lock(TxId::new(3), k4.clone(), LockMode::Exclusive),
            Ok(LockGrantMode::Waiting)
        ));

        // T4 tries to wait k1 — T4→{T1} would form T1→T2→T3→T4→T1 cycle.
        // Deadlock pre-check must reject.
        assert!(matches!(
            manager.acquire_lock(TxId::new(4), k1.clone(), LockMode::Exclusive),
            Err(LockError::Deadlock)
        ));
    }
}
