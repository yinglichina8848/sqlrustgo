use crate::model::NodeId;
use std::collections::HashSet;

pub fn multi_hop<G>(graph: G, start: NodeId, depth: usize) -> Vec<NodeId>
where
    G: Fn(NodeId) -> Vec<NodeId>,
{
    let mut current_layer: Vec<NodeId> = vec![start];
    let mut visited: HashSet<NodeId> = HashSet::new();
    visited.insert(start);

    for _ in 0..depth {
        let mut next_layer: Vec<NodeId> = Vec::new();
        for node in current_layer {
            let neighbors = graph(node);
            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    next_layer.push(neighbor);
                }
            }
        }
        current_layer = next_layer;
    }

    current_layer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2hop_query() {
        let graph = |n: NodeId| -> Vec<NodeId> {
            match n {
                NodeId(1) => vec![NodeId(2)],
                NodeId(2) => vec![NodeId(3)],
                NodeId(3) => vec![NodeId(4)],
                _ => vec![],
            }
        };

        let result = multi_hop(graph, NodeId(1), 2);
        assert!(result.contains(&NodeId(3)));
    }

    #[test]
    fn test_3hop_query() {
        let graph = |n: NodeId| -> Vec<NodeId> {
            match n {
                NodeId(1) => vec![NodeId(2)],
                NodeId(2) => vec![NodeId(3), NodeId(5)],
                NodeId(3) => vec![NodeId(4)],
                _ => vec![],
            }
        };

        let result = multi_hop(graph, NodeId(1), 3);
        assert!(result.contains(&NodeId(4)));
    }
}
