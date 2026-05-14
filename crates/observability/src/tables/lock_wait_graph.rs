use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockWaitEdge {
    pub waiter_tx_id: u64,
    pub holder_tx_id: u64,
    pub lock_key: String,
    pub lock_mode: String,
    pub wait_start_time: u64,
}

pub struct LockWaitGraph {
    edges: VecDeque<LockWaitEdge>,
    active_waits: HashMap<u64, LockWaitEdge>,
}

impl LockWaitGraph {
    pub fn new() -> Self {
        Self {
            edges: VecDeque::new(),
            active_waits: HashMap::new(),
        }
    }

    pub fn add_wait(&mut self, edge: LockWaitEdge) {
        self.edges.push_back(edge.clone());
        self.active_waits.insert(edge.waiter_tx_id, edge);
    }

    pub fn remove_wait(&mut self, tx_id: u64) {
        self.active_waits.remove(&tx_id);
    }

    pub fn get_active_waits(&self) -> Vec<&LockWaitEdge> {
        self.active_waits.values().collect()
    }

    pub fn detect_deadlock(&self) -> Vec<Vec<u64>> {
        let mut cycles = Vec::new();

        for &waiter in self.active_waits.keys() {
            let mut path = Vec::new();
            let mut current_waiter = waiter;

            path.push(current_waiter);

            while let Some(current_edge) = self.active_waits.get(&current_waiter) {
                let holder = current_edge.holder_tx_id;

                if holder == waiter {
                    path.push(holder);
                    cycles.push(path);
                    break;
                }

                if path.contains(&holder) {
                    break;
                }

                if self.active_waits.contains_key(&holder) {
                    path.push(holder);
                    current_waiter = holder;
                } else {
                    break;
                }
            }
        }

        cycles
    }

    pub fn clear(&mut self) {
        self.edges.clear();
        self.active_waits.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.active_waits.is_empty()
    }

    pub fn len(&self) -> usize {
        self.active_waits.len()
    }
}

impl Default for LockWaitGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_wait_graph_add_remove() {
        let mut graph = LockWaitGraph::new();
        let edge = LockWaitEdge {
            waiter_tx_id: 2,
            holder_tx_id: 1,
            lock_key: "table:users:1".to_string(),
            lock_mode: "X".to_string(),
            wait_start_time: 1000,
        };
        graph.add_wait(edge);
        assert_eq!(graph.len(), 1);
        assert_eq!(graph.get_active_waits().len(), 1);

        graph.remove_wait(2);
        assert!(graph.is_empty());
    }

    #[test]
    fn test_lock_wait_graph_detect_deadlock() {
        let mut graph = LockWaitGraph::new();

        let edge1 = LockWaitEdge {
            waiter_tx_id: 1,
            holder_tx_id: 2,
            lock_key: "lock:A".to_string(),
            lock_mode: "X".to_string(),
            wait_start_time: 1000,
        };
        let edge2 = LockWaitEdge {
            waiter_tx_id: 2,
            holder_tx_id: 1,
            lock_key: "lock:B".to_string(),
            lock_mode: "X".to_string(),
            wait_start_time: 1001,
        };

        graph.add_wait(edge1);
        graph.add_wait(edge2);

        let cycles = graph.detect_deadlock();
        assert!(!cycles.is_empty());
    }
}
