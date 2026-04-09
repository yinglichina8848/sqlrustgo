# Unified Optimizer Implementation Plan (Issue #1339)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a unified optimizer that automatically selects the best execution path for mixed SQL + Vector + Graph workloads using CBO (Cost-Based Optimization).

**Architecture:** 
- Extend the existing `Plan` enum with `VectorScan` and `GraphScan` variants for unified plan representation
- Create specialized cost models for Vector (ANN search, similarity) and Graph (traversal, pattern matching) operations
- Implement `UnifiedCostModel` that combines SQL, Vector, and Graph cost estimation
- Create `PathSelector` that uses cost comparison to automatically choose between SQL table scan, vector index scan, or graph traversal
- Implement `QueryPlanner` for generating unified query plans with multiple execution alternatives
- Add LRU-based plan caching and text-based plan visualization

**Tech Stack:** Rust, lru_cache crate, std::collections::HashMap

---

## Task 1: Create unified_plan.rs with extended Plan enum

**Files:**
- Create: `crates/optimizer/src/unified_plan.rs`
- Modify: `crates/optimizer/src/lib.rs` (add module and exports)
- Test: `crates/optimizer/src/unified_plan.rs` (tests at bottom of file)

**Step 1: Create unified_plan.rs**

```rust
//! Unified Plan Types Module
//! 
//! Extends the basic Plan enum with VectorScan and GraphScan for unified query planning.

use super::rules::{Expr, Plan as SqlPlan, JoinType};
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
    Filter { predicate: Expr, input: Box<UnifiedPlan> },
    /// Projection operation (SELECT columns)
    Projection { expr: Vec<Expr>, input: Box<UnifiedPlan> },
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
    Sort { expr: Vec<Expr>, input: Box<UnifiedPlan> },
    /// Limit operation
    Limit { limit: usize, input: Box<UnifiedPlan> },
    
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
            UnifiedPlan::SqlVectorJoin { sql_plan, vector_plan, .. } => {
                (sql_plan.estimate_cardinality() * vector_plan.estimate_cardinality()) / 10
            }
            UnifiedPlan::SqlGraphJoin { sql_plan, graph_plan, .. } => {
                (sql_plan.estimate_cardinality() * graph_plan.estimate_cardinality()) / 10
            }
            UnifiedPlan::VectorGraphJoin { vector_plan, graph_plan, .. } => {
                (vector_plan.estimate_cardinality() * graph_plan.estimate_cardinality()) / 10
            }
            UnifiedPlan::EmptyRelation => 0,
        }
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
}
```

**Step 2: Update lib.rs to export unified_plan module**

Add to `crates/optimizer/src/lib.rs`:
```rust
pub mod unified_plan;
pub use unified_plan::{
    GraphPattern, GraphScanType, UnifiedPlan, VectorScanType,
};
```

**Step 3: Run tests**

Run: `cargo test -p sqlrustgo-optimizer unified_plan -- --nocapture`
Expected: All tests pass

---

## Task 2: Create vector_cost.rs for vector operation cost estimation

**Files:**
- Create: `crates/optimizer/src/vector_cost.rs`
- Modify: `crates/optimizer/src/lib.rs` (add module and exports)
- Test: `crates/optimizer/src/vector_cost.rs` (tests at bottom of file)

**Step 1: Create vector_cost.rs**

```rust
//! Vector Cost Model Module
//! 
//! Provides cost estimation for vector operations (ANN search, similarity search, KNN).

use super::unified_plan::{VectorScanType, UnifiedPlan};
use std::collections::HashMap;

/// Vector index types for cost estimation
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VectorIndexType {
    /// IVF (Inverted File) index
    Ivf,
    /// HNSW (Hierarchical Navigable Small World) index
    Hnsw,
    /// PQ (Product Quantization) based index
    PQ,
    /// Simple brute force (no index)
    BruteForce,
}

/// Cost factors for vector operations
#[derive(Debug, Clone)]
pub struct VectorCostFactors {
    /// CPU cost per vector comparison
    pub cpu_cost_per_vector_cmp: f64,
    /// Memory access cost per vector
    pub memory_cost_per_vector: f64,
    /// I/O cost per page for vector index
    pub io_cost_per_page: f64,
    /// Network cost per byte for distributed vector search
    pub network_cost_per_byte: f64,
    /// Index type specific cost multipliers
    pub index_type_multipliers: HashMap<VectorIndexType, f64>,
}

impl Default for VectorCostFactors {
    fn default() -> Self {
        let mut index_type_multipliers = HashMap::new();
        index_type_multipliers.insert(VectorIndexType::Ivf, 0.1);
        index_type_multipliers.insert(VectorIndexType::Hnsw, 0.05);
        index_type_multipliers.insert(VectorIndexType::PQ, 0.08);
        index_type_multipliers.insert(VectorIndexType::BruteForce, 1.0);
        
        Self {
            cpu_cost_per_vector_cmp: 0.001,
            memory_cost_per_vector: 0.0001,
            io_cost_per_page: 10.0,
            network_cost_per_byte: 0.001,
            index_type_multipliers,
        }
    }
}

impl VectorCostFactors {
    /// Get cost multiplier for index type
    pub fn get_index_multiplier(&self, index_type: &VectorIndexType) -> f64 {
        self.index_type_multipliers.get(index_type).copied().unwrap_or(1.0)
    }
}

/// VectorCostModel - cost estimation for vector operations
#[derive(Debug, Clone)]
pub struct VectorCostModel {
    factors: VectorCostFactors,
}

impl VectorCostModel {
    pub fn new(factors: VectorCostFactors) -> Self {
        Self { factors }
    }
    
    pub fn default_model() -> Self {
        Self {
            factors: VectorCostFactors::default(),
        }
    }
    
    /// Estimate cost for KNN search
    /// 
    /// Parameters:
    /// - k: number of nearest neighbors
    /// - n: total vectors in index
    /// - dimension: vector dimension
    /// - index_type: type of index used
    pub fn knn_cost(&self, k: usize, n: u64, dimension: u32, index_type: &VectorIndexType) -> f64 {
        let base_comparisons = n as f64 * self.factors.cpu_cost_per_vector_cmp * dimension as f64;
        let index_multiplier = self.factors.get_index_multiplier(index_type);
        base_comparisons * index_multiplier
    }
    
    /// Estimate cost for ANN search
    /// 
    /// Parameters:
    /// - n: total vectors in index
    /// - ef: search parameter (higher = more accurate but slower)
    /// - dimension: vector dimension
    /// - index_type: type of index used
    pub fn ann_cost(&self, n: u64, ef: usize, dimension: u32, index_type: &VectorIndexType) -> f64 {
        // ANN search scans a fraction of the index
        let scan_fraction = (ef as f64 / 100.0).min(1.0);
        let scanned_vectors = n as f64 * scan_fraction;
        let base_cost = scanned_vectors as f64 * self.factors.cpu_cost_per_vector_cmp * dimension as f64;
        let index_multiplier = self.factors.get_index_multiplier(index_type);
        base_cost * index_multiplier
    }
    
    /// Estimate cost for similarity search
    /// 
    /// Parameters:
    /// - n: total vectors
    /// - dimension: vector dimension
    /// - threshold: similarity threshold
    /// - index_type: type of index used
    pub fn similarity_cost(
        &self,
        n: u64,
        dimension: u32,
        threshold: f32,
        index_type: &VectorIndexType,
    ) -> f64 {
        // Similarity search typically scans more vectors than ANN
        let scan_fraction = (1.0 - threshold as f64).max(0.1);
        let scanned_vectors = n as f64 * scan_fraction;
        let base_cost = scanned_vectors as f64 * self.factors.cpu_cost_per_vector_cmp * dimension as f64;
        let index_multiplier = self.factors.get_index_multiplier(index_type);
        base_cost * index_multiplier
    }
    
    /// Estimate cost for range search in vector space
    /// 
    /// Parameters:
    /// - n: total vectors
    /// - dimension: vector dimension
    /// - radius: search radius
    /// - index_type: type of index used
    pub fn range_cost(
        &self,
        n: u64,
        dimension: u32,
        radius: f32,
        index_type: &VectorIndexType,
    ) -> f64 {
        // Range search scans vectors within radius
        let scan_fraction = (radius as f64 * 0.1).min(1.0);
        let scanned_vectors = n as f64 * scan_fraction;
        let base_cost = scanned_vectors as f64 * self.factors.cpu_cost_per_vector_cmp * dimension as f64;
        let index_multiplier = self.factors.get_index_multiplier(index_type);
        base_cost * index_multiplier
    }
    
    /// Estimate cost for a UnifiedPlan node
    pub fn estimate_plan_cost(&self, plan: &UnifiedPlan, dimension: u32) -> f64 {
        match plan {
            UnifiedPlan::VectorScan {
                scan_type, limit, ..
            } => {
                let n = 10000; // Assume 10k vectors - would come from stats
                let k = limit.unwrap_or(100);
                match scan_type {
                    VectorScanType::Knn { k: knn_k } => {
                        self.knn_cost(*knn_k, n, dimension, &VectorIndexType::Hnsw)
                    }
                    VectorScanType::Ann { threshold: _ } => {
                        self.ann_cost(n, k * 2, dimension, &VectorIndexType::Hnsw)
                    }
                    VectorScanType::Similarity { threshold } => {
                        self.similarity_cost(n, dimension, *threshold, &VectorIndexType::Hnsw)
                    }
                    VectorScanType::Range { radius } => {
                        self.range_cost(n, dimension, *radius, &VectorIndexType::Hnsw)
                    }
                }
            }
            UnifiedPlan::HybridVectorScan {
                sql_filter: Some(_),
                scan_type,
                limit,
                ..
            } => {
                // Hybrid has extra cost due to SQL pre-filtering
                let base_cost = self.estimate_plan_cost(
                    &UnifiedPlan::VectorScan {
                        vector_index: String::new(),
                        query_vector: vec![],
                        scan_type: scan_type.clone(),
                        limit: *limit,
                    },
                    dimension,
                );
                base_cost * 1.2 // 20% overhead for hybrid
            }
            _ => 0.0,
        }
    }
    
    /// Compare vector scan cost vs SQL table scan cost
    /// Returns true if vector scan is cheaper
    pub fn vector_scan_cheaper_than_sql(
        &self,
        vector_cardinality: u64,
        sql_cardinality: u64,
        dimension: u32,
        index_type: &VectorIndexType,
    ) -> bool {
        let vector_cost = self.ann_cost(vector_cardinality, 100, dimension, index_type);
        let sql_cost = sql_cardinality as f64 * 0.01; // Simplified SQL scan cost
        vector_cost < sql_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vector_cost_factors_default() {
        let factors = VectorCostFactors::default();
        assert_eq!(factors.cpu_cost_per_vector_cmp, 0.001);
        assert!(factors.get_index_multiplier(&VectorIndexType::Hnsw) < 
                factors.get_index_multiplier(&VectorIndexType::BruteForce));
    }
    
    #[test]
    fn test_knn_cost() {
        let model = VectorCostModel::default_model();
        let cost = model.knn_cost(10, 10000, 128, &VectorIndexType::Hnsw);
        assert!(cost > 0.0);
    }
    
    #[test]
    fn test_ann_cost() {
        let model = VectorCostModel::default_model();
        let cost = model.ann_cost(10000, 100, 128, &VectorIndexType::Hnsw);
        assert!(cost > 0.0);
    }
    
    #[test]
    fn test_similarity_cost() {
        let model = VectorCostModel::default_model();
        let cost = model.similarity_cost(10000, 128, 0.8, &VectorIndexType::Hnsw);
        assert!(cost > 0.0);
    }
    
    #[test]
    fn test_range_cost() {
        let model = VectorCostModel::default_model();
        let cost = model.range_cost(10000, 128, 0.5, &VectorIndexType::Hnsw);
        assert!(cost > 0.0);
    }
    
    #[test]
    fn test_vector_scan_cheaper_than_sql() {
        let model = VectorCostModel::default_model();
        // For high selectivity, vector should be cheaper
        let cheaper = model.vector_scan_cheaper_than_sql(10, 10000, 128, &VectorIndexType::Hnsw);
        assert!(cheaper);
    }
    
    #[test]
    fn test_estimate_plan_cost() {
        let model = VectorCostModel::default_model();
        let plan = UnifiedPlan::VectorScan {
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Knn { k: 10 },
            limit: Some(10),
        };
        let cost = model.estimate_plan_cost(&plan, 128);
        assert!(cost > 0.0);
    }
}
```

**Step 2: Update lib.rs**

Add exports:
```rust
pub mod vector_cost;
pub use vector_cost::{VectorCostFactors, VectorCostModel, VectorIndexType};
```

**Step 3: Run tests**

Run: `cargo test -p sqlrustgo-optimizer vector_cost -- --nocapture`
Expected: All tests pass

---

## Task 3: Create graph_cost.rs for graph operation cost estimation

**Files:**
- Create: `crates/optimizer/src/graph_cost.rs`
- Modify: `crates/optimizer/src/lib.rs` (add module and exports)
- Test: `crates/optimizer/src/graph_cost.rs` (tests at bottom of file)

**Step 1: Create graph_cost.rs**

```rust
//! Graph Cost Model Module
//! 
//! Provides cost estimation for graph operations (traversal, pattern matching, shortest path).

use super::unified_plan::{GraphScanType, UnifiedPlan, GraphPattern};
use std::collections::HashMap;

/// Graph index types for cost estimation
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GraphIndexType {
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
        self.index_type_multipliers.get(index_type).copied().unwrap_or(1.0)
    }
}

/// GraphCostModel - cost estimation for graph operations
#[derive(Debug, Clone)]
pub struct GraphCostModel {
    factors: GraphCostFactors,
}

impl GraphCostModel {
    pub fn new(factors: GraphCostFactors) -> Self {
        Self { factors }
    }
    
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
    pub fn traversal_cost(&self, max_depth: usize, avg_degree: f64, index_type: &GraphIndexType) -> f64 {
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
        let base_cost = graph_size as f64 * pattern_complexity * self.factors.pattern_match_multiplier;
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
            UnifiedPlan::GraphScan {
                scan_type, ..
            } => {
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
                    GraphScanType::ShortestPath { .. } => {
                        self.shortest_path_cost(graph_size, avg_degree, &GraphIndexType::AdjacencyList)
                    }
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
                    GraphScanType::ShortestPath { .. } => {
                        self.shortest_path_cost(graph_size, avg_degree, &GraphIndexType::AdjacencyList)
                    }
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
        assert!(factors.get_index_multiplier(&GraphIndexType::LabeledIndex) < 
                factors.get_index_multiplier(&GraphIndexType::AdjacencyList));
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
}
```

**Step 2: Update lib.rs**

Add exports:
```rust
pub mod graph_cost;
pub use graph_cost::{GraphCostFactors, GraphCostModel, GraphIndexType};
```

**Step 3: Run tests**

Run: `cargo test -p sqlrustgo-optimizer graph_cost -- --nocapture`
Expected: All tests pass

---

## Task 4: Create unified_cost.rs combining all cost models

**Files:**
- Create: `crates/optimizer/src/unified_cost.rs`
- Modify: `crates/optimizer/src/lib.rs` (add module and exports)
- Test: `crates/optimizer/src/unified_cost.rs` (tests at bottom of file)

**Step 1: Create unified_cost.rs**

```rust
//! Unified Cost Model Module
//! 
//! Combines SQL, Vector, and Graph cost models for unified cost estimation.

use super::cost::SimpleCostModel;
use super::vector_cost::VectorCostModel;
use super::graph_cost::GraphCostModel;
use super::unified_plan::UnifiedPlan;

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
    pub fn is_hybrid(&self) -> bool {
        matches!(
            self,
            ExecutionPath::HybridSqlVector
                | ExecutionPath::HybridSqlGraph
                | ExecutionPath::HybridVectorGraph
                | ExecutionPath::Unified
        )
    }
    
    pub fn involves_vector(&self) -> bool {
        matches!(
            self,
            ExecutionPath::Vector | ExecutionPath::HybridSqlVector | ExecutionPath::HybridVectorGraph | ExecutionPath::Unified
        )
    }
    
    pub fn involves_graph(&self) -> bool {
        matches!(
            self,
            ExecutionPath::Graph | ExecutionPath::HybridSqlGraph | ExecutionPath::HybridVectorGraph | ExecutionPath::Unified
        )
    }
}

/// UnifiedCostModel - combines all cost models for cross-domain optimization
#[derive(Debug, Clone)]
pub struct UnifiedCostModel {
    sql_cost_model: SimpleCostModel,
    vector_cost_model: VectorCostModel,
    graph_cost_model: GraphCostModel,
    /// Vector dimension for cost estimation
    vector_dimension: u32,
    /// Graph size for cost estimation
    graph_size: u64,
}

impl UnifiedCostModel {
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
        }
    }
    
    pub fn default_model(vector_dimension: u32, graph_size: u64) -> Self {
        Self {
            sql_cost_model: SimpleCostModel::default_model(),
            vector_cost_model: VectorCostModel::default_model(),
            graph_cost_model: GraphCostModel::default_model(),
            vector_dimension,
            graph_size,
        }
    }
    
    /// Estimate cost for any UnifiedPlan
    pub fn estimate_cost(&self, plan: &UnifiedPlan) -> f64 {
        match plan {
            // Pure SQL operations - delegate to SimpleCostModel
            UnifiedPlan::TableScan { table_name, projection } => {
                // Simplified: use row_count=1000, page_count=10
                let row_count = 1000u64;
                let page_count = 10u64;
                self.sql_cost_model.seq_scan_cost(row_count, page_count)
            }
            UnifiedPlan::IndexScan { .. } => {
                self.sql_cost_model.index_scan_cost(100, 1, 10)
            }
            UnifiedPlan::Filter { input, .. } => {
                self.estimate_cost(input) * 1.1 // 10% overhead
            }
            UnifiedPlan::Projection { input, .. } => {
                self.estimate_cost(input)
            }
            UnifiedPlan::Join { left, right, join_type, .. } => {
                let left_rows = left.estimate_cardinality();
                let right_rows = right.estimate_cardinality();
                let join_method = match join_type {
                    super::rules::JoinType::Inner => "hash_join",
                    _ => "nested_loop",
                };
                self.sql_cost_model.join_cost(left_rows, right_rows, join_method)
            }
            UnifiedPlan::Aggregate { input, group_by, .. } => {
                let row_count = input.estimate_cardinality();
                self.sql_cost_model.agg_cost(row_count, group_by.len() as u32)
            }
            UnifiedPlan::Sort { input, .. } => {
                let row_count = input.estimate_cardinality();
                self.sql_cost_model.sort_cost(row_count, 100)
            }
            UnifiedPlan::Limit { limit, input } => {
                self.estimate_cost(input).min(*limit as f64 * 0.1)
            }
            
            // Vector operations - delegate to VectorCostModel
            UnifiedPlan::VectorScan { .. } | UnifiedPlan::HybridVectorScan { .. } => {
                self.vector_cost_model.estimate_plan_cost(plan, self.vector_dimension)
            }
            
            // Graph operations - delegate to GraphCostModel
            UnifiedPlan::GraphScan { .. } | UnifiedPlan::HybridGraphScan { .. } => {
                self.graph_cost_model.estimate_plan_cost(plan, self.graph_size)
            }
            
            // Cross-domain joins - combine costs
            UnifiedPlan::SqlVectorJoin { sql_plan, vector_plan, .. } => {
                self.estimate_cost(sql_plan) + self.estimate_cost(vector_plan) * 1.2
            }
            UnifiedPlan::SqlGraphJoin { sql_plan, graph_plan, .. } => {
                self.estimate_cost(sql_plan) + self.estimate_cost(graph_plan) * 1.2
            }
            UnifiedPlan::VectorGraphJoin { vector_plan, graph_plan, .. } => {
                self.estimate_cost(vector_plan) + self.estimate_cost(graph_plan) * 1.2
            }
            
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
        
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        candidates.first().copied().unwrap_or((ExecutionPath::Sql, sql_cost))
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
    use super::super::unified_plan::{VectorScanType, GraphScanType};
    
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
            join_type: super::rules::JoinType::Inner,
            condition: None,
        };
        let cost = model.estimate_cost(&plan);
        assert!(cost > 0.0);
    }
}
```

**Step 2: Update lib.rs**

Add exports:
```rust
pub mod unified_cost;
pub use unified_cost::{ExecutionPath, UnifiedCostModel};
```

**Step 3: Run tests**

Run: `cargo test -p sqlrustgo-optimizer unified_cost -- --nocapture`
Expected: All tests pass

---

## Task 5: Create path_selector.rs for automatic execution path selection

**Files:**
- Create: `crates/optimizer/src/path_selector.rs`
- Modify: `crates/optimizer/src/lib.rs` (add module and exports)
- Test: `crates/optimizer/src/path_selector.rs` (tests at bottom of file)

**Step 1: Create path_selector.rs**

```rust
//! Path Selector Module
//! 
//! Automatically selects the best execution path based on cost estimation.

use super::unified_cost::{ExecutionPath, UnifiedCostModel};
use super::unified_plan::UnifiedPlan;
use std::collections::HashMap;

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
    pub fn new(config: PathSelectorConfig, unified_cost_model: UnifiedCostModel) -> Self {
        Self {
            config,
            unified_cost_model,
        }
    }
    
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
                    vector_cost, sql_cost,
                    if cardinality < self.config.vector_scan_threshold { "Low" } else { "High" }
                )
            }
            ExecutionPath::Graph => {
                format!(
                    "Graph path selected. Cost: {:.2} vs SQL: {:.2}.",
                    graph_cost, sql_cost
                )
            }
            ExecutionPath::HybridSqlVector => {
                format!(
                    "Hybrid SQL+Vector path selected. Best for mixed workloads."
                )
            }
            ExecutionPath::HybridSqlGraph => {
                format!(
                    "Hybrid SQL+Graph path selected. Best for graph-enriched SQL queries."
                )
            }
            ExecutionPath::Unified => {
                format!(
                    "Unified path selected. All three domains involved in query."
                )
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
    use super::super::unified_plan::{VectorScanType, GraphScanType};
    
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
}
```

**Step 2: Update lib.rs**

Add exports:
```rust
pub mod path_selector;
pub use path_selector::{PathSelector, PathSelectorConfig, PathSelection};
```

**Step 3: Run tests**

Run: `cargo test -p sqlrustgo-optimizer path_selector -- --nocapture`
Expected: All tests pass

---

## Task 6: Create query_planner.rs for unified query plan generation

**Files:**
- Create: `crates/optimizer/src/query_planner.rs`
- Modify: `crates/optimizer/src/lib.rs` (add module and exports)
- Test: `crates/optimizer/src/query_planner.rs` (tests at bottom of file)

**Step 1: Create query_planner.rs**

```rust
//! Query Planner Module
//! 
//! Unified query plan generation with multiple execution alternatives.

use super::path_selector::{PathSelector, PathSelection};
use super::unified_cost::{ExecutionPath, UnifiedCostModel};
use super::unified_plan::UnifiedPlan;
use std::collections::HashMap;

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
    pub fn new(config: QueryPlannerConfig, unified_cost_model: UnifiedCostModel) -> Self {
        let path_selector = PathSelector::new(
            super::path_selector::PathSelectorConfig::default(),
            unified_cost_model.clone(),
        );
        
        Self {
            config,
            path_selector,
            unified_cost_model,
            plan_cache: HashMap::new(),
        }
    }
    
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
        let selected_plan = alternatives.first()
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
        alternatives.sort_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap_or(std::cmp::Ordering::Equal));
        alternatives.truncate(max_alts);
        
        // Ensure selected path is first
        alternatives.sort_by(|a, b| {
            if a.path == path_selection.recommended_path {
                std::cmp::Ordering::Less
            } else if b.path == path_selection.recommended_path {
                std::cmp::Ordering::Greater
            } else {
                a.cost.partial_cmp(&b.cost).unwrap_or(std::cmp::Ordering::Equal)
            }
        });
        
        alternatives
    }
    
    /// Transform plan to use vector execution if possible
    fn transform_to_vector_plan(&self, plan: &UnifiedPlan) -> UnifiedPlan {
        // For demonstration, convert TableScan to VectorScan
        // In production, this would be a more sophisticated transformation
        match plan {
            UnifiedPlan::TableScan { table_name, projection } => {
                UnifiedPlan::VectorScan {
                    vector_index: format!("{}_embedding_idx", table_name),
                    query_vector: vec![0.0; self.config.default_vector_dim],
                    scan_type: super::unified_plan::VectorScanType::Ann {
                        threshold: 0.8,
                    },
                    limit: Some(100),
                }
            }
            _ => plan.clone(),
        }
    }
    
    /// Transform plan to use graph execution if possible
    fn transform_to_graph_plan(&self, plan: &UnifiedPlan) -> UnifiedPlan {
        // For demonstration, convert TableScan to GraphScan
        match plan {
            UnifiedPlan::TableScan { table_name, projection: _ } => {
                UnifiedPlan::GraphScan {
                    graph_name: format!("{}_graph", table_name),
                    scan_type: super::unified_plan::GraphScanType::Traversal {
                        max_depth: 3,
                    },
                    start_node: None,
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    
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
}
```

**Step 2: Update lib.rs**

Add exports:
```rust
pub mod query_planner;
pub use query_planner::{PlanAlternative, QueryPlanner, QueryPlannerConfig, QueryPlanResult};
```

**Step 3: Run tests**

Run: `cargo test -p sqlrustgo-optimizer query_planner -- --nocapture`
Expected: All tests pass

---

## Task 7: Add plan visualization to query_planner.rs

**Files:**
- Modify: `crates/optimizer/src/query_planner.rs` (add visualization methods)
- Test: Add visualization tests

**Step 1: Add visualization to QueryPlanResult**

Add to `query_planner.rs`:

```rust
impl QueryPlanResult {
    /// Generate text-based visualization of the plan
    pub fn visualize_text(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Query: {}\n", self.query));
        output.push_str(&format!("Selected Path: {:?}\n", self.path_selection.recommended_path));
        output.push_str(&format!("Estimated Cost: {:.2}\n", self.path_selection.estimated_cost));
        output.push_str(&format!("From Cache: {}\n", self.from_cache));
        output.push_str("\n--- Selected Plan ---\n");
        output.push_str(&self.visualize_plan(&self.selected_plan, 0));
        output.push_str("\n--- Alternatives ---\n");
        for (i, alt) in self.alternatives.iter().enumerate() {
            output.push_str(&format!("\nAlternative {}: {:?} (cost: {:.2}, latency: {:.2}ms)\n",
                i + 1, alt.path, alt.cost, alt.estimated_latency_ms));
            output.push_str(&self.visualize_plan(&alt.plan, 0));
        }
        output
    }
    
    fn visualize_plan(&self, plan: &UnifiedPlan, indent: usize) -> String {
        let prefix = "  ".repeat(indent);
        let mut output = String::new();
        
        match plan {
            UnifiedPlan::TableScan { table_name, projection } => {
                output.push_str(&format!("{}TableScan: {}\n", prefix, table_name));
                if let Some(cols) = projection {
                    output.push_str(&format!("{}  projection: {:?}\n", prefix, cols));
                }
            }
            UnifiedPlan::VectorScan { vector_index, scan_type, limit, .. } => {
                output.push_str(&format!("{}VectorScan: {} ({:?})\n", prefix, vector_index, scan_type));
                if let Some(l) = limit {
                    output.push_str(&format!("{}  limit: {}\n", prefix, l));
                }
            }
            UnifiedPlan::GraphScan { graph_name, scan_type, start_node } => {
                output.push_str(&format!("{}GraphScan: {} ({:?})\n", prefix, graph_name, scan_type));
                if let Some(node) = start_node {
                    output.push_str(&format!("{}  start_node: {}\n", prefix, node));
                }
            }
            UnifiedPlan::Filter { predicate, input } => {
                output.push_str(&format!("{}Filter\n", prefix));
                output.push_str(&self.visualize_plan(input, indent + 1));
            }
            UnifiedPlan::Projection { expr, input } => {
                output.push_str(&format!("{}Projection: {} cols\n", prefix, expr.len()));
                output.push_str(&self.visualize_plan(input, indent + 1));
            }
            UnifiedPlan::Join { join_type, condition, left, right } => {
                output.push_str(&format!("{}Join: {:?}\n", prefix, join_type));
                if condition.is_some() {
                    output.push_str(&format!("{}  condition: present\n", prefix));
                }
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
```

**Step 2: Add test for visualization**

```rust
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
```

**Step 3: Run tests**

Run: `cargo test -p sqlrustgo-optimizer query_planner::tests::test_visualize_text -- --nocapture`
Expected: PASS

---

## Task 8: Run all tests and ensure everything passes

**Step 1: Run full optimizer test suite**

Run: `cargo test -p sqlrustgo-optimizer -- --nocapture`
Expected: All tests pass (including existing tests + new tests)

**Step 2: Check for any compilation warnings**

Run: `cargo clippy -p sqlrustgo-optimizer -- -D warnings`
Expected: No warnings

---

## Task 9: Commit changes

**Step 1: Stage and commit**

```bash
git add -A
git commit -m "feat(optimizer): add unified optimizer for CBO-based execution path selection (Issue #1339)

- Add unified_plan.rs with VectorScan, GraphScan, and hybrid plan variants
- Add vector_cost.rs for ANN/KNN/similarity search cost estimation
- Add graph_cost.rs for traversal/pattern matching cost estimation  
- Add unified_cost.rs combining SQL + Vector + Graph cost models
- Add path_selector.rs for automatic execution path selection
- Add query_planner.rs for unified plan generation with caching
- Add plan visualization (text format)

Implements Issue #1339: Unified Optimizer - CBO自动选择执行路径"
```

---

## Task 10: Create PR and merge

**Step 1: Push branch**

```bash
git push -u origin feature/unified-optimizer
```

**Step 2: Create PR**

Use GitHub CLI or UI to create PR targeting `develop/v2.5.0`

**Step 3: Merge after CI passes**

---

## Verification Checklist

Before claiming completion, verify:

- [ ] All tests pass: `cargo test -p sqlrustgo-optimizer`
- [ ] No clippy warnings: `cargo clippy -p sqlrustgo-optimizer -- -D warnings`
- [ ] Code compiles: `cargo build -p sqlrustgo-optimizer`
- [ ] Plan visualization works
- [ ] Plan caching works
- [ ] Path selection chooses correct execution path
- [ ] All new modules have tests
- [ ] PR created and merged