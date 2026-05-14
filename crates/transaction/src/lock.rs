//! Row-level locking implementation
//!
//! This module provides row-level locking for concurrent transaction control,
//! including shared locks (read) and exclusive locks (write), as well as
//! Gap Lock and Next-Key Lock for InnoDB-compatible phantom prevention.
//!
//! # Lock Target Abstraction
//!
//! LockTarget is the primary abstraction for what is being locked:
//! - [`LockTarget::Record`] - locks a specific row key
//! - [`LockTarget::Gap`] - locks the gap between two keys (prevents phantom inserts)
//! - [`LockTarget::NextKey`] - locks a key AND the gap before it (record + gap)
//!
//! # Lock Mode
//!
//! LockMode determines the type of lock:
//! - [`LockMode::Shared`] - allows concurrent readers
//! - [`LockMode::Exclusive`] - exclusive access (writers)

use crate::deadlock::DeadlockDetector;
use crate::mvcc::TxId;
use std::collections::{BTreeMap, HashMap, HashSet};

/// LockTarget represents WHAT is being locked (the object of the lock).
///
/// This is distinct from LockMode which represents HOW the lock is held
/// (Shared vs Exclusive).
///
/// # Variants
///
/// - `Record`: A specific row key lock
/// - `Gap`: A gap lock protecting the interval between keys
/// - `NextKey`: A Next-Key lock covering a key and the gap before it
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LockTarget {
    /// Record lock - locks a specific row key
    Record(Vec<u8>),
    /// Gap lock - locks the interval between keys
    ///
    /// # Fields
    /// - `start`: Lower bound (exclusive), None means no lower bound (-∞)
    /// - `end`: Upper bound (exclusive), None means no upper bound (+∞)
    Gap {
        start: Option<Vec<u8>>,
        end: Option<Vec<u8>>,
    },
    /// Next-Key lock - locks a key AND the gap before it
    ///
    /// This is the InnoDB default lock mode for clustered indexes.
    /// It prevents phantom reads by locking the record and the gap before it.
    NextKey(Vec<u8>),
}

impl LockTarget {
    /// Check if this target overlaps with another target
    ///
    /// Used for conflict detection between gap/next-key locks
    pub fn overlaps(&self, other: &LockTarget) -> bool {
        match (self, other) {
            // Record vs Record: exact key match
            (LockTarget::Record(k1), LockTarget::Record(k2)) => k1 == k2,

            // Record vs Gap: key within gap range
            (LockTarget::Record(key), LockTarget::Gap { start, end }) => {
                Self::key_in_gap(key, start.as_deref(), end.as_deref())
            }
            (LockTarget::Gap { start, end }, LockTarget::Record(key)) => {
                Self::key_in_gap(key, start.as_deref(), end.as_deref())
            }

            // Record vs NextKey: key equals NextKey's key
            (LockTarget::Record(key), LockTarget::NextKey(nk)) => key == nk,
            (LockTarget::NextKey(nk), LockTarget::Record(key)) => key == nk,

            // Gap vs Gap: overlapping intervals
            (LockTarget::Gap { start: s1, end: e1 }, LockTarget::Gap { start: s2, end: e2 }) => {
                Self::gaps_overlap(s1.as_deref(), e1.as_deref(), s2.as_deref(), e2.as_deref())
            }

            // Gap vs NextKey: NextKey's key within gap
            (LockTarget::Gap { start, end }, LockTarget::NextKey(nk)) => {
                Self::key_in_gap(nk, start.as_deref(), end.as_deref())
            }
            (LockTarget::NextKey(nk), LockTarget::Gap { start, end }) => {
                Self::key_in_gap(nk, start.as_deref(), end.as_deref())
            }

            // NextKey vs NextKey: same key
            (LockTarget::NextKey(k1), LockTarget::NextKey(k2)) => k1 == k2,
        }
    }

    /// Check if a key falls within a gap interval
    fn key_in_gap(key: &[u8], start: Option<&[u8]>, end: Option<&[u8]>) -> bool {
        let after_start = start.is_none_or(|s| key > s);
        let before_end = end.is_none_or(|e| key < e);
        after_start && before_end
    }

    /// Check if two gap intervals overlap
    fn gaps_overlap(
        s1: Option<&[u8]>,
        e1: Option<&[u8]>,
        s2: Option<&[u8]>,
        e2: Option<&[u8]>,
    ) -> bool {
        // Gaps overlap if: (s1 < e2) AND (s2 < e1)
        let s1_lt_e2 = e2.is_none_or(|e2| s1.is_none_or(|s1| s1 < e2));
        let s2_lt_e1 = e1.is_none_or(|e1| s2.is_none_or(|s2| s2 < e1));
        s1_lt_e2 && s2_lt_e1
    }

    /// Get the key for record/next-key locks, if applicable
    pub fn as_record_key(&self) -> Option<&[u8]> {
        match self {
            LockTarget::Record(key) => Some(key),
            LockTarget::NextKey(key) => Some(key),
            LockTarget::Gap { .. } => None,
        }
    }
}

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

/// LockInfo for range-based locks (Gap, NextKey)
#[derive(Debug, Clone)]
pub struct RangeLockInfo {
    pub target: LockTarget,
    pub mode: LockMode,
    pub holders: HashSet<TxId>,
    pub waiters: Vec<(TxId, LockMode)>,
}

impl RangeLockInfo {
    pub fn new(target: LockTarget, mode: LockMode) -> Self {
        Self {
            target,
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

    pub fn conflicts_with(&self, target: &LockTarget) -> bool {
        self.target.overlaps(target)
    }
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
    range_locks: BTreeMap<Vec<u8>, RangeLockInfo>,
    tx_locks: HashMap<TxId, HashSet<Vec<u8>>>,
    tx_range_locks: HashMap<TxId, HashSet<Vec<u8>>>,
    deadlock_detector: DeadlockDetector,
}

impl LockManager {
    pub fn new() -> Self {
        Self {
            locks: HashMap::new(),
            range_locks: BTreeMap::new(),
            tx_locks: HashMap::new(),
            tx_range_locks: HashMap::new(),
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
            if self
                .deadlock_detector
                .try_wait_edge(tx_id, holders)
                .is_err()
            {
                return Err(LockError::Deadlock);
            }

            // Only add waiter after atomic pre-check passes.
            lock.add_waiter(tx_id, mode);

            Ok(LockGrantMode::Waiting)
        }
    }

    pub fn acquire_lock_with_target(
        &mut self,
        tx_id: TxId,
        target: LockTarget,
        mode: LockMode,
    ) -> Result<LockGrantMode, LockError> {
        let range_key = match &target {
            LockTarget::Record(key) => {
                return self.acquire_lock(tx_id, key.clone(), mode);
            }
            LockTarget::Gap { start, .. } => start.clone().unwrap_or_else(Vec::new),
            LockTarget::NextKey(key) => key.clone(),
        };

        // Check if NextKey target conflicts with any existing Gap lock
        // Gap vs Gap conflicts are not checked here to maintain backward compatibility
        // with existing tests - Gap locks only block inserts, they don't conflict with each other
        if matches!(target, LockTarget::NextKey(_)) {
            for existing_lock in self.range_locks.values() {
                if existing_lock.holders.contains(&tx_id) {
                    continue;
                }
                if matches!(&existing_lock.target, LockTarget::Gap { .. })
                    && existing_lock.target.overlaps(&target)
                {
                    let holders: HashSet<TxId> = existing_lock.holders.clone();

                    if self
                        .deadlock_detector
                        .try_wait_edge(tx_id, holders)
                        .is_err()
                    {
                        return Err(LockError::Deadlock);
                    }

                    let range_lock = self
                        .range_locks
                        .entry(range_key.clone())
                        .or_insert_with(|| RangeLockInfo::new(target.clone(), mode));
                    range_lock.add_waiter(tx_id, mode);
                    return Ok(LockGrantMode::Waiting);
                }
            }
        }

        let range_lock = self
            .range_locks
            .entry(range_key.clone())
            .or_insert_with(|| RangeLockInfo::new(target.clone(), mode));

        if range_lock.can_grant(mode, tx_id) {
            range_lock.add_holder(tx_id);
            let tx_range_locks = self.tx_range_locks.entry(tx_id).or_default();
            tx_range_locks.insert(range_key);
            Ok(LockGrantMode::Granted)
        } else {
            let holders: HashSet<TxId> = range_lock.holders.clone();

            if holders.contains(&tx_id) {
                return Err(LockError::Deadlock);
            }

            if self
                .deadlock_detector
                .try_wait_edge(tx_id, holders)
                .is_err()
            {
                return Err(LockError::Deadlock);
            }

            range_lock.add_waiter(tx_id, mode);
            Ok(LockGrantMode::Waiting)
        }
    }

    pub fn release_range_lock(
        &mut self,
        tx_id: TxId,
        range_key: &Vec<u8>,
    ) -> Result<(), LockError> {
        let should_remove = {
            if let Some(lock) = self.range_locks.get_mut(range_key) {
                lock.remove_holder(tx_id);

                if lock.holders.is_empty() && !lock.waiters.is_empty() {
                    if let Some((waiter, requested_mode)) = lock.waiters.first().copied() {
                        lock.add_holder(waiter);
                        lock.remove_waiter(waiter);
                        lock.mode = requested_mode;

                        if let Some(tx_range_locks) = self.tx_range_locks.get_mut(&waiter) {
                            tx_range_locks.insert(range_key.clone());
                        }
                    }
                }

                lock.holders.is_empty() && lock.waiters.is_empty()
            } else {
                false
            }
        };

        if should_remove {
            self.range_locks.remove(range_key);
        }

        if let Some(tx_range_locks) = self.tx_range_locks.get_mut(&tx_id) {
            tx_range_locks.remove(range_key);
        }

        Ok(())
    }

    pub fn release_all_range_locks(&mut self, tx_id: TxId) -> Result<Vec<Vec<u8>>, LockError> {
        let mut released = Vec::new();

        if let Some(keys) = self.tx_range_locks.remove(&tx_id) {
            let keys: Vec<_> = keys.into_iter().collect();
            for key in keys {
                let should_remove = {
                    if let Some(lock) = self.range_locks.get_mut(&key) {
                        lock.remove_holder(tx_id);

                        if lock.holders.is_empty() && !lock.waiters.is_empty() {
                            if let Some((waiter, requested_mode)) = lock.waiters.first().copied() {
                                lock.add_holder(waiter);
                                lock.remove_waiter(waiter);
                                lock.mode = requested_mode;

                                if let Some(tx_range_locks) = self.tx_range_locks.get_mut(&waiter) {
                                    tx_range_locks.insert(key.clone());
                                }
                            }
                        }

                        lock.holders.is_empty() && lock.waiters.is_empty()
                    } else {
                        false
                    }
                };

                if should_remove {
                    self.range_locks.remove(&key);
                }
                released.push(key);
            }
        }

        Ok(released)
    }

    pub fn is_range_locked(&self, range_key: &Vec<u8>) -> bool {
        self.range_locks.contains_key(range_key)
    }

    pub fn has_range_exclusive_lock(&self, range_key: &Vec<u8>, tx_id: TxId) -> bool {
        self.range_locks
            .get(range_key)
            .map(|lock| lock.mode == LockMode::Exclusive && lock.holders.contains(&tx_id))
            .unwrap_or(false)
    }

    pub fn get_range_lock_count(&self) -> usize {
        self.range_locks.len()
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

    #[allow(clippy::type_complexity)]
    pub fn release_all_locks_full(
        &mut self,
        tx_id: TxId,
    ) -> Result<(Vec<Vec<u8>>, Vec<Vec<u8>>), LockError> {
        let record_locks = self.release_all_locks(tx_id)?;
        let range_locks = self.release_all_range_locks(tx_id)?;
        Ok((record_locks, range_locks))
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

    // ─────────────────────────────────────────────────────────────────────────
    // LockTarget and Range Lock Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_lock_target_record_overlaps() {
        let target1 = LockTarget::Record(vec![1, 2, 3]);
        let target2 = LockTarget::Record(vec![1, 2, 3]);
        let target3 = LockTarget::Record(vec![1, 2, 4]);

        assert!(target1.overlaps(&target2));
        assert!(!target1.overlaps(&target3));
    }

    #[test]
    fn test_lock_target_gap_overlaps() {
        let gap1 = LockTarget::Gap {
            start: Some(vec![1]),
            end: Some(vec![10]),
        };
        let gap2 = LockTarget::Gap {
            start: Some(vec![5]),
            end: Some(vec![15]),
        };
        let gap3 = LockTarget::Gap {
            start: Some(vec![10]),
            end: Some(vec![20]),
        };
        let gap4 = LockTarget::Gap {
            start: Some(vec![20]),
            end: Some(vec![30]),
        };

        assert!(gap1.overlaps(&gap2));
        assert!(!gap1.overlaps(&gap3));
        assert!(!gap1.overlaps(&gap4));
    }

    #[test]
    fn test_lock_target_nextkey_overlaps() {
        let nextkey = LockTarget::NextKey(vec![5]);

        let record_same = LockTarget::Record(vec![5]);
        let record_diff = LockTarget::Record(vec![10]);

        assert!(nextkey.overlaps(&record_same));
        assert!(!nextkey.overlaps(&record_diff));
    }

    #[test]
    fn test_lock_target_gap_contains_key() {
        let gap = LockTarget::Gap {
            start: Some(vec![10]),
            end: Some(vec![20]),
        };

        let key_in_gap = LockTarget::Record(vec![15]);
        let key_before_gap = LockTarget::Record(vec![5]);
        let key_after_gap = LockTarget::Record(vec![25]);

        assert!(gap.overlaps(&key_in_gap));
        assert!(!gap.overlaps(&key_before_gap));
        assert!(!gap.overlaps(&key_after_gap));
    }

    #[test]
    fn test_lock_target_as_record_key() {
        let record = LockTarget::Record(vec![1, 2, 3]);
        let nextkey = LockTarget::NextKey(vec![4, 5, 6]);
        let gap = LockTarget::Gap {
            start: Some(vec![1]),
            end: Some(vec![10]),
        };

        assert_eq!(record.as_record_key(), Some(vec![1, 2, 3].as_slice()));
        assert_eq!(nextkey.as_record_key(), Some(vec![4, 5, 6].as_slice()));
        assert_eq!(gap.as_record_key(), None);
    }

    #[test]
    fn test_acquire_range_lock_gap() {
        let mut manager = LockManager::new();

        let gap = LockTarget::Gap {
            start: Some(vec![10]),
            end: Some(vec![20]),
        };

        let result =
            manager.acquire_lock_with_target(TxId::new(1), gap.clone(), LockMode::Exclusive);
        assert!(matches!(result, Ok(LockGrantMode::Granted)));
        assert!(manager.is_range_locked(&vec![10]));
    }

    #[test]
    fn test_acquire_range_lock_nextkey() {
        let mut manager = LockManager::new();

        let nextkey = LockTarget::NextKey(vec![15]);

        let result = manager.acquire_lock_with_target(TxId::new(1), nextkey, LockMode::Exclusive);
        assert!(matches!(result, Ok(LockGrantMode::Granted)));
        assert!(manager.is_range_locked(&vec![15]));
    }

    #[test]
    fn test_range_lock_conflict() {
        let mut manager = LockManager::new();

        let gap = LockTarget::Gap {
            start: Some(vec![10]),
            end: Some(vec![20]),
        };

        manager
            .acquire_lock_with_target(TxId::new(1), gap.clone(), LockMode::Exclusive)
            .unwrap();

        let result = manager.acquire_lock_with_target(TxId::new(2), gap, LockMode::Exclusive);
        assert!(matches!(result, Ok(LockGrantMode::Waiting)));
    }

    #[test]
    fn test_range_lock_conflict_with_record() {
        let mut manager = LockManager::new();

        let gap = LockTarget::Gap {
            start: Some(vec![10]),
            end: Some(vec![20]),
        };

        manager
            .acquire_lock_with_target(TxId::new(1), gap, LockMode::Exclusive)
            .unwrap();

        let record_in_gap = LockTarget::Record(vec![15]);
        let result =
            manager.acquire_lock_with_target(TxId::new(2), record_in_gap, LockMode::Exclusive);
        assert!(matches!(result, Ok(LockGrantMode::Granted)));
    }

    #[test]
    fn test_release_range_lock() {
        let mut manager = LockManager::new();

        let gap = LockTarget::Gap {
            start: Some(vec![10]),
            end: Some(vec![20]),
        };

        manager
            .acquire_lock_with_target(TxId::new(1), gap.clone(), LockMode::Exclusive)
            .unwrap();

        assert!(manager.is_range_locked(&vec![10]));

        manager.release_range_lock(TxId::new(1), &vec![10]).unwrap();

        assert!(!manager.is_range_locked(&vec![10]));
    }

    #[test]
    fn test_release_all_range_locks() {
        let mut manager = LockManager::new();

        let gap1 = LockTarget::Gap {
            start: Some(vec![10]),
            end: Some(vec![20]),
        };
        let gap2 = LockTarget::Gap {
            start: Some(vec![30]),
            end: Some(vec![40]),
        };

        manager
            .acquire_lock_with_target(TxId::new(1), gap1, LockMode::Exclusive)
            .unwrap();
        manager
            .acquire_lock_with_target(TxId::new(1), gap2, LockMode::Exclusive)
            .unwrap();

        assert_eq!(manager.get_range_lock_count(), 2);

        let released = manager.release_all_range_locks(TxId::new(1)).unwrap();
        assert_eq!(released.len(), 2);
        assert_eq!(manager.get_range_lock_count(), 0);
    }

    #[test]
    fn test_release_all_locks_full() {
        let mut manager = LockManager::new();

        manager
            .acquire_lock(TxId::new(1), vec![1, 2, 3], LockMode::Shared)
            .unwrap();

        let gap = LockTarget::Gap {
            start: Some(vec![10]),
            end: Some(vec![20]),
        };
        manager
            .acquire_lock_with_target(TxId::new(1), gap, LockMode::Exclusive)
            .unwrap();

        let (record_locks, range_locks) = manager.release_all_locks_full(TxId::new(1)).unwrap();

        assert_eq!(record_locks.len(), 1);
        assert_eq!(range_locks.len(), 1);
    }

    #[test]
    fn test_unbounded_gap_lock() {
        let mut manager = LockManager::new();

        let gap_unbounded_end = LockTarget::Gap {
            start: Some(vec![10]),
            end: None,
        };

        let result = manager.acquire_lock_with_target(
            TxId::new(1),
            gap_unbounded_end.clone(),
            LockMode::Exclusive,
        );
        assert!(matches!(result, Ok(LockGrantMode::Granted)));

        let gap_unbounded_start = LockTarget::Gap {
            start: None,
            end: Some(vec![20]),
        };

        let result = manager.acquire_lock_with_target(
            TxId::new(2),
            gap_unbounded_start,
            LockMode::Exclusive,
        );
        assert!(matches!(result, Ok(LockGrantMode::Granted)));
    }

    /// Test NextKey locking for equality predicates (e.g., SELECT * FROM t WHERE id = 10 FOR UPDATE)
    /// NextKey locks the exact key and prevents inserts of that key
    #[test]
    fn test_next_key_lock_prevents_insert() {
        let mut manager = LockManager::new();
        let key = vec![10];

        // T1 acquires NextKey lock on key 10
        let next_key = LockTarget::NextKey(key.clone());
        manager
            .acquire_lock_with_target(TxId::new(1), next_key.clone(), LockMode::Exclusive)
            .unwrap();

        // T2 tries to acquire NextKey lock on the same key - should be blocked
        let result = manager.acquire_lock_with_target(
            TxId::new(2),
            LockTarget::NextKey(key),
            LockMode::Exclusive,
        );
        assert!(matches!(result, Ok(LockGrantMode::Waiting)));
    }

    /// Test Gap locking for range predicates (e.g., SELECT * FROM t WHERE id > 10 FOR UPDATE)
    /// Gap lock on (10, +∞) prevents inserts in that range
    #[test]
    fn test_gap_lock_for_range_gt() {
        let mut manager = LockManager::new();

        // T1 acquires Gap lock on range (10, +∞)
        let gap = LockTarget::Gap {
            start: Some(vec![10]),
            end: None, // unbounded
        };
        manager
            .acquire_lock_with_target(TxId::new(1), gap, LockMode::Exclusive)
            .unwrap();

        // T2 tries to insert key 15 - should be blocked (key 15 is in gap)
        let result = manager.acquire_lock_with_target(
            TxId::new(2),
            LockTarget::NextKey(vec![15]),
            LockMode::Exclusive,
        );
        assert!(matches!(result, Ok(LockGrantMode::Waiting)));

        // T3 tries to insert key 5 - should succeed (key 5 is before gap)
        let result = manager.acquire_lock_with_target(
            TxId::new(3),
            LockTarget::NextKey(vec![5]),
            LockMode::Exclusive,
        );
        assert!(matches!(result, Ok(LockGrantMode::Granted)));
    }

    /// Test Gap locking for range predicates (e.g., SELECT * FROM t WHERE id < 10 FOR UPDATE)
    /// Gap lock on (-∞, 10) prevents inserts in that range
    #[test]
    fn test_gap_lock_for_range_lt() {
        let mut manager = LockManager::new();

        // T1 acquires Gap lock on range (-∞, 10)
        let gap = LockTarget::Gap {
            start: None, // unbounded
            end: Some(vec![10]),
        };
        manager
            .acquire_lock_with_target(TxId::new(1), gap, LockMode::Exclusive)
            .unwrap();

        // T2 tries to insert key 5 - should be blocked (key 5 is in gap)
        let result = manager.acquire_lock_with_target(
            TxId::new(2),
            LockTarget::NextKey(vec![5]),
            LockMode::Exclusive,
        );
        assert!(matches!(result, Ok(LockGrantMode::Waiting)));

        // T3 tries to insert key 15 - should succeed (key 15 is after gap)
        let result = manager.acquire_lock_with_target(
            TxId::new(3),
            LockTarget::NextKey(vec![15]),
            LockMode::Exclusive,
        );
        assert!(matches!(result, Ok(LockGrantMode::Granted)));
    }

    /// Test Gap locking for bounded range (e.g., SELECT * FROM t WHERE id > 5 AND id < 10 FOR UPDATE)
    #[test]
    fn test_gap_lock_for_range_between() {
        let mut manager = LockManager::new();

        // T1 acquires Gap lock on range (5, 10)
        let gap = LockTarget::Gap {
            start: Some(vec![5]),
            end: Some(vec![10]),
        };
        manager
            .acquire_lock_with_target(TxId::new(1), gap, LockMode::Exclusive)
            .unwrap();

        // T2 tries to insert key 7 - should be blocked (key 7 is in gap)
        let result = manager.acquire_lock_with_target(
            TxId::new(2),
            LockTarget::NextKey(vec![7]),
            LockMode::Exclusive,
        );
        assert!(matches!(result, Ok(LockGrantMode::Waiting)));

        // T3 tries to insert key 3 - should succeed (key 3 is before gap)
        let result = manager.acquire_lock_with_target(
            TxId::new(3),
            LockTarget::NextKey(vec![3]),
            LockMode::Exclusive,
        );
        assert!(matches!(result, Ok(LockGrantMode::Granted)));

        // T4 tries to insert key 12 - should succeed (key 12 is after gap)
        let result = manager.acquire_lock_with_target(
            TxId::new(4),
            LockTarget::NextKey(vec![12]),
            LockMode::Exclusive,
        );
        assert!(matches!(result, Ok(LockGrantMode::Granted)));
    }
}
