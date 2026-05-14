use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    entries: HashMap<String, u64>,
}

impl Hash for VectorClock {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut entries: Vec<_> = self.entries.iter().collect();
        entries.sort();
        entries.hash(state);
    }
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn with_entry(mut self, node_id: impl Into<String>, counter: u64) -> Self {
        self.entries.insert(node_id.into(), counter);
        self
    }

    pub fn increment(&mut self, node_id: &str) {
        let new_val = self.entries.get(node_id).copied().unwrap_or(0) + 1;
        self.entries.insert(node_id.to_string(), new_val);
    }

    pub fn get(&self, node_id: &str) -> u64 {
        self.entries.get(node_id).copied().unwrap_or(0)
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (node, counter) in &other.entries {
            let current = self.entries.get(node).copied().unwrap_or(0);
            if *counter > current {
                self.entries.insert(node.clone(), *counter);
            }
        }
    }

    pub fn happens_before(&self, other: &VectorClock) -> bool {
        let mut dominated = true;
        let mut strictly_less = false;

        for (node, &self_counter) in &self.entries {
            let other_counter = other.get(node);
            if self_counter > other_counter {
                dominated = false;
                break;
            }
            if self_counter < other_counter {
                strictly_less = true;
            }
        }

        for (node, &other_counter) in &other.entries {
            if !self.entries.contains_key(node) && other_counter > 0 {
                strictly_less = true;
            }
        }

        dominated && strictly_less
    }

    pub fn concurrent_with(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self) && self != other
    }

    pub fn entries(&self) -> &HashMap<String, u64> {
        &self.entries
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VectorClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pairs: Vec<String> = self.entries
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect();
        write!(f, "{{{}}}", pairs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        let mut clock = VectorClock::new();
        clock.increment("node-a");
        clock.increment("node-a");
        clock.increment("node-b");

        assert_eq!(clock.get("node-a"), 2);
        assert_eq!(clock.get("node-b"), 1);
        assert_eq!(clock.get("node-c"), 0);
    }

    #[test]
    fn test_merge() {
        let mut clock1 = VectorClock::new().with_entry("node-a", 1).with_entry("node-b", 1);
        let clock2 = VectorClock::new().with_entry("node-b", 2).with_entry("node-c", 1);

        clock1.merge(&clock2);

        assert_eq!(clock1.get("node-a"), 1);
        assert_eq!(clock1.get("node-b"), 2);
        assert_eq!(clock1.get("node-c"), 1);
    }

    #[test]
    fn test_happens_before() {
        let clock1 = VectorClock::new().with_entry("node-a", 1).with_entry("node-b", 1);
        let clock2 = VectorClock::new().with_entry("node-a", 2).with_entry("node-b", 1);

        assert!(clock1.happens_before(&clock2));
        assert!(!clock2.happens_before(&clock1));
    }

    #[test]
    fn test_concurrent() {
        let clock1 = VectorClock::new().with_entry("node-a", 1).with_entry("node-b", 2);
        let clock2 = VectorClock::new().with_entry("node-a", 2).with_entry("node-b", 1);

        assert!(clock1.concurrent_with(&clock2));
    }
}
