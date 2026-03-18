use crate::mvcc::TxId;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

#[derive(Debug)]
pub struct DeadlockDetector {
    waits_for: HashMap<TxId, HashSet<TxId>>,
    #[allow(dead_code)]
    lock_wait_timeout: Duration, // Reserved for future timeout functionality
}

impl DeadlockDetector {
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(5))
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            waits_for: HashMap::new(),
            lock_wait_timeout: timeout,
        }
    }

    pub fn get_timeout(&self) -> Duration {
        self.lock_wait_timeout
    }

    pub fn add_edge(&mut self, blocked: TxId, holder: TxId) {
        self.waits_for.entry(blocked).or_default().insert(holder);
    }

    pub fn remove_edges_for(&mut self, tx_id: TxId) {
        self.waits_for.remove(&tx_id);
        for holders in self.waits_for.values_mut() {
            holders.remove(&tx_id);
        }
    }

    pub fn detect_cycle(&self, start: TxId) -> Option<Vec<TxId>> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        self.dfs(start, &mut visited, &mut path)
    }

    fn dfs(
        &self,
        current: TxId,
        visited: &mut HashSet<TxId>,
        path: &mut Vec<TxId>,
    ) -> Option<Vec<TxId>> {
        if path.contains(&current) {
            let idx = path.iter().position(|&x| x == current).unwrap();
            return Some(path[idx..].to_vec());
        }

        if visited.contains(&current) {
            return None;
        }

        visited.insert(current);
        path.push(current);

        if let Some(holders) = self.waits_for.get(&current) {
            for &holder in holders {
                if let Some(cycle) = self.dfs(holder, visited, path) {
                    return Some(cycle);
                }
            }
        }

        path.pop();
        None
    }
}

impl Default for DeadlockDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deadlock_detector_new() {
        let detector = DeadlockDetector::new();
        assert!(detector.waits_for.is_empty());
    }

    #[test]
    fn test_add_edge() {
        let mut detector = DeadlockDetector::new();
        detector.add_edge(TxId::new(1), TxId::new(2));
        assert!(detector
            .waits_for
            .get(&TxId::new(1))
            .unwrap()
            .contains(&TxId::new(2)));
    }

    #[test]
    fn test_detect_cycle() {
        let mut detector = DeadlockDetector::new();
        detector.add_edge(TxId::new(1), TxId::new(2));
        detector.add_edge(TxId::new(2), TxId::new(3));
        detector.add_edge(TxId::new(3), TxId::new(1));

        let cycle = detector.detect_cycle(TxId::new(1));
        assert!(cycle.is_some());
    }

    #[test]
    fn test_no_cycle() {
        let mut detector = DeadlockDetector::new();
        detector.add_edge(TxId::new(1), TxId::new(2));
        detector.add_edge(TxId::new(2), TxId::new(3));

        let cycle = detector.detect_cycle(TxId::new(1));
        assert!(cycle.is_none());
    }
}
