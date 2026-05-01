//! Graph Cost Model Module
//!
//! Provides cost estimation for graph operations (traversal, pattern matching, shortest path).

use crate::unified_plan::{GraphPattern, GraphScanType, UnifiedPlan};
use std::collections::HashMap;
use std::hash::Hash;

/// Graph index types for cost estimation
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub enum GraphIndexType {
    #[default]
    /// Adjacency list representation
    AdjacencyList,
    /// Compressed adjacency
    Compressed,
    /// Labeled graph index
    LabeledIndex,
    /// Spatial-temporal graph index
    SpatioTemporal,
}

/// Cost factors for graph operations
#[derive(Debug, Clone)]
pub struct GraphCostFactors {
    /// CPU cost per node visited
    pub cpu_cost_per_node: f64,
    /// CPU cost per edge traversed
    pub cpu_cost_per_edge: f64,
    /// Memory cost per node
    pub memory_cost_per_node: f64,
    /// I/O cost per page for graph storage
    pub io_cost_per_page: f64,
    /// Pattern matching complexity multiplier
    pub pattern_match_multiplier: f64,
    /// Index type specific cost multipliers
    pub index_type_multipliers: HashMap<GraphIndexType, f64>,
}

impl Default for GraphCostFactors {
    fn default() -> Self {
        let mut index_type_multipliers = HashMap::new();
        index_type_multipliers.insert(GraphIndexType::AdjacencyList, 1.0);
        index_type_multipliers.insert(GraphIndexType::Compressed, 0.8);
        index_type_multipliers.insert(GraphIndexType::LabeledIndex, 0.5);
        index_type_multipliers.insert(GraphIndexType::SpatioTemporal, 0.6);

        Self {
            cpu_cost_per_node: 0.01,
            cpu_cost_per_edge: 0.001,
            memory_cost_per_node: 0.0001,
            io_cost_per_page: 10.0,
            pattern_match_multiplier: 10.0,
            index_type_multipliers,
        }
    }
}

impl GraphCostFactors {
    /// Get cost multiplier for index type
    pub fn get_index_multiplier(&self, index_type: &GraphIndexType) -> f64 {
        self.index_type_multipliers
            .get(index_type)
            .copied()
            .unwrap_or(1.0)
    }
}

/// GraphCostModel - cost estimation for graph operations
#[derive(Debug, Clone)]
pub struct GraphCostModel {
    factors: GraphCostFactors,
}

impl GraphCostModel {
    /// Create a new GraphCostModel with given factors
    pub fn new(factors: GraphCostFactors) -> Self {
        Self { factors }
    }

    /// Create a GraphCostModel with default factors
    pub fn default_model() -> Self {
        Self {
            factors: GraphCostFactors::default(),
        }
    }

    /// Estimate cost for BFS/DFS traversal
    ///
    /// Parameters:
    /// - max_depth: maximum traversal depth
    /// - avg_degree: average node degree
    /// - index_type: type of graph index
    pub fn traversal_cost(
        &self,
        max_depth: usize,
        avg_degree: f64,
        index_type: &GraphIndexType,
    ) -> f64 {
        // BFS visits approximately: n * (avg_degree ^ depth)
        // We estimate based on depth and assume a bounded search space
        let visited_nodes = (avg_degree.powi(max_depth as i32)).min(1_000_000.0) as u64;
        let visited_edges = visited_nodes * (avg_degree as u64);

        let node_cost = visited_nodes as f64 * self.factors.cpu_cost_per_node;
        let edge_cost = visited_edges as f64 * self.factors.cpu_cost_per_edge;
        let index_multiplier = self.factors.get_index_multiplier(index_type);

        (node_cost + edge_cost) * index_multiplier
    }

    /// Estimate cost for pattern matching
    ///
    /// Parameters:
    /// - pattern: graph pattern to match
    /// - graph_size: total nodes in graph
    /// - index_type: type of graph index
    pub fn pattern_match_cost(
        &self,
        pattern: &GraphPattern,
        graph_size: u64,
        index_type: &GraphIndexType,
    ) -> f64 {
        // Pattern matching is expensive - exponential in pattern size
        let pattern_complexity = (pattern.node_labels.len() + pattern.edge_labels.len()) as f64;
        let base_cost =
            graph_size as f64 * pattern_complexity * self.factors.pattern_match_multiplier;
        let index_multiplier = self.factors.get_index_multiplier(index_type);
        base_cost * index_multiplier
    }

    /// Estimate cost for reachability query
    ///
    /// Parameters:
    /// - graph_size: total nodes in graph
    /// - index_type: type of graph index
    pub fn reachability_cost(&self, graph_size: u64, index_type: &GraphIndexType) -> f64 {
        // Reachability can use indexes for labeled graphs
        let base_cost = graph_size as f64 * self.factors.cpu_cost_per_node;
        let index_multiplier = self.factors.get_index_multiplier(index_type);
        base_cost * index_multiplier * 0.1 // Index helps significantly
    }

    /// Estimate cost for shortest path query
    ///
    /// Parameters:
    /// - graph_size: total nodes in graph
    /// - avg_degree: average node degree
    /// - index_type: type of graph index
    pub fn shortest_path_cost(
        &self,
        graph_size: u64,
        avg_degree: f64,
        index_type: &GraphIndexType,
    ) -> f64 {
        // Dijkstra's algorithm complexity: O((V + E) log V)
        let estimated_edges = graph_size as f64 * avg_degree;
        let base_cost = (graph_size as f64 + estimated_edges) * avg_degree.log2();
        let index_multiplier = self.factors.get_index_multiplier(index_type);
        base_cost * index_multiplier
    }

    /// Estimate cost for a UnifiedPlan graph node
    pub fn estimate_plan_cost(&self, plan: &UnifiedPlan, graph_size: u64) -> f64 {
        match plan {
            UnifiedPlan::GraphScan { scan_type, .. } => {
                let avg_degree = 10.0; // Assume avg degree - would come from stats
                match scan_type {
                    GraphScanType::Traversal { max_depth } => {
                        self.traversal_cost(*max_depth, avg_degree, &GraphIndexType::AdjacencyList)
                    }
                    GraphScanType::PatternMatch { pattern } => {
                        self.pattern_match_cost(pattern, graph_size, &GraphIndexType::LabeledIndex)
                    }
                    GraphScanType::Reachability { .. } => {
                        self.reachability_cost(graph_size, &GraphIndexType::LabeledIndex)
                    }
                    GraphScanType::ShortestPath { .. } => self.shortest_path_cost(
                        graph_size,
                        avg_degree,
                        &GraphIndexType::AdjacencyList,
                    ),
                }
            }
            UnifiedPlan::HybridGraphScan {
                sql_filter: Some(_),
                scan_type,
                ..
            } => {
                // Hybrid has extra cost due to SQL pre-filtering
                let avg_degree = 10.0;
                let base_cost = match scan_type {
                    GraphScanType::Traversal { max_depth } => {
                        self.traversal_cost(*max_depth, avg_degree, &GraphIndexType::AdjacencyList)
                    }
                    GraphScanType::PatternMatch { pattern } => {
                        self.pattern_match_cost(pattern, graph_size, &GraphIndexType::LabeledIndex)
                    }
                    GraphScanType::Reachability { .. } => {
                        self.reachability_cost(graph_size, &GraphIndexType::LabeledIndex)
                    }
                    GraphScanType::ShortestPath { .. } => self.shortest_path_cost(
                        graph_size,
                        avg_degree,
                        &GraphIndexType::AdjacencyList,
                    ),
                };
                base_cost * 1.2 // 20% overhead for hybrid
            }
            _ => 0.0,
        }
    }

    /// Compare graph scan cost vs SQL table scan cost
    /// Returns true if graph scan is cheaper
    pub fn graph_scan_cheaper_than_sql(
        &self,
        graph_cardinality: u64,
        sql_cardinality: u64,
        query_type: &GraphScanType,
    ) -> bool {
        let avg_degree = 10.0;
        let graph_cost = match query_type {
            GraphScanType::Traversal { max_depth } => {
                self.traversal_cost(*max_depth, avg_degree, &GraphIndexType::AdjacencyList)
            }
            GraphScanType::Reachability { .. } => {
                self.reachability_cost(graph_cardinality, &GraphIndexType::LabeledIndex)
            }
            _ => graph_cardinality as f64 * 0.01,
        };
        let sql_cost = sql_cardinality as f64 * 0.01;
        graph_cost < sql_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_cost_factors_default() {
        let factors = GraphCostFactors::default();
        assert_eq!(factors.cpu_cost_per_node, 0.01);
        assert!(
            factors.get_index_multiplier(&GraphIndexType::LabeledIndex)
                < factors.get_index_multiplier(&GraphIndexType::AdjacencyList)
        );
    }

    #[test]
    fn test_traversal_cost() {
        let model = GraphCostModel::default_model();
        let cost = model.traversal_cost(3, 10.0, &GraphIndexType::AdjacencyList);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_pattern_match_cost() {
        let model = GraphCostModel::default_model();
        let pattern = GraphPattern {
            node_labels: vec!["User".to_string(), "Product".to_string()],
            edge_labels: vec!["BUYS".to_string()],
            path_pattern: "(User)-[BUYS]->(Product)".to_string(),
        };
        let cost = model.pattern_match_cost(&pattern, 10000, &GraphIndexType::LabeledIndex);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_reachability_cost() {
        let model = GraphCostModel::default_model();
        let cost = model.reachability_cost(10000, &GraphIndexType::LabeledIndex);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_shortest_path_cost() {
        let model = GraphCostModel::default_model();
        let cost = model.shortest_path_cost(10000, 10.0, &GraphIndexType::AdjacencyList);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_plan_cost() {
        let model = GraphCostModel::default_model();
        let plan = UnifiedPlan::GraphScan {
            graph_name: "social_graph".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: Some("user_123".to_string()),
        };
        let cost = model.estimate_plan_cost(&plan, 10000);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_graph_scan_cheaper_than_sql_traversal() {
        let model = GraphCostModel::default_model();
        let scan_type = GraphScanType::Traversal { max_depth: 2 };
        let cheaper = model.graph_scan_cheaper_than_sql(100, 10000, &scan_type);
        assert!(cheaper);
    }

    #[test]
    fn test_graph_index_type_default() {
        let index_type = GraphIndexType::default();
        assert!(matches!(index_type, GraphIndexType::AdjacencyList));
    }
}
