//! Path Selector Module
//!
//! Automatically selects the best execution path based on cost estimation.

use crate::unified_cost::{ExecutionPath, UnifiedCostModel};
use crate::unified_plan::UnifiedPlan;
use std::collections::HashMap;
use std::fmt::Debug;

/// PathSelector configuration
#[derive(Debug, Clone)]
pub struct PathSelectorConfig {
    /// Minimum cardinality to consider vector scan
    pub vector_scan_threshold: u64,
    /// Minimum cardinality to consider graph scan
    pub graph_scan_threshold: u64,
    /// Cost ratio threshold for hybrid vs pure (e.g., 1.2 = hybrid must be 20% cheaper)
    pub hybrid_threshold: f64,
    /// Maximum depth for graph traversal
    pub max_graph_depth: usize,
    /// Default vector dimension
    pub default_vector_dim: u32,
    /// Default graph size
    pub default_graph_size: u64,
}

impl Default for PathSelectorConfig {
    fn default() -> Self {
        Self {
            vector_scan_threshold: 1000,
            graph_scan_threshold: 100,
            hybrid_threshold: 1.2,
            max_graph_depth: 10,
            default_vector_dim: 128,
            default_graph_size: 10000,
        }
    }
}

/// PathSelector - automatically selects the best execution path
#[derive(Debug, Clone)]
pub struct PathSelector {
    config: PathSelectorConfig,
    unified_cost_model: UnifiedCostModel,
}

impl PathSelector {
    /// Create a new PathSelector with config and cost model
    pub fn new(config: PathSelectorConfig, unified_cost_model: UnifiedCostModel) -> Self {
        Self {
            config,
            unified_cost_model,
        }
    }

    /// Create a PathSelector with default settings
    pub fn with_defaults() -> Self {
        Self::new(
            PathSelectorConfig::default(),
            UnifiedCostModel::default_model(128, 10000),
        )
    }

    /// Analyze a plan and return the recommended execution path
    pub fn analyze(&self, plan: &UnifiedPlan) -> PathSelection {
        let sql_cost = self.estimate_sql_cost(plan);
        let vector_cost = self.estimate_vector_cost(plan);
        let graph_cost = self.estimate_graph_cost(plan);

        // Calculate hybrid costs if applicable
        let hybrid_sql_vector = self.estimate_hybrid_sql_vector_cost(plan);
        let hybrid_sql_graph = self.estimate_hybrid_sql_graph_cost(plan);

        let (best_path, best_cost) = self.unified_cost_model.select_best_path(
            sql_cost,
            vector_cost,
            graph_cost,
            hybrid_sql_vector,
            hybrid_sql_graph,
        );

        PathSelection {
            recommended_path: best_path,
            estimated_cost: best_cost,
            sql_cost,
            vector_cost,
            graph_cost,
            hybrid_sql_vector_cost: hybrid_sql_vector,
            hybrid_sql_graph_cost: hybrid_sql_graph,
            reasoning: self.generate_reasoning(plan, best_path, sql_cost, vector_cost, graph_cost),
        }
    }

    fn estimate_sql_cost(&self, plan: &UnifiedPlan) -> f64 {
        // Extract pure SQL portion and estimate
        let sql_plan = self.extract_sql_plan(plan);
        // If the plan is pure vector/graph (no SQL components), return MAX cost
        if matches!(sql_plan, UnifiedPlan::EmptyRelation) && plan.is_vector_op() {
            return f64::MAX;
        }
        if matches!(sql_plan, UnifiedPlan::EmptyRelation) && plan.is_graph_op() {
            return f64::MAX;
        }
        if matches!(sql_plan, UnifiedPlan::EmptyRelation) {
            // Plan is empty or pure something else
            return f64::MAX;
        }
        self.unified_cost_model.estimate_cost(&sql_plan)
    }

    fn estimate_vector_cost(&self, plan: &UnifiedPlan) -> f64 {
        if plan.is_vector_op() {
            self.unified_cost_model.estimate_cost(plan)
        } else {
            f64::MAX // Not a vector operation
        }
    }

    fn estimate_graph_cost(&self, plan: &UnifiedPlan) -> f64 {
        if plan.is_graph_op() {
            self.unified_cost_model.estimate_cost(plan)
        } else {
            f64::MAX // Not a graph operation
        }
    }

    fn estimate_hybrid_sql_vector_cost(&self, plan: &UnifiedPlan) -> Option<f64> {
        if plan.is_vector_op() && !plan.is_sql_only() {
            // Plan has both SQL and vector components
            Some(self.unified_cost_model.estimate_cost(plan))
        } else if plan.is_vector_op() {
            // Pure vector, estimate SQL alternative
            Some(self.estimate_sql_cost(plan) * self.config.hybrid_threshold)
        } else {
            None
        }
    }

    fn estimate_hybrid_sql_graph_cost(&self, plan: &UnifiedPlan) -> Option<f64> {
        if plan.is_graph_op() && !plan.is_sql_only() {
            Some(self.unified_cost_model.estimate_cost(plan))
        } else if plan.is_graph_op() {
            Some(self.estimate_graph_cost(plan) * self.config.hybrid_threshold)
        } else {
            None
        }
    }

    /// Extract pure SQL portion from a unified plan
    fn extract_sql_plan(&self, plan: &UnifiedPlan) -> UnifiedPlan {
        match plan {
            UnifiedPlan::SqlVectorJoin { sql_plan, .. } => *sql_plan.clone(),
            UnifiedPlan::SqlGraphJoin { sql_plan, .. } => *sql_plan.clone(),
            UnifiedPlan::VectorGraphJoin { .. } => UnifiedPlan::EmptyRelation,
            UnifiedPlan::HybridVectorScan { sql_filter, .. } => {
                if sql_filter.is_some() {
                    UnifiedPlan::Filter {
                        predicate: sql_filter.clone().unwrap(),
                        input: Box::new(UnifiedPlan::EmptyRelation),
                    }
                } else {
                    UnifiedPlan::EmptyRelation
                }
            }
            UnifiedPlan::HybridGraphScan { sql_filter, .. } => {
                if sql_filter.is_some() {
                    UnifiedPlan::Filter {
                        predicate: sql_filter.clone().unwrap(),
                        input: Box::new(UnifiedPlan::EmptyRelation),
                    }
                } else {
                    UnifiedPlan::EmptyRelation
                }
            }
            // For pure vector/graph plans without SQL components, return EmptyRelation
            _ if plan.is_vector_op() || plan.is_graph_op() => UnifiedPlan::EmptyRelation,
            // For pure SQL plans, return the plan itself
            _ => plan.clone(),
        }
    }

    fn generate_reasoning(
        &self,
        plan: &UnifiedPlan,
        path: ExecutionPath,
        sql_cost: f64,
        vector_cost: f64,
        graph_cost: f64,
    ) -> String {
        let cardinality = plan.estimate_cardinality();

        match path {
            ExecutionPath::Sql => {
                format!(
                    "SQL path selected. Cost: {:.2}. Card: {}. Vector/Graph costs too high.",
                    sql_cost, cardinality
                )
            }
            ExecutionPath::Vector => {
                format!(
                    "Vector path selected. Cost: {:.2} vs SQL: {:.2}. {} cardinality.",
                    vector_cost,
                    sql_cost,
                    if cardinality < self.config.vector_scan_threshold {
                        "Low"
                    } else {
                        "High"
                    }
                )
            }
            ExecutionPath::Graph => {
                format!(
                    "Graph path selected. Cost: {:.2} vs SQL: {:.2}.",
                    graph_cost, sql_cost
                )
            }
            ExecutionPath::HybridSqlVector => {
                format!("Hybrid SQL+Vector path selected. Best for mixed workloads.")
            }
            ExecutionPath::HybridSqlGraph => {
                format!("Hybrid SQL+Graph path selected. Best for graph-enriched SQL queries.")
            }
            ExecutionPath::Unified => {
                format!("Unified path selected. All three domains involved in query.")
            }
            _ => format!("Path: {:?}", path),
        }
    }
}

/// Result of path selection analysis
#[derive(Debug, Clone)]
pub struct PathSelection {
    /// Recommended execution path
    pub recommended_path: ExecutionPath,
    /// Estimated cost for recommended path
    pub estimated_cost: f64,
    /// Cost if using pure SQL
    pub sql_cost: f64,
    /// Cost if using pure Vector
    pub vector_cost: f64,
    /// Cost if using pure Graph
    pub graph_cost: f64,
    /// Cost if using hybrid SQL+Vector
    pub hybrid_sql_vector_cost: Option<f64>,
    /// Cost if using hybrid SQL+Graph
    pub hybrid_sql_graph_cost: Option<f64>,
    /// Human-readable reasoning for selection
    pub reasoning: String,
}

impl PathSelection {
    /// Check if vector path is recommended
    pub fn should_use_vector(&self) -> bool {
        self.recommended_path.involves_vector()
    }

    /// Check if graph path is recommended
    pub fn should_use_graph(&self) -> bool {
        self.recommended_path.involves_graph()
    }

    /// Check if hybrid execution is recommended
    pub fn should_use_hybrid(&self) -> bool {
        self.recommended_path.is_hybrid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_plan::{GraphScanType, VectorScanType};

    #[test]
    fn test_path_selector_config_default() {
        let config = PathSelectorConfig::default();
        assert_eq!(config.vector_scan_threshold, 1000);
        assert_eq!(config.max_graph_depth, 10);
    }

    #[test]
    fn test_path_selector_with_defaults() {
        let selector = PathSelector::with_defaults();
        let plan = UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        let selection = selector.analyze(&plan);
        assert_eq!(selection.recommended_path, ExecutionPath::Sql);
    }

    #[test]
    fn test_analyze_vector_scan() {
        let selector = PathSelector::with_defaults();
        let plan = UnifiedPlan::VectorScan {
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Knn { k: 10 },
            limit: Some(10),
        };
        let selection = selector.analyze(&plan);
        assert!(selection.should_use_vector());
    }

    #[test]
    fn test_analyze_graph_scan() {
        let selector = PathSelector::with_defaults();
        let plan = UnifiedPlan::GraphScan {
            graph_name: "social_graph".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: Some("user_123".to_string()),
        };
        let selection = selector.analyze(&plan);
        assert!(selection.should_use_graph());
    }

    #[test]
    fn test_path_selection_should_use_hybrid() {
        let selection = PathSelection {
            recommended_path: ExecutionPath::HybridSqlVector,
            estimated_cost: 100.0,
            sql_cost: 150.0,
            vector_cost: 120.0,
            graph_cost: f64::MAX,
            hybrid_sql_vector_cost: Some(100.0),
            hybrid_sql_graph_cost: None,
            reasoning: "Hybrid selected".to_string(),
        };
        assert!(selection.should_use_hybrid());
        assert!(selection.should_use_vector());
        assert!(!selection.should_use_graph());
    }

    #[test]
    fn test_path_selection_reasoning() {
        let selection = PathSelection {
            recommended_path: ExecutionPath::Sql,
            estimated_cost: 100.0,
            sql_cost: 100.0,
            vector_cost: 500.0,
            graph_cost: 200.0,
            hybrid_sql_vector_cost: None,
            hybrid_sql_graph_cost: None,
            reasoning: "SQL path selected".to_string(),
        };
        assert!(selection.reasoning.contains("SQL path selected"));
    }
}
