//! DFS (Depth-First Search) traversal

use crate::model::NodeId;
use std::collections::HashSet;

/// DFS traversal result
pub struct DfsResult {
    /// Visited nodes in DFS order
    pub visited: Vec<NodeId>,
    /// Entry times (discovery order)
    pub entry_time: std::collections::HashMap<NodeId, u64>,
    /// Exit times (finish order)
    pub exit_time: std::collections::HashMap<NodeId, u64>,
    /// Parent map
    pub parents: std::collections::HashMap<NodeId, NodeId>,
}

impl DfsResult {
    /// Reconstruct path from start to target using parents
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

/// DFS iterator for lazy evaluation
pub struct DfsIterator<'a> {
    stack: Vec<NodeId>,
    visited: HashSet<NodeId>,
    get_neighbors: &'a dyn Fn(NodeId) -> Vec<NodeId>,
}

impl<'a> DfsIterator<'a> {
    pub fn new(start: NodeId, get_neighbors: &'a dyn Fn(NodeId) -> Vec<NodeId>) -> Self {
        DfsIterator {
            stack: vec![start],
            visited: HashSet::new(),
            get_neighbors,
        }
    }
}

impl<'a> Iterator for DfsIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            if self.visited.contains(&node) {
                continue;
            }
            self.visited.insert(node);

            // Add neighbors in reverse order so first neighbor is processed first
            let neighbors = (self.get_neighbors)(node);
            for neighbor in neighbors.iter().rev() {
                if !self.visited.contains(neighbor) {
                    self.stack.push(*neighbor);
                }
            }

            return Some(node);
        }

        None
    }
}

/// Perform DFS and collect all visited nodes (recursive version)
pub fn dfs_collect_recursive<G>(graph: &G, start: NodeId) -> Vec<NodeId>
where
    G: Fn(NodeId) -> Vec<NodeId>,
{
    let mut visited = Vec::new();
    let mut seen = HashSet::new();

    fn dfs<G>(node: NodeId, graph: &G, visited: &mut Vec<NodeId>, seen: &mut HashSet<NodeId>)
    where
        G: Fn(NodeId) -> Vec<NodeId>,
    {
        if seen.contains(&node) {
            return;
        }
        seen.insert(node);
        visited.push(node);

        for neighbor in graph(node) {
            dfs(neighbor, graph, visited, seen);
        }
    }

    dfs(start, graph, &mut visited, &mut seen);
    visited
}

/// Perform DFS and collect all visited nodes (iterative version)
pub fn dfs_collect<G>(graph: &G, start: NodeId) -> Vec<NodeId>
where
    G: Fn(NodeId) -> Vec<NodeId>,
{
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = vec![start];

    while let Some(node) = stack.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node);
        result.push(node);

        let neighbors = graph(node);
        for neighbor in neighbors.iter().rev() {
            if !visited.contains(neighbor) {
                stack.push(*neighbor);
            }
        }
    }

    result
}

/// Perform DFS with timing information
pub fn dfs_with_timing<G>(graph: &G, start: NodeId) -> DfsResult
where
    G: Fn(NodeId) -> Vec<NodeId>,
{
    let visited = Vec::new();
    let mut entry_time = std::collections::HashMap::new();
    let mut exit_time = std::collections::HashMap::new();
    let mut parents = std::collections::HashMap::new();
    let mut time = 0;
    let mut visited_set = HashSet::new();
    let mut stack = vec![(start, None)];

    while let Some((node, parent)) = stack.pop() {
        if let Some(p) = parent {
            parents.insert(node, p);
        }

        if visited_set.contains(&node) {
            // Exit time
            time += 1;
            exit_time.insert(node, time);
            continue;
        }

        visited_set.insert(node);
        time += 1;
        entry_time.insert(node, time);

        // Push exit marker
        stack.push((node, None));

        // Push neighbors in reverse order
        let neighbors = graph(node);
        for neighbor in neighbors.iter().rev() {
            if !visited_set.contains(neighbor) {
                stack.push((*neighbor, Some(node)));
            }
        }
    }

    DfsResult {
        visited,
        entry_time,
        exit_time,
        parents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfs_collect() {
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

        let result = dfs_collect(&graph, NodeId(1));
        assert!(result.contains(&NodeId(1)));
        assert!(result.contains(&NodeId(2)));
        assert!(result.contains(&NodeId(3)));
        assert!(result.contains(&NodeId(4)));
    }

    #[test]
    fn test_dfs_with_timing() {
        let graph = |node: NodeId| -> Vec<NodeId> {
            match node {
                NodeId(1) => vec![NodeId(2)],
                NodeId(2) => vec![NodeId(3)],
                _ => vec![],
            }
        };

        let result = dfs_with_timing(&graph, NodeId(1));

        assert!(result.entry_time.contains_key(&NodeId(1)));
        assert!(result.entry_time.contains_key(&NodeId(2)));
        assert!(result.exit_time.contains_key(&NodeId(1)));
        assert!(result.exit_time.contains_key(&NodeId(2)));

        // Entry time of 1 should be < entry time of 2
        assert!(result.entry_time.get(&NodeId(1)) < result.entry_time.get(&NodeId(2)));
    }

    #[test]
    fn test_dfs_path_reconstruction() {
        let graph = |node: NodeId| -> Vec<NodeId> {
            match node {
                NodeId(1) => vec![NodeId(2)],
                NodeId(2) => vec![NodeId(3)],
                NodeId(3) => vec![NodeId(4)],
                _ => vec![],
            }
        };

        let result = dfs_with_timing(&graph, NodeId(1));
        let path = result.reconstruct_path(NodeId(1), NodeId(4));

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path, vec![NodeId(1), NodeId(2), NodeId(3), NodeId(4)]);
    }

    #[test]
    fn test_dfs_iterator() {
        let graph = |node: NodeId| -> Vec<NodeId> {
            match node {
                NodeId(1) => vec![NodeId(2), NodeId(3)],
                NodeId(2) => vec![],
                NodeId(3) => vec![],
                _ => vec![],
            }
        };

        let iter = DfsIterator::new(NodeId(1), &graph);
        let collected: Vec<NodeId> = iter.collect();

        assert!(collected.contains(&NodeId(1)));
        assert!(collected.contains(&NodeId(2)));
        assert!(collected.contains(&NodeId(3)));
    }
}
