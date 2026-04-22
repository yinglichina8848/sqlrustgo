//! Unified Cost Model Module
//!
//! Combines SQL, Vector, and Graph cost models for unified cost estimation.

use crate::cost::SimpleCostModel;
use crate::graph_cost::GraphCostModel;
use crate::rules::JoinType;
use crate::unified_plan::UnifiedPlan;
use crate::vector_cost::VectorCostModel;

/// Execution path types for cost comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPath {
    /// Pure SQL execution
    Sql,
    /// Pure Vector execution
    Vector,
    /// Pure Graph execution
    Graph,
    /// Hybrid SQL + Vector execution
    HybridSqlVector,
    /// Hybrid SQL + Graph execution
    HybridSqlGraph,
    /// Hybrid Vector + Graph execution
    HybridVectorGraph,
    /// All three combined
    Unified,
}

impl ExecutionPath {
    /// Check if this path involves hybrid execution
    pub fn is_hybrid(&self) -> bool {
        matches!(
            self,
            ExecutionPath::HybridSqlVector
                | ExecutionPath::HybridSqlGraph
                | ExecutionPath::HybridVectorGraph
                | ExecutionPath::Unified
        )
    }

    /// Check if this path involves vector operations
    pub fn involves_vector(&self) -> bool {
        matches!(
            self,
            ExecutionPath::Vector
                | ExecutionPath::HybridSqlVector
                | ExecutionPath::HybridVectorGraph
                | ExecutionPath::Unified
        )
    }

    /// Check if this path involves graph operations
    pub fn involves_graph(&self) -> bool {
        matches!(
            self,
            ExecutionPath::Graph
                | ExecutionPath::HybridSqlGraph
                | ExecutionPath::HybridVectorGraph
                | ExecutionPath::Unified
        )
    }
}

/// UnifiedCostModel - combines all cost models for cross-domain optimization
#[derive(Debug, Clone)]
pub struct UnifiedCostModel {
    /// SQL cost model
    sql_cost_model: SimpleCostModel,
    /// Vector cost model
    vector_cost_model: VectorCostModel,
    /// Graph cost model
    graph_cost_model: GraphCostModel,
    /// Vector dimension for cost estimation
    vector_dimension: u32,
    /// Graph size for cost estimation
    graph_size: u64,
    /// Table statistics for cost estimation (table_name -> (row_count, page_count))
    table_stats: std::collections::HashMap<String, (u64, u64)>,
}

impl UnifiedCostModel {
    /// Create a new UnifiedCostModel with all component models
    pub fn new(
        sql_cost_model: SimpleCostModel,
        vector_cost_model: VectorCostModel,
        graph_cost_model: GraphCostModel,
        vector_dimension: u32,
        graph_size: u64,
    ) -> Self {
        Self {
            sql_cost_model,
            vector_cost_model,
            graph_cost_model,
            vector_dimension,
            graph_size,
            table_stats: std::collections::HashMap::new(),
        }
    }

    /// Create a UnifiedCostModel with default settings
    pub fn default_model(vector_dimension: u32, graph_size: u64) -> Self {
        Self {
            sql_cost_model: SimpleCostModel::default_model(),
            vector_cost_model: VectorCostModel::default_model(),
            graph_cost_model: GraphCostModel::default_model(),
            vector_dimension,
            graph_size,
            table_stats: std::collections::HashMap::new(),
        }
    }

    /// Update table statistics for cost estimation
    /// This allows the cost model to use actual statistics instead of defaults
    pub fn update_table_stats(&mut self, table_name: String, row_count: u64, page_count: u64) {
        self.table_stats.insert(table_name, (row_count, page_count));
    }

    /// Get the default row count for a table if no stats available
    fn get_row_count(&self, table_name: &str) -> u64 {
        self.table_stats
            .get(table_name)
            .map(|(rows, _)| *rows)
            .unwrap_or(1000) // Default estimate
    }

    /// Get the default page count for a table if no stats available
    fn get_page_count(&self, table_name: &str) -> u64 {
        self.table_stats
            .get(table_name)
            .map(|(_, pages)| *pages)
            .unwrap_or(10) // Default estimate
    }

    /// Estimate cost for any UnifiedPlan
    pub fn estimate_cost(&self, plan: &UnifiedPlan) -> f64 {
        match plan {
            // Pure SQL operations - delegate to SimpleCostModel
            UnifiedPlan::TableScan {
                table_name,
                projection: _,
            } => {
                // Use actual statistics if available, otherwise use defaults
                let row_count = self.get_row_count(table_name);
                let page_count = self.get_page_count(table_name);
                self.sql_cost_model.seq_scan_cost(row_count, page_count)
            }
            UnifiedPlan::IndexScan { .. } => self.sql_cost_model.index_scan_cost(100, 1, 10),
            UnifiedPlan::Filter { input, .. } => {
                self.estimate_cost(input) * 1.1 // 10% overhead
            }
            UnifiedPlan::Projection { input, .. } => self.estimate_cost(input),
            UnifiedPlan::Join {
                left,
                right,
                join_type,
                ..
            } => {
                let left_rows = left.estimate_cardinality();
                let right_rows = right.estimate_cardinality();
                let join_method = match join_type {
                    JoinType::Inner => "hash_join",
                    _ => "nested_loop",
                };
                self.sql_cost_model
                    .join_cost(left_rows, right_rows, join_method)
            }
            UnifiedPlan::Aggregate {
                input, group_by, ..
            } => {
                let row_count = input.estimate_cardinality();
                self.sql_cost_model
                    .agg_cost(row_count, group_by.len() as u32)
            }
            UnifiedPlan::Sort { input, .. } => {
                let row_count = input.estimate_cardinality();
                self.sql_cost_model.sort_cost(row_count, 100)
            }
            UnifiedPlan::Limit { limit, input } => {
                self.estimate_cost(input).min(*limit as f64 * 0.1)
            }

            // Vector operations - delegate to VectorCostModel
            UnifiedPlan::VectorScan { .. } | UnifiedPlan::HybridVectorScan { .. } => self
                .vector_cost_model
                .estimate_plan_cost(plan, self.vector_dimension),

            // Graph operations - delegate to GraphCostModel
            UnifiedPlan::GraphScan { .. } | UnifiedPlan::HybridGraphScan { .. } => self
                .graph_cost_model
                .estimate_plan_cost(plan, self.graph_size),

            // Cross-domain joins - combine costs
            UnifiedPlan::SqlVectorJoin {
                sql_plan,
                vector_plan,
                ..
            } => self.estimate_cost(sql_plan) + self.estimate_cost(vector_plan) * 1.2,
            UnifiedPlan::SqlGraphJoin {
                sql_plan,
                graph_plan,
                ..
            } => self.estimate_cost(sql_plan) + self.estimate_cost(graph_plan) * 1.2,
            UnifiedPlan::VectorGraphJoin {
                vector_plan,
                graph_plan,
                ..
            } => self.estimate_cost(vector_plan) + self.estimate_cost(graph_plan) * 1.2,

            UnifiedPlan::EmptyRelation => 0.0,
        }
    }

    /// Select the best execution path for a query
    pub fn select_best_path(
        &self,
        sql_cost: f64,
        vector_cost: f64,
        graph_cost: f64,
        hybrid_sql_vector_cost: Option<f64>,
        hybrid_sql_graph_cost: Option<f64>,
    ) -> (ExecutionPath, f64) {
        let mut candidates = vec![
            (ExecutionPath::Sql, sql_cost),
            (ExecutionPath::Vector, vector_cost),
            (ExecutionPath::Graph, graph_cost),
        ];

        if let Some(cost) = hybrid_sql_vector_cost {
            candidates.push((ExecutionPath::HybridSqlVector, cost));
        }
        if let Some(cost) = hybrid_sql_graph_cost {
            candidates.push((ExecutionPath::HybridSqlGraph, cost));
        }

        // Also consider full unified if we have hybrid costs
        if let (Some(hsq), Some(hsg)) = (hybrid_sql_vector_cost, hybrid_sql_graph_cost) {
            candidates.push((ExecutionPath::Unified, sql_cost + hsq + hsg));
        }

        // Filter out invalid paths (f64::MAX) and find minimum
        let valid_candidates: Vec<_> = candidates
            .into_iter()
            .filter(|(_, cost)| cost.is_finite())
            .collect();

        if valid_candidates.is_empty() {
            // Fallback: return SQL if all costs are invalid
            return (ExecutionPath::Sql, sql_cost);
        }

        // Find the minimum cost candidate
        let mut best @ (_, mut best_cost) = valid_candidates[0];
        for candidate in &valid_candidates[1..] {
            if candidate.1 < best_cost {
                best = *candidate;
                best_cost = candidate.1;
            }
        }

        best
    }

    /// Get the SQL cost model for direct access
    pub fn sql_model(&self) -> &SimpleCostModel {
        &self.sql_cost_model
    }

    /// Get the vector cost model for direct access
    pub fn vector_model(&self) -> &VectorCostModel {
        &self.vector_cost_model
    }

    /// Get the graph cost model for direct access
    pub fn graph_model(&self) -> &GraphCostModel {
        &self.graph_cost_model
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Expr;
    use crate::unified_plan::{GraphScanType, VectorScanType};

    #[test]
    fn test_execution_path_is_hybrid() {
        assert!(ExecutionPath::HybridSqlVector.is_hybrid());
        assert!(ExecutionPath::HybridSqlGraph.is_hybrid());
        assert!(ExecutionPath::Unified.is_hybrid());
        assert!(!ExecutionPath::Sql.is_hybrid());
        assert!(!ExecutionPath::Vector.is_hybrid());
    }

    #[test]
    fn test_execution_path_involves() {
        assert!(ExecutionPath::Vector.involves_vector());
        assert!(ExecutionPath::HybridSqlVector.involves_vector());
        assert!(ExecutionPath::Graph.involves_graph());
        assert!(ExecutionPath::HybridSqlGraph.involves_graph());
    }

    #[test]
    fn test_unified_cost_model_default() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let plan = UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_vector_cost() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let plan = UnifiedPlan::VectorScan {
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Knn { k: 10 },
            limit: Some(10),
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_graph_cost() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let plan = UnifiedPlan::GraphScan {
            graph_name: "social_graph".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: Some("user_123".to_string()),
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_select_best_path() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let (path, cost) = model.select_best_path(100.0, 50.0, 200.0, Some(80.0), Some(150.0));
        assert_eq!(path, ExecutionPath::Vector);
        assert_eq!(cost, 50.0);
    }

    #[test]
    fn test_select_best_path_sql() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let (path, cost) = model.select_best_path(100.0, 500.0, 200.0, None, None);
        assert_eq!(path, ExecutionPath::Sql);
        assert_eq!(cost, 100.0);
    }

    #[test]
    fn test_estimate_join_cost() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let left = UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        let right = UnifiedPlan::TableScan {
            table_name: "orders".to_string(),
            projection: None,
        };
        let plan = UnifiedPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            join_type: JoinType::Inner,
            condition: None,
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_sql_vector_join_cost() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let sql_plan = Box::new(UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        });
        let vector_plan = Box::new(UnifiedPlan::VectorScan {
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Knn { k: 10 },
            limit: Some(10),
        });
        let plan = UnifiedPlan::SqlVectorJoin {
            sql_plan,
            vector_plan,
            join_condition: crate::rules::Expr::Column("user_id".to_string()),
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_update_table_stats() {
        let mut model = UnifiedCostModel::default_model(128, 10000);
        model.update_table_stats("users".to_string(), 10000, 100);
        assert_eq!(model.get_row_count("users"), 10000);
        assert_eq!(model.get_page_count("users"), 100);
    }

    #[test]
    fn test_cost_with_updated_stats() {
        let mut model = UnifiedCostModel::default_model(128, 10000);

        // Default cost for users table
        let default_cost = {
            let plan = UnifiedPlan::TableScan {
                table_name: "users".to_string(),
                projection: None,
            };
            model.estimate_cost(&plan)
        };

        // Update stats to be much larger
        model.update_table_stats("users".to_string(), 1_000_000, 10000);

        // New cost should be higher
        let new_cost = {
            let plan = UnifiedPlan::TableScan {
                table_name: "users".to_string(),
                projection: None,
            };
            model.estimate_cost(&plan)
        };

        assert!(new_cost > default_cost);
    }

    #[test]
    fn test_cost_model_with_statistics() {
        let mut model = UnifiedCostModel::default_model(128, 10000);

        // Update stats for multiple tables
        model.update_table_stats("orders".to_string(), 50000, 500);
        model.update_table_stats("customers".to_string(), 10000, 100);

        let orders_plan = UnifiedPlan::TableScan {
            table_name: "orders".to_string(),
            projection: None,
        };
        let customers_plan = UnifiedPlan::TableScan {
            table_name: "customers".to_string(),
            projection: None,
        };

        let orders_cost = model.estimate_cost(&orders_plan);
        let customers_cost = model.estimate_cost(&customers_plan);

        // Orders table is larger, so should have higher cost
        assert!(orders_cost > customers_cost);
    }

    #[test]
    fn test_select_best_path_with_hybrid_costs() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let (path, cost) = model.select_best_path(100.0, 50.0, 200.0, Some(80.0), Some(150.0));
        assert_eq!(path, ExecutionPath::Vector);
    }

    #[test]
    fn test_select_best_path_graph_cheapest() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let (path, cost) = model.select_best_path(500.0, 300.0, 50.0, None, None);
        assert_eq!(path, ExecutionPath::Graph);
        assert_eq!(cost, 50.0);
    }

    #[test]
    fn test_select_best_path_hybrid_sql_vector_cheapest() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let (path, cost) = model.select_best_path(100.0, 80.0, 200.0, Some(30.0), None);
        assert_eq!(path, ExecutionPath::HybridSqlVector);
        assert_eq!(cost, 30.0);
    }

    #[test]
    fn test_select_best_path_hybrid_sql_graph_cheapest() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let (path, cost) = model.select_best_path(100.0, 80.0, 200.0, None, Some(25.0));
        assert_eq!(path, ExecutionPath::HybridSqlGraph);
        assert_eq!(cost, 25.0);
    }

    #[test]
    fn test_select_best_path_with_all_hybrid_costs() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let (path, cost) = model.select_best_path(
            100.0,
            500.0,
            200.0,
            Some(30.0),
            Some(25.0),
        );
        assert_eq!(path, ExecutionPath::HybridSqlGraph);
    }

    #[test]
    fn test_select_best_path_all_invalid_fallback() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let (path, cost) = model.select_best_path(
            f64::MAX,
            f64::MAX,
            f64::MAX,
            None,
            None,
        );
        assert_eq!(path, ExecutionPath::Sql);
        assert_eq!(cost, f64::MAX);
    }

    #[test]
    fn test_select_best_path_some_invalid() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let (path, cost) = model.select_best_path(
            100.0,
            f64::MAX,
            50.0,
            None,
            None,
        );
        assert_eq!(path, ExecutionPath::Graph);
        assert_eq!(cost, 50.0);
    }

    #[test]
    fn test_estimate_vector_cost_ann() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let plan = UnifiedPlan::VectorScan {
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Ann { threshold: 0.8 },
            limit: Some(100),
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_vector_cost_similarity() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let plan = UnifiedPlan::VectorScan {
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Similarity { threshold: 0.9 },
            limit: Some(50),
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_vector_cost_range() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let plan = UnifiedPlan::VectorScan {
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Range { radius: 0.5 },
            limit: Some(100),
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_hybrid_vector_scan() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let plan = UnifiedPlan::HybridVectorScan {
            sql_filter: None,
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Knn { k: 10 },
            limit: Some(10),
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost.is_finite());
    }

    #[test]
    fn test_estimate_hybrid_graph_scan() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let plan = UnifiedPlan::HybridGraphScan {
            sql_filter: Some(Expr::Column("active".to_string())),
            graph_name: "social".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: Some("user1".to_string()),
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_sql_graph_join_cost() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let sql_plan = Box::new(UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        });
        let graph_plan = Box::new(UnifiedPlan::GraphScan {
            graph_name: "social".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: None,
        });
        let plan = UnifiedPlan::SqlGraphJoin {
            sql_plan,
            graph_plan,
            join_condition: Expr::Column("user_id".to_string()),
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_vector_graph_join_cost() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let vector_plan = Box::new(UnifiedPlan::VectorScan {
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Knn { k: 5 },
            limit: Some(5),
        });
        let graph_plan = Box::new(UnifiedPlan::GraphScan {
            graph_name: "social".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: None,
        });
        let plan = UnifiedPlan::VectorGraphJoin {
            vector_plan,
            graph_plan,
            join_condition: Expr::Column("entity_id".to_string()),
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_empty_relation_cost() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let plan = UnifiedPlan::EmptyRelation;
        let cost = model.estimate_cost(&plan);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_unified_cost_model_accessors() {
        let model = UnifiedCostModel::default_model(128, 10000);
        let _sql_cost = model.sql_model();
        let _vector_cost = model.vector_model();
        let _graph_cost = model.graph_model();
    }
}
