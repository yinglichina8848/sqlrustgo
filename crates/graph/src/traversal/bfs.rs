//! BFS (Breadth-First Search) traversal

use crate::model::NodeId;
use std::collections::{HashSet, VecDeque};

/// BFS traversal result
pub struct BfsResult {
    /// Visited nodes in order
    pub visited: Vec<NodeId>,
    /// Distance from start node
    pub distances: std::collections::HashMap<NodeId, u64>,
    /// Parent map for path reconstruction
    pub parents: std::collections::HashMap<NodeId, NodeId>,
}

impl BfsResult {
    /// Reconstruct path from start to target
    pub fn reconstruct_path(&self, start: NodeId, target: NodeId) -> Option<Vec<NodeId>> {
        if !self.parents.contains_key(&target) && target != start {
            return None;
        }

        let mut path = vec![target];
        let mut current = target;

        while current != start {
            if let Some(&parent) = self.parents.get(&current) {
                path.push(parent);
                current = parent;
            } else {
                return None;
            }
        }

        path.reverse();
        Some(path)
    }
}

/// BFS iterator for lazy evaluation
pub struct BfsIterator<'a> {
    queue: VecDeque<NodeId>,
    visited: HashSet<NodeId>,
    get_neighbors: &'a dyn Fn(NodeId) -> Vec<NodeId>,
}

impl<'a> BfsIterator<'a> {
    pub fn new(start: NodeId, get_neighbors: &'a dyn Fn(NodeId) -> Vec<NodeId>) -> Self {
        let mut visited = HashSet::new();
        visited.insert(start);

        BfsIterator {
            queue: VecDeque::from([start]),
            visited,
            get_neighbors,
        }
    }
}

impl<'a> Iterator for BfsIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_front()?;

        let neighbors = (self.get_neighbors)(node);
        for neighbor in neighbors {
            if !self.visited.contains(&neighbor) {
                self.visited.insert(neighbor);
                self.queue.push_back(neighbor);
            }
        }

        Some(node)
    }
}

/// Perform BFS and collect all visited nodes
pub fn bfs_collect<G>(graph: &G, start: NodeId) -> Vec<NodeId>
where
    G: Fn(NodeId) -> Vec<NodeId>,
{
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(start);
    visited.insert(start);

    while let Some(node) = queue.pop_front() {
        result.push(node);

        let neighbors = graph(node);
        for neighbor in neighbors {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }

    result
}

/// Perform BFS with distance tracking
pub fn bfs_with_distances<G>(graph: &G, start: NodeId) -> BfsResult
where
    G: Fn(NodeId) -> Vec<NodeId>,
{
    let mut visited = Vec::new();
    let mut distances = std::collections::HashMap::new();
    let mut parents = std::collections::HashMap::new();
    let mut queue = VecDeque::new();

    queue.push_back((start, 0));
    visited.push(start);
    distances.insert(start, 0);

    while let Some((node, dist)) = queue.pop_front() {
        let neighbors = graph(node);

        for neighbor in neighbors {
            if let std::collections::hash_map::Entry::Vacant(e) = distances.entry(neighbor) {
                e.insert(dist + 1);
                parents.insert(neighbor, node);
                queue.push_back((neighbor, dist + 1));
                visited.push(neighbor);
            }
        }
    }

    BfsResult {
        visited,
        distances,
        parents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bfs_collect() {
        // Create a simple graph:
        // 1 -> 2, 1 -> 3, 2 -> 4, 3 -> 4
        let graph = |node: NodeId| -> Vec<NodeId> {
            match node {
                NodeId(1) => vec![NodeId(2), NodeId(3)],
                NodeId(2) => vec![NodeId(4)],
                NodeId(3) => vec![NodeId(4)],
                _ => vec![],
            }
        };

        let result = bfs_collect(&graph, NodeId(1));
        assert!(result.contains(&NodeId(1)));
        assert!(result.contains(&NodeId(2)));
        assert!(result.contains(&NodeId(3)));
        assert!(result.contains(&NodeId(4)));
        // 1 should come before 4
        assert!(
            result.iter().position(|&n| n == NodeId(1))
                < result.iter().position(|&n| n == NodeId(4))
        );
    }

    #[test]
    fn test_bfs_with_distances() {
        let graph = |node: NodeId| -> Vec<NodeId> {
            match node {
                NodeId(1) => vec![NodeId(2), NodeId(3)],
                NodeId(2) => vec![NodeId(4)],
                NodeId(3) => vec![NodeId(4)],
                _ => vec![],
            }
        };

        let result = bfs_with_distances(&graph, NodeId(1));

        assert_eq!(result.distances.get(&NodeId(1)), Some(&0));
        assert_eq!(result.distances.get(&NodeId(2)), Some(&1));
        assert_eq!(result.distances.get(&NodeId(3)), Some(&1));
        assert_eq!(result.distances.get(&NodeId(4)), Some(&2));
    }

    #[test]
    fn test_bfs_path_reconstruction() {
        let graph = |node: NodeId| -> Vec<NodeId> {
            match node {
                NodeId(1) => vec![NodeId(2)],
                NodeId(2) => vec![NodeId(3)],
                NodeId(3) => vec![NodeId(4)],
                _ => vec![],
            }
        };

        let result = bfs_with_distances(&graph, NodeId(1));
        let path = result.reconstruct_path(NodeId(1), NodeId(4));

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path, vec![NodeId(1), NodeId(2), NodeId(3), NodeId(4)]);
    }

    #[test]
    fn test_bfs_iterator() {
        let graph = |node: NodeId| -> Vec<NodeId> {
            match node {
                NodeId(1) => vec![NodeId(2), NodeId(3)],
                NodeId(2) => vec![NodeId(4)],
                _ => vec![],
            }
        };

        let iter = BfsIterator::new(NodeId(1), &graph);
        let collected: Vec<NodeId> = iter.collect();

        assert!(collected.contains(&NodeId(1)));
        assert!(collected.contains(&NodeId(2)));
        assert!(collected.contains(&NodeId(3)));
        assert!(collected.contains(&NodeId(4)));
    }
}
