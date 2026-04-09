use crate::api::{GraphResult, UnifiedQueryRequest};
use crate::error::QueryResult;
use crate::QueryPlan;
use sqlrustgo_graph::store::InMemoryGraphStore;
use sqlrustgo_graph::GraphStore;
use sqlrustgo_graph::model::{NodeId, PropertyMap};

/// Graph adapter for traversal queries
pub struct GraphAdapter {
    store: InMemoryGraphStore,
}

impl GraphAdapter {
    pub fn new() -> Self {
        Self {
            store: InMemoryGraphStore::new(),
        }
    }

    pub async fn execute(
        &self,
        request: &UnifiedQueryRequest,
        _plan: &QueryPlan,
    ) -> QueryResult<Vec<GraphResult>> {
        // Extract graph query from request
        let graph_query = match &request.graph_query {
            Some(gq) => gq,
            None => {
                return QueryResult::Err("No graph query provided".to_string());
            }
        };

        // Get start nodes
        let start_nodes = self.resolve_start_nodes(&graph_query.start_nodes);
        
        if start_nodes.is_empty() {
            return QueryResult::Err("No valid start nodes found".to_string());
        }

        let max_depth = graph_query.max_depth;
        let results = match graph_query.traversal {
            crate::api::TraversalType::BFS => {
                self.bfs_traverse(&start_nodes, max_depth)
            }
            crate::api::TraversalType::DFS => {
                self.dfs_traverse(&start_nodes, max_depth)
            }
        };

        QueryResult::Ok(results)
    }

    /// Resolve start node names/IDs to NodeIds
    fn resolve_start_nodes(&self, start_nodes: &[String]) -> Vec<NodeId> {
        start_nodes
            .iter()
            .filter_map(|name| {
                // Try to parse as NodeId number first
                if let Ok(num) = name.parse::<u64>() {
                    let node_id = NodeId(num);
                    // Verify node exists
                    if self.store.get_node(node_id).is_some() {
                        return Some(node_id);
                    }
                }
                
                // Try to find by label
                let nodes = self.store.nodes_by_label(name);
                if !nodes.is_empty() {
                    return Some(nodes[0]);
                }
                
                None
            })
            .collect()
    }

    /// BFS traversal
    fn bfs_traverse(&self, start_nodes: &[NodeId], max_depth: u32) -> Vec<GraphResult> {
        use std::collections::{HashSet, VecDeque};
        
        let mut results = Vec::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut queue: VecDeque<(NodeId, u32, Vec<NodeId>)> = VecDeque::new();
        
        // Initialize with start nodes
        for &start in start_nodes {
            if !visited.contains(&start) {
                visited.insert(start);
                queue.push_back((start, 0, vec![start]));
            }
        }
        
        while let Some((current, depth, path)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            
            // Get neighbors
            for neighbor in self.store.outgoing_neighbors(current) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    
                    // Calculate score based on depth (shallower = higher score)
                    let score = 1.0 - (depth as f32 * 0.1).min(0.9);
                    
                    results.push(GraphResult {
                        path: new_path.iter().map(|n| format!("{:?}", n)).collect(),
                        score,
                        depth: depth + 1,
                    });
                    
                    queue.push_back((neighbor, depth + 1, new_path));
                }
            }
        }
        
        results
    }

    /// DFS traversal
    fn dfs_traverse(&self, start_nodes: &[NodeId], max_depth: u32) -> Vec<GraphResult> {
        use std::collections::HashSet;
        
        let mut results = Vec::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<(NodeId, u32, Vec<NodeId>)> = Vec::new();
        
        // Initialize with start nodes
        for &start in start_nodes {
            if !visited.contains(&start) {
                stack.push((start, 0, vec![start]));
            }
        }
        
        while let Some((current, depth, path)) = stack.pop() {
            if depth >= max_depth {
                continue;
            }
            
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            
            // Get neighbors (reverse for DFS order)
            let neighbors: Vec<NodeId> = self.store.outgoing_neighbors(current);
            for neighbor in neighbors.iter().rev() {
                if !visited.contains(neighbor) {
                    let mut new_path = path.clone();
                    new_path.push(*neighbor);
                    
                    // Calculate score based on depth (shallower = higher score)
                    let score = 1.0 - (depth as f32 * 0.1).min(0.9);
                    
                    results.push(GraphResult {
                        path: new_path.iter().map(|n| format!("{:?}", n)).collect(),
                        score,
                        depth: depth + 1,
                    });
                    
                    stack.push((*neighbor, depth + 1, new_path));
                }
            }
        }
        
        results
    }

    /// Add a node to the graph (for testing purposes)
    pub fn add_node(&mut self, label: &str, props: PropertyMap) -> NodeId {
        self.store.create_node(label, props)
    }

    /// Add an edge between nodes (for testing purposes)
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, label: &str) -> std::result::Result<sqlrustgo_graph::model::EdgeId, String> {
        self.store
            .create_edge(from, to, label, PropertyMap::new())
            .map_err(|e| e.to_string())
    }

    /// Get node by ID (for testing purposes)
    pub fn get_node(&self, id: NodeId) -> Option<sqlrustgo_graph::model::Node> {
        self.store.get_node(id)
    }
}

impl Default for GraphAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graph_adapter_traverse() {
        let mut adapter = GraphAdapter::new();
        
        // Build a simple graph: A -> B -> C
        let node_a = adapter.add_node("A", PropertyMap::new());
        let node_b = adapter.add_node("B", PropertyMap::new());
        let node_c = adapter.add_node("C", PropertyMap::new());
        
        adapter.add_edge(node_a, node_b, "rel").unwrap();
        adapter.add_edge(node_b, node_c, "rel").unwrap();
        
        let request = UnifiedQueryRequest {
            query: "traverse A".to_string(),
            mode: crate::api::QueryMode::Graph,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: Some(crate::api::GraphQuery {
                start_nodes: vec![format!("{}", node_a)],
                traversal: crate::api::TraversalType::DFS,
                max_depth: 3,
            }),
            top_k: Some(10),
            offset: Some(0),
        };
        
        let plan = QueryPlan {
            execute_sql: false,
            execute_vector: false,
            execute_graph: true,
            weights: Default::default(),
            top_k: 10,
            offset: 0,
        };
        
        let results = adapter.execute(&request, &plan).await;
        assert!(results.is_ok());
        
        let graph_results = results.unwrap();
        assert!(!graph_results.is_empty());
    }
}
