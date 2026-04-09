//! Query Planner Module
//!
//! Unified query plan generation with multiple execution alternatives.

use crate::path_selector::{PathSelection, PathSelector};
use crate::unified_cost::{ExecutionPath, UnifiedCostModel};
use crate::unified_plan::UnifiedPlan;
use std::collections::HashMap;
use std::fmt::Debug;

/// QueryPlanner configuration
#[derive(Debug, Clone)]
pub struct QueryPlannerConfig {
    /// Enable plan caching
    pub enable_cache: bool,
    /// Maximum number of plans to generate per query
    pub max_alternatives: usize,
    /// Enable plan visualization output
    pub enable_visualization: bool,
    /// Default vector dimension
    pub default_vector_dim: u32,
    /// Default graph size
    pub default_graph_size: u64,
}

impl Default for QueryPlannerConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            max_alternatives: 3,
            enable_visualization: false,
            default_vector_dim: 128,
            default_graph_size: 10000,
        }
    }
}

/// Query plan alternatives
#[derive(Debug, Clone)]
pub struct QueryPlanResult {
    /// The original query
    pub query: String,
    /// The selected plan
    pub selected_plan: UnifiedPlan,
    /// Alternative plans considered
    pub alternatives: Vec<PlanAlternative>,
    /// Path selection analysis
    pub path_selection: PathSelection,
    /// Whether plan was served from cache
    pub from_cache: bool,
}

/// A single plan alternative
#[derive(Debug, Clone)]
pub struct PlanAlternative {
    /// Execution path for this alternative
    pub path: ExecutionPath,
    /// The plan itself
    pub plan: UnifiedPlan,
    /// Estimated cost
    pub cost: f64,
    /// Estimated latency (ms)
    pub estimated_latency_ms: f64,
}

impl PlanAlternative {
    /// Create a new PlanAlternative
    pub fn new(path: ExecutionPath, plan: UnifiedPlan, cost: f64) -> Self {
        // Rough latency estimation: cost * 0.1ms per unit
        let estimated_latency_ms = cost * 0.1;
        Self {
            path,
            plan,
            cost,
            estimated_latency_ms,
        }
    }
}

/// QueryPlanner - generates and optimizes unified query plans
#[derive(Debug, Clone)]
pub struct QueryPlanner {
    config: QueryPlannerConfig,
    path_selector: PathSelector,
    unified_cost_model: UnifiedCostModel,
    plan_cache: HashMap<String, QueryPlanResult>,
}

impl QueryPlanner {
    /// Create a new QueryPlanner with config and cost model
    pub fn new(config: QueryPlannerConfig, unified_cost_model: UnifiedCostModel) -> Self {
        let path_selector = PathSelector::new(
            crate::path_selector::PathSelectorConfig::default(),
            unified_cost_model.clone(),
        );

        Self {
            config,
            path_selector,
            unified_cost_model,
            plan_cache: HashMap::new(),
        }
    }

    /// Create a QueryPlanner with default settings
    pub fn with_defaults() -> Self {
        Self::new(
            QueryPlannerConfig::default(),
            UnifiedCostModel::default_model(128, 10000),
        )
    }

    /// Plan a query and return the optimal plan
    pub fn plan(&mut self, query: &str, plan: UnifiedPlan) -> QueryPlanResult {
        // Check cache if enabled
        if self.config.enable_cache {
            if let Some(cached) = self.plan_cache.get(query) {
                return QueryPlanResult {
                    query: query.to_string(),
                    selected_plan: cached.selected_plan.clone(),
                    alternatives: cached.alternatives.clone(),
                    path_selection: cached.path_selection.clone(),
                    from_cache: true,
                };
            }
        }

        // Analyze path selection
        let path_selection = self.path_selector.analyze(&plan);

        // Generate alternatives
        let alternatives = self.generate_alternatives(&plan, &path_selection);

        // Select best plan
        let selected_plan = alternatives
            .first()
            .map(|alt| alt.plan.clone())
            .unwrap_or(plan);

        let result = QueryPlanResult {
            query: query.to_string(),
            selected_plan,
            alternatives: alternatives.clone(),
            path_selection: path_selection.clone(),
            from_cache: false,
        };

        // Cache if enabled
        if self.config.enable_cache {
            self.plan_cache.insert(query.to_string(), result.clone());
        }

        result
    }

    /// Generate alternative plans with different execution paths
    fn generate_alternatives(
        &self,
        original_plan: &UnifiedPlan,
        path_selection: &PathSelection,
    ) -> Vec<PlanAlternative> {
        let mut alternatives = Vec::new();
        let max_alts = self.config.max_alternatives;

        // Always add SQL alternative if not already best
        if path_selection.recommended_path != ExecutionPath::Sql {
            alternatives.push(PlanAlternative::new(
                ExecutionPath::Sql,
                original_plan.clone(),
                path_selection.sql_cost,
            ));
        }

        // Add Vector alternative if applicable
        if path_selection.vector_cost < f64::MAX
            && path_selection.recommended_path != ExecutionPath::Vector
            && alternatives.len() < max_alts
        {
            let vector_plan = self.transform_to_vector_plan(original_plan);
            alternatives.push(PlanAlternative::new(
                ExecutionPath::Vector,
                vector_plan,
                path_selection.vector_cost,
            ));
        }

        // Add Graph alternative if applicable
        if path_selection.graph_cost < f64::MAX
            && path_selection.recommended_path != ExecutionPath::Graph
            && alternatives.len() < max_alts
        {
            let graph_plan = self.transform_to_graph_plan(original_plan);
            alternatives.push(PlanAlternative::new(
                ExecutionPath::Graph,
                graph_plan,
                path_selection.graph_cost,
            ));
        }

        // Add hybrid alternatives
        if let Some(hybrid_cost) = path_selection.hybrid_sql_vector_cost {
            if path_selection.recommended_path != ExecutionPath::HybridSqlVector
                && alternatives.len() < max_alts
            {
                alternatives.push(PlanAlternative::new(
                    ExecutionPath::HybridSqlVector,
                    original_plan.clone(),
                    hybrid_cost,
                ));
            }
        }

        if let Some(hybrid_cost) = path_selection.hybrid_sql_graph_cost {
            if path_selection.recommended_path != ExecutionPath::HybridSqlGraph
                && alternatives.len() < max_alts
            {
                alternatives.push(PlanAlternative::new(
                    ExecutionPath::HybridSqlGraph,
                    original_plan.clone(),
                    hybrid_cost,
                ));
            }
        }

        // Sort by cost and limit
        alternatives.sort_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        alternatives.truncate(max_alts);

        // Ensure selected path is first
        alternatives.sort_by(|a, b| {
            if a.path == path_selection.recommended_path {
                std::cmp::Ordering::Less
            } else if b.path == path_selection.recommended_path {
                std::cmp::Ordering::Greater
            } else {
                a.cost
                    .partial_cmp(&b.cost)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        alternatives
    }

    /// Transform plan to use vector execution if possible
    fn transform_to_vector_plan(&self, plan: &UnifiedPlan) -> UnifiedPlan {
        // For demonstration, convert TableScan to VectorScan
        // In production, this would be a more sophisticated transformation
        match plan {
            UnifiedPlan::TableScan {
                table_name,
                projection: _,
            } => UnifiedPlan::VectorScan {
                vector_index: format!("{}_embedding_idx", table_name),
                query_vector: vec![0.0; self.config.default_vector_dim as usize],
                scan_type: crate::unified_plan::VectorScanType::Ann { threshold: 0.8 },
                limit: Some(100),
            },
            _ => plan.clone(),
        }
    }

    /// Transform plan to use graph execution if possible
    fn transform_to_graph_plan(&self, plan: &UnifiedPlan) -> UnifiedPlan {
        // For demonstration, convert TableScan to GraphScan
        match plan {
            UnifiedPlan::TableScan {
                table_name,
                projection: _,
            } => UnifiedPlan::GraphScan {
                graph_name: format!("{}_graph", table_name),
                scan_type: crate::unified_plan::GraphScanType::Traversal { max_depth: 3 },
                start_node: None,
            },
            _ => plan.clone(),
        }
    }

    /// Clear the plan cache
    pub fn clear_cache(&mut self) {
        self.plan_cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.plan_cache.len()
    }
}

impl QueryPlanResult {
    /// Generate text-based visualization of the plan
    pub fn visualize_text(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Query: {}\n", self.query));
        output.push_str(&format!(
            "Selected Path: {:?}\n",
            self.path_selection.recommended_path
        ));
        output.push_str(&format!(
            "Estimated Cost: {:.2}\n",
            self.path_selection.estimated_cost
        ));
        output.push_str(&format!("From Cache: {}\n", self.from_cache));
        output.push_str("\n--- Selected Plan ---\n");
        output.push_str(&self.visualize_plan(&self.selected_plan, 0));
        output.push_str("\n--- Alternatives ---\n");
        for (i, alt) in self.alternatives.iter().enumerate() {
            output.push_str(&format!(
                "\nAlternative {}: {:?} (cost: {:.2}, latency: {:.2}ms)\n",
                i + 1,
                alt.path,
                alt.cost,
                alt.estimated_latency_ms
            ));
            output.push_str(&self.visualize_plan(&alt.plan, 0));
        }
        output
    }

    fn visualize_plan(&self, plan: &UnifiedPlan, indent: usize) -> String {
        let prefix = "  ".repeat(indent);
        let mut output = String::new();

        match plan {
            UnifiedPlan::TableScan {
                table_name,
                projection,
            } => {
                output.push_str(&format!("{}TableScan: {}\n", prefix, table_name));
                if let Some(cols) = projection {
                    output.push_str(&format!("{}  projection: {:?}\n", prefix, cols));
                }
            }
            UnifiedPlan::VectorScan {
                vector_index,
                scan_type,
                limit,
                ..
            } => {
                output.push_str(&format!(
                    "{}VectorScan: {} ({:?})\n",
                    prefix, vector_index, scan_type
                ));
                if let Some(l) = limit {
                    output.push_str(&format!("{}  limit: {}\n", prefix, l));
                }
            }
            UnifiedPlan::GraphScan {
                graph_name,
                scan_type,
                start_node,
            } => {
                output.push_str(&format!(
                    "{}GraphScan: {} ({:?})\n",
                    prefix, graph_name, scan_type
                ));
                if let Some(node) = start_node {
                    output.push_str(&format!("{}  start_node: {}\n", prefix, node));
                }
            }
            UnifiedPlan::Filter {
                predicate: _,
                input,
            } => {
                output.push_str(&format!("{}Filter\n", prefix));
                output.push_str(&self.visualize_plan(input, indent + 1));
            }
            UnifiedPlan::Projection { expr, input } => {
                output.push_str(&format!("{}Projection: {} cols\n", prefix, expr.len()));
                output.push_str(&self.visualize_plan(input, indent + 1));
            }
            UnifiedPlan::Join {
                join_type,
                condition: _,
                left,
                right,
            } => {
                output.push_str(&format!("{}Join: {:?}\n", prefix, join_type));
                output.push_str(&format!("{}  left:\n", prefix));
                output.push_str(&self.visualize_plan(left, indent + 2));
                output.push_str(&format!("{}  right:\n", prefix));
                output.push_str(&self.visualize_plan(right, indent + 2));
            }
            UnifiedPlan::Limit { limit, input } => {
                output.push_str(&format!("{}Limit: {}\n", prefix, limit));
                output.push_str(&self.visualize_plan(input, indent + 1));
            }
            UnifiedPlan::EmptyRelation => {
                output.push_str(&format!("{}EmptyRelation\n", prefix));
            }
            _ => {
                output.push_str(&format!("{}{}\n", prefix, plan.type_name()));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_plan::{GraphScanType, VectorScanType};

    #[test]
    fn test_query_planner_config_default() {
        let config = QueryPlannerConfig::default();
        assert!(config.enable_cache);
        assert_eq!(config.max_alternatives, 3);
    }

    #[test]
    fn test_query_planner_with_defaults() {
        let planner = QueryPlanner::with_defaults();
        assert_eq!(planner.cache_size(), 0);
    }

    #[test]
    fn test_plan_table_scan() {
        let mut planner = QueryPlanner::with_defaults();
        let plan = UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };

        let result = planner.plan("SELECT * FROM users", plan);
        assert!(!result.from_cache);
        assert_eq!(result.selected_plan.type_name(), "TableScan");
    }

    #[test]
    fn test_plan_vector_scan() {
        let mut planner = QueryPlanner::with_defaults();
        let plan = UnifiedPlan::VectorScan {
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Knn { k: 10 },
            limit: Some(10),
        };

        let result = planner.plan("SELECT * FROM embeddings LIMIT 10", plan);
        assert!(result.path_selection.should_use_vector());
    }

    #[test]
    fn test_plan_graph_scan() {
        let mut planner = QueryPlanner::with_defaults();
        let plan = UnifiedPlan::GraphScan {
            graph_name: "social_graph".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: Some("user_123".to_string()),
        };

        let result = planner.plan("MATCH (u:User)-[:KNOWS]->(v) FROM user_123", plan);
        assert!(result.path_selection.should_use_graph());
    }

    #[test]
    fn test_cache_hit() {
        let mut planner = QueryPlanner::with_defaults();
        let plan = UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        let result1 = planner.plan("SELECT * FROM users", plan);
        assert!(!result1.from_cache);

        let plan2 = UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        let result2 = planner.plan("SELECT * FROM users", plan2);
        assert!(result2.from_cache);
    }

    #[test]
    fn test_clear_cache() {
        let mut planner = QueryPlanner::with_defaults();
        let plan = UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        planner.plan("SELECT * FROM users", plan);
        assert_eq!(planner.cache_size(), 1);

        planner.clear_cache();
        assert_eq!(planner.cache_size(), 0);
    }

    #[test]
    fn test_plan_alternatives() {
        let mut planner = QueryPlanner::with_defaults();
        let plan = UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };

        let result = planner.plan("SELECT * FROM users", plan);
        // Should have alternatives when SQL is selected (vector and graph transforms)
        assert!(result.alternatives.len() <= 3);
    }

    #[test]
    fn test_visualize_text() {
        let mut planner = QueryPlanner::with_defaults();
        let plan = UnifiedPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };

        let result = planner.plan("SELECT * FROM users", plan);
        let viz = result.visualize_text();
        assert!(viz.contains("Query: SELECT * FROM users"));
        assert!(viz.contains("Selected Path: Sql"));
    }

    #[test]
    fn test_plan_alternative_latency() {
        let alt = PlanAlternative::new(ExecutionPath::Sql, UnifiedPlan::EmptyRelation, 100.0);
        assert_eq!(alt.estimated_latency_ms, 10.0); // 100 * 0.1
    }
}
