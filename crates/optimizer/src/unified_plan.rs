//! Unified Plan Types Module
//!
//! Extends the basic Plan enum with VectorScan and GraphScan for unified query planning.

use crate::plan::AsAny;
use crate::rules::{Expr, JoinType};
use std::any::Any;
use std::fmt::Debug;

/// Vector scan operation types
#[derive(Debug, Clone, PartialEq)]
pub enum VectorScanType {
    /// Exact nearest neighbor search (KNN)
    Knn { k: usize },
    /// Approximate nearest neighbor (ANN) search
    Ann { threshold: f32 },
    /// Similarity search with threshold
    Similarity { threshold: f32 },
    /// Range search in vector space
    Range { radius: f32 },
}

/// Graph scan operation types
#[derive(Debug, Clone, PartialEq)]
pub enum GraphScanType {
    /// Graph traversal (BFS/DFS)
    Traversal { max_depth: usize },
    /// Pattern matching (subgraph isomorphism)
    PatternMatch { pattern: GraphPattern },
    /// Reachability query
    Reachability { target: String },
    /// Shortest path query
    ShortestPath { target: String },
}

/// Graph pattern for matching
#[derive(Debug, Clone, PartialEq)]
pub struct GraphPattern {
    pub node_labels: Vec<String>,
    pub edge_labels: Vec<String>,
    pub path_pattern: String,
}

impl Default for GraphPattern {
    fn default() -> Self {
        Self {
            node_labels: Vec::new(),
            edge_labels: Vec::new(),
            path_pattern: String::new(),
        }
    }
}

/// Unified plan node types - extends SQL plan with vector and graph operations
#[derive(Debug, Clone, PartialEq)]
pub enum UnifiedPlan {
    // SQL Plan variants (from existing Plan)
    /// Table scan operation
    TableScan {
        table_name: String,
        projection: Option<Vec<usize>>,
    },
    /// Index scan operation
    IndexScan {
        table_name: String,
        index_name: String,
        predicate: Option<Expr>,
    },
    /// Filter operation (WHERE clause)
    Filter {
        predicate: Expr,
        input: Box<UnifiedPlan>,
    },
    /// Projection operation (SELECT columns)
    Projection {
        expr: Vec<Expr>,
        input: Box<UnifiedPlan>,
    },
    /// Join operation
    Join {
        left: Box<UnifiedPlan>,
        right: Box<UnifiedPlan>,
        join_type: JoinType,
        condition: Option<Expr>,
    },
    /// Aggregate operation (GROUP BY)
    Aggregate {
        group_by: Vec<Expr>,
        aggregates: Vec<Expr>,
        input: Box<UnifiedPlan>,
    },
    /// Sort operation (ORDER BY)
    Sort {
        expr: Vec<Expr>,
        input: Box<UnifiedPlan>,
    },
    /// Limit operation
    Limit {
        limit: usize,
        input: Box<UnifiedPlan>,
    },

    // Vector Plan variants
    /// Vector index scan (ANN/KNN)
    VectorScan {
        vector_index: String,
        query_vector: Vec<f32>,
        scan_type: VectorScanType,
        limit: Option<usize>,
    },
    /// Hybrid scan combining SQL filter with vector search
    HybridVectorScan {
        sql_filter: Option<Expr>,
        vector_index: String,
        query_vector: Vec<f32>,
        scan_type: VectorScanType,
        limit: Option<usize>,
    },

    // Graph Plan variants
    /// Graph traversal
    GraphScan {
        graph_name: String,
        scan_type: GraphScanType,
        start_node: Option<String>,
    },
    /// Hybrid scan combining SQL filter with graph traversal
    HybridGraphScan {
        sql_filter: Option<Expr>,
        graph_name: String,
        scan_type: GraphScanType,
        start_node: Option<String>,
    },

    // Composite variants
    /// Cross-domain join between SQL and Vector results
    SqlVectorJoin {
        sql_plan: Box<UnifiedPlan>,
        vector_plan: Box<UnifiedPlan>,
        join_condition: Expr,
    },
    /// Cross-domain join between SQL and Graph results
    SqlGraphJoin {
        sql_plan: Box<UnifiedPlan>,
        graph_plan: Box<UnifiedPlan>,
        join_condition: Expr,
    },
    /// Cross-domain join between Vector and Graph results
    VectorGraphJoin {
        vector_plan: Box<UnifiedPlan>,
        graph_plan: Box<UnifiedPlan>,
        join_condition: Expr,
    },

    /// Empty relation
    EmptyRelation,
}

impl UnifiedPlan {
    /// Get the type name of this plan node
    pub fn type_name(&self) -> &'static str {
        match self {
            UnifiedPlan::TableScan { .. } => "TableScan",
            UnifiedPlan::IndexScan { .. } => "IndexScan",
            UnifiedPlan::Filter { .. } => "Filter",
            UnifiedPlan::Projection { .. } => "Projection",
            UnifiedPlan::Join { .. } => "Join",
            UnifiedPlan::Aggregate { .. } => "Aggregate",
            UnifiedPlan::Sort { .. } => "Sort",
            UnifiedPlan::Limit { .. } => "Limit",
            UnifiedPlan::VectorScan { .. } => "VectorScan",
            UnifiedPlan::HybridVectorScan { .. } => "HybridVectorScan",
            UnifiedPlan::GraphScan { .. } => "GraphScan",
            UnifiedPlan::HybridGraphScan { .. } => "HybridGraphScan",
            UnifiedPlan::SqlVectorJoin { .. } => "SqlVectorJoin",
            UnifiedPlan::SqlGraphJoin { .. } => "SqlGraphJoin",
            UnifiedPlan::VectorGraphJoin { .. } => "VectorGraphJoin",
            UnifiedPlan::EmptyRelation => "EmptyRelation",
        }
    }

    /// Check if this is a vector operation
    pub fn is_vector_op(&self) -> bool {
        matches!(
            self,
            UnifiedPlan::VectorScan { .. }
                | UnifiedPlan::HybridVectorScan { .. }
                | UnifiedPlan::SqlVectorJoin { .. }
                | UnifiedPlan::VectorGraphJoin { .. }
        )
    }

    /// Check if this is a graph operation
    pub fn is_graph_op(&self) -> bool {
        matches!(
            self,
            UnifiedPlan::GraphScan { .. }
                | UnifiedPlan::HybridGraphScan { .. }
                | UnifiedPlan::SqlGraphJoin { .. }
                | UnifiedPlan::VectorGraphJoin { .. }
        )
    }

    /// Check if this is a pure SQL operation (no vector/graph)
    pub fn is_sql_only(&self) -> bool {
        !self.is_vector_op() && !self.is_graph_op()
    }

    /// Estimate result cardinality for this plan
    pub fn estimate_cardinality(&self) -> u64 {
        match self {
            UnifiedPlan::TableScan { .. } => 1000,
            UnifiedPlan::IndexScan { .. } => 100,
            UnifiedPlan::Filter { input, .. } => input.estimate_cardinality() / 2,
            UnifiedPlan::Projection { input, .. } => input.estimate_cardinality(),
            UnifiedPlan::Join { left, right, .. } => {
                (left.estimate_cardinality() * right.estimate_cardinality()) / 10
            }
            UnifiedPlan::Aggregate { input, .. } => input.estimate_cardinality() / 10,
            UnifiedPlan::Sort { input, .. } => input.estimate_cardinality(),
            UnifiedPlan::Limit { limit, input } => {
                (*limit as u64).min(input.estimate_cardinality())
            }
            UnifiedPlan::VectorScan { limit, .. } => limit.unwrap_or(100) as u64,
            UnifiedPlan::HybridVectorScan { limit, .. } => limit.unwrap_or(100) as u64,
            UnifiedPlan::GraphScan { scan_type, .. } => match scan_type {
                GraphScanType::Traversal { max_depth } => (1000 * (*max_depth as u64)).min(100000),
                GraphScanType::PatternMatch { .. } => 100,
                GraphScanType::Reachability { .. } => 1,
                GraphScanType::ShortestPath { .. } => 1,
            },
            UnifiedPlan::HybridGraphScan { .. } => 100,
            UnifiedPlan::SqlVectorJoin {
                sql_plan,
                vector_plan,
                ..
            } => (sql_plan.estimate_cardinality() * vector_plan.estimate_cardinality()) / 10,
            UnifiedPlan::SqlGraphJoin {
                sql_plan,
                graph_plan,
                ..
            } => (sql_plan.estimate_cardinality() * graph_plan.estimate_cardinality()) / 10,
            UnifiedPlan::VectorGraphJoin {
                vector_plan,
                graph_plan,
                ..
            } => (vector_plan.estimate_cardinality() * graph_plan.estimate_cardinality()) / 10,
            UnifiedPlan::EmptyRelation => 0,
        }
    }
}

impl AsAny for UnifiedPlan {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_plan_type_name() {
        let plan = UnifiedPlan::EmptyRelation;
        assert_eq!(plan.type_name(), "EmptyRelation");

        let plan = UnifiedPlan::VectorScan {
            vector_index: " embeddings_idx".to_string(),
            query_vector: vec![0.1, 0.2, 0.3],
            scan_type: VectorScanType::Knn { k: 10 },
            limit: Some(10),
        };
        assert_eq!(plan.type_name(), "VectorScan");
    }

    #[test]
    fn test_is_vector_op() {
        let plan = UnifiedPlan::VectorScan {
            vector_index: " embeddings_idx".to_string(),
            query_vector: vec![0.1, 0.2, 0.3],
            scan_type: VectorScanType::Knn { k: 10 },
            limit: Some(10),
        };
        assert!(plan.is_vector_op());
        assert!(!plan.is_graph_op());
        assert!(!plan.is_sql_only());
    }

    #[test]
    fn test_is_graph_op() {
        let plan = UnifiedPlan::GraphScan {
            graph_name: "social_graph".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: Some("user_123".to_string()),
        };
        assert!(plan.is_graph_op());
        assert!(!plan.is_vector_op());
        assert!(!plan.is_sql_only());
    }

    #[test]
    fn test_is_sql_only() {
        let plan = UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        assert!(plan.is_sql_only());
        assert!(!plan.is_vector_op());
        assert!(!plan.is_graph_op());
    }

    #[test]
    fn test_estimate_cardinality() {
        let plan = UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        assert_eq!(plan.estimate_cardinality(), 1000);

        let plan = UnifiedPlan::Limit {
            limit: 10,
            input: Box::new(plan),
        };
        assert_eq!(plan.estimate_cardinality(), 10);
    }

    #[test]
    fn test_knn_scan_type() {
        let scan_type = VectorScanType::Knn { k: 10 };
        assert!(matches!(scan_type, VectorScanType::Knn { k: 10 }));
    }

    #[test]
    fn test_graph_pattern_default() {
        let pattern = GraphPattern::default();
        assert!(pattern.node_labels.is_empty());
        assert!(pattern.edge_labels.is_empty());
    }

    #[test]
    fn test_hybrid_vector_scan() {
        let plan = UnifiedPlan::HybridVectorScan {
            sql_filter: None,
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Ann { threshold: 0.8 },
            limit: Some(50),
        };
        assert!(plan.is_vector_op());
        assert!(!plan.is_sql_only());
    }

    #[test]
    fn test_sql_vector_join() {
        let sql_plan = Box::new(UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        });
        let vector_plan = Box::new(UnifiedPlan::VectorScan {
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Knn { k: 5 },
            limit: Some(5),
        });
        let plan = UnifiedPlan::SqlVectorJoin {
            sql_plan,
            vector_plan,
            join_condition: Expr::Column("user_id".to_string()),
        };
        assert!(plan.is_vector_op());
        assert!(!plan.is_sql_only());
    }
}
