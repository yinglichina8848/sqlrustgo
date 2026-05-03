//! Graph builder: AST symbols → Code Graph
//!
//! Converts extracted symbols into a directed graph with edges:
//! - `Contains`: nodes in the same file
//! - `Calls`: function → called function (regex-based heuristic)

use crate::graph::{CodeGraph, Edge, EdgeType, Node};

/// Build a code graph from a list of nodes
pub fn build_graph(nodes: Vec<Node>) -> CodeGraph {
    let mut graph = CodeGraph::with_capacity(nodes.len(), nodes.len() * 2);
    graph.nodes = nodes;

    // Phase 1: Add Contains edges for nodes in the same file
    let file_groups = group_nodes_by_file(&graph.nodes);
    for (file_path, node_ids) in file_groups {
        // Create a virtual module node for the file
        let module_id = format!("mod:{file_path}");
        if !graph.nodes.iter().any(|n| n.id == module_id) {
            let file_name = std::path::Path::new(&file_path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| file_path.clone());
            let module_node = Node::new(
                file_name,
                crate::graph::NodeType::Module,
                file_path.to_string(),
                1,
                0,
                None,
            );
            graph.nodes.push(module_node);
        }

        for node_id in &node_ids {
            graph.edges.push(Edge::new(
                module_id.clone(),
                node_id.clone(),
                EdgeType::Contains,
            ));
        }
    }

    // Phase 2: Build call graph edges (heuristic: parse function bodies)
    // This requires source access - handled separately in ast parser
    // For P0, we build import-based DependsOn edges

    graph
}

/// Group node IDs by their file path
fn group_nodes_by_file(nodes: &[Node]) -> std::collections::HashMap<String, Vec<String>> {
    let mut groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for node in nodes {
        groups
            .entry(node.file_path.clone())
            .or_default()
            .push(node.id.clone());
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_empty() {
        let graph = build_graph(vec![]);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_build_with_nodes() {
        let nodes = vec![
            Node::new(
                "add".to_string(),
                crate::graph::NodeType::Function,
                "src/lib.rs".to_string(),
                10,
                15,
                None,
            ),
            Node::new(
                "User".to_string(),
                crate::graph::NodeType::Struct,
                "src/lib.rs".to_string(),
                1,
                9,
                None,
            ),
        ];
        let graph = build_graph(nodes);
        assert_eq!(graph.node_count(), 3); // 2 nodes + 1 module
        assert_eq!(graph.edge_count(), 2); // module→add, module→User
    }
}
