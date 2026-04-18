use crate::model::{NodeId, PropertyMap};
use crate::store::{GraphStore, InMemoryGraphStore};
use std::collections::HashMap;

pub struct GraphGenerator {
    seed: u64,
}

impl GraphGenerator {
    pub fn new(seed: u64) -> Self {
        GraphGenerator { seed }
    }

    pub fn generate(&self, node_count: usize, edges_per_node: usize) -> InMemoryGraphStore {
        let mut store = InMemoryGraphStore::new();
        let mut rng = SimpleRng::new(self.seed);

        if node_count == 0 {
            return store;
        }

        let m = edges_per_node.min(node_count.saturating_sub(1));
        if m == 0 {
            for _ in 0..node_count {
                store.create_node("Node", PropertyMap::new());
            }
            return store;
        }

        let initial_nodes = m + 1;
        for _i in 0..initial_nodes {
            store.create_node("Node", PropertyMap::new());
        }

        let mut degree_sum = 0;
        let mut degrees: HashMap<usize, usize> = HashMap::new();
        for i in 0..initial_nodes {
            degrees.insert(i, m);
            degree_sum += m;
        }

        for i in 0..initial_nodes {
            for j in (i + 1)..initial_nodes {
                store
                    .create_edge(
                        NodeId(i as u64),
                        NodeId(j as u64),
                        "connects",
                        PropertyMap::new(),
                    )
                    .unwrap();
            }
        }

        for new_node_id in initial_nodes..node_count {
            store.create_node("Node", PropertyMap::new());
            let new_idx = new_node_id;

            let mut targets = Vec::with_capacity(m);
            let mut current_degree_sum = degree_sum;

            while targets.len() < m {
                let threshold = rng.next_double() * current_degree_sum as f64;
                let mut cumsum = 0.0;

                for (node_idx, &degree) in degrees.iter() {
                    cumsum += degree as f64;
                    if cumsum > threshold && !targets.contains(node_idx) {
                        targets.push(*node_idx);
                        *degrees.entry(*node_idx).or_insert(0) += 1;
                        degree_sum += 1;
                        current_degree_sum += 1;
                        break;
                    }
                }
            }

            for &target in &targets {
                store
                    .create_edge(
                        NodeId(new_idx as u64),
                        NodeId(target as u64),
                        "connects",
                        PropertyMap::new(),
                    )
                    .unwrap();
            }
        }

        store
    }
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        SimpleRng { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_double(&mut self) -> f64 {
        self.next() as f64 / (u64::MAX as f64)
    }
}

#[allow(dead_code)]
fn is_connected(store: &InMemoryGraphStore) -> bool {
    if store.node_count() == 0 {
        return true;
    }

    let mut visited = std::collections::HashSet::new();
    let mut queue = Vec::new();

    let start = NodeId(0);
    queue.push(start);
    visited.insert(start);

    while let Some(node) = queue.pop() {
        for neighbor in store.outgoing_neighbors(node) {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                queue.push(neighbor);
            }
        }
        for neighbor in store.incoming_neighbors(node) {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                queue.push(neighbor);
            }
        }
    }

    visited.len() == store.node_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_generator_100_nodes() {
        let gen = GraphGenerator::new(42);
        let store = gen.generate(100, 3);
        assert_eq!(store.node_count(), 100);
    }

    #[test]
    fn test_graph_generator_small() {
        let gen = GraphGenerator::new(42);
        let store = gen.generate(10, 2);
        assert_eq!(store.node_count(), 10);
        assert!(store.edge_count() > 0);
    }

    #[test]
    fn test_graph_generator_deterministic() {
        let gen1 = GraphGenerator::new(42);
        let store1 = gen1.generate(50, 3);

        let gen2 = GraphGenerator::new(42);
        let store2 = gen2.generate(50, 3);

        assert_eq!(store1.node_count(), store2.node_count());
        assert_eq!(store1.edge_count(), store2.edge_count());

        for i in 0..store1.node_count() {
            let node_id = NodeId(i as u64);
            let neighbors1: Vec<_> = store1.outgoing_neighbors(node_id);
            let neighbors2: Vec<_> = store2.outgoing_neighbors(node_id);
            assert_eq!(neighbors1.len(), neighbors2.len());
        }
    }

    #[test]
    fn test_graph_generator_connectivity() {
        let gen = GraphGenerator::new(42);
        let store = gen.generate(100, 3);
        assert!(is_connected(&store));
    }

    #[test]
    fn test_graph_generator_zero_nodes() {
        let gen = GraphGenerator::new(42);
        let store = gen.generate(0, 3);
        assert_eq!(store.node_count(), 0);
    }

    #[test]
    fn test_graph_generator_single_node() {
        let gen = GraphGenerator::new(42);
        let store = gen.generate(1, 3);
        assert_eq!(store.node_count(), 1);
        assert_eq!(store.edge_count(), 0);
    }

    #[test]
    fn test_graph_generator_edges_per_node_limited() {
        let gen = GraphGenerator::new(42);
        let store = gen.generate(5, 10);
        assert_eq!(store.node_count(), 5);
        assert!(store.edge_count() <= 10);
    }
}
