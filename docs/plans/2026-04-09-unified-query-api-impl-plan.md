# Unified Query API Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement unified query API supporting SQL + Vector + Graph hybrid queries with parallel execution and unified scoring.

**Architecture:** Create new `crates/unified-query/` crate with ParallelExecutor using `tokio::join!`, ResultFusion with weighted scoring, and QueryCache with LRU policy. Adapters provide error isolation for each engine.

**Tech Stack:** Rust, tokio, serde, lru-cache, sqlrustgo-storage, sqlrustgo-vector, sqlrustgo-graph

---

## Task 1: Create crate structure

**Files:**
- Create: `crates/unified-query/Cargo.toml`
- Create: `crates/unified-query/src/lib.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "sqlrustgo-unified-query"
version.workspace = true
edition.workspace = true

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
lru-cache.workspace = true
parking_lot.workspace = true

sqlrustgo-storage.workspace = true
sqlrustgo-vector.workspace = true
sqlrustgo-graph.workspace = true
anyhow.workspace = true
thiserror.workspace = true
```

**Step 2: Create lib.rs**

```rust
pub mod api;
pub mod engine;
pub mod executor;
pub mod fusion;
pub mod router;
pub mod cache;
pub mod stats;
pub mod adapters;
pub mod error;
```

**Step 3: Run build verification**

Run: `cargo build -p sqlrustgo-unified-query`
Expected: Build succeeds (empty crate)

---

## Task 2: Implement error types

**Files:**
- Create: `crates/unified-query/src/error.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_result_ok() {
        let result: QueryResult<i32> = QueryResult::Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or_default(), Some(42));
    }

    #[test]
    fn test_query_result_partial() {
        let result: QueryResult<i32> = QueryResult::Partial(vec!["warning1".to_string()]);
        assert!(result.is_partial());
        assert_eq!(result.unwrap_or_default(), None);
    }

    #[test]
    fn test_query_result_err() {
        let result: QueryResult<i32> = QueryResult::Err("error".to_string());
        assert!(!result.is_ok());
        assert!(!result.is_partial());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-unified-query error::tests`
Expected: FAIL - module not found

**Step 3: Write minimal implementation**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnifiedQueryError {
    #[error("SQL error: {0}")]
    Sql(String),
    #[error("Vector error: {0}")]
    Vector(String),
    #[error("Graph error: {0}")]
    Graph(String),
    #[error("Routing error: {0}")]
    Routing(String),
    #[error("Fusion error: {0}")]
    Fusion(String),
}

pub enum QueryResult<T> {
    Ok(T),
    Partial(Vec<String>),
    Err(String),
}

impl<T> QueryResult<T> {
    pub fn is_ok(&self) -> bool {
        matches!(self, QueryResult::Ok(_))
    }

    pub fn is_partial(&self) -> bool {
        matches!(self, QueryResult::Partial(_))
    }

    pub fn unwrap_or_default(&self) -> Option<T> {
        match self {
            QueryResult::Ok(t) => Some(t.clone()),
            _ => None,
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-unified-query error::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/unified-query/src/error.rs
git commit -m "feat(unified-query): add error types and QueryResult"
```

---

## Task 3: Implement API types

**Files:**
- Create: `crates/unified-query/src/api.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_query_request_default() {
        let request = UnifiedQueryRequest {
            query: "test".to_string(),
            mode: QueryMode::SQL,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: Some(10),
            offset: Some(0),
        };
        assert_eq!(request.top_k, Some(10));
        assert_eq!(request.mode, QueryMode::SQL);
    }

    #[test]
    fn test_weights_default() {
        let weights = Weights::default();
        assert_eq!(weights.sql, 0.4);
        assert_eq!(weights.vector, 0.3);
        assert_eq!(weights.graph, 0.3);
    }

    #[test]
    fn test_query_mode_variants() {
        let modes = vec![
            QueryMode::SQL,
            QueryMode::Vector,
            QueryMode::Graph,
            QueryMode::SQLVector,
            QueryMode::SQLGraph,
            QueryMode::VectorGraph,
            QueryMode::SQLVectorGraph,
        ];
        assert_eq!(modes.len(), 7);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-unified-query api::tests`
Expected: FAIL - module not found

**Step 3: Write minimal implementation**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMode {
    SQL,
    Vector,
    Graph,
    SQLVector,
    SQLGraph,
    VectorGraph,
    SQLVectorGraph,
}

impl Default for QueryMode {
    fn default() -> Self {
        QueryMode::SQL
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weights {
    pub sql: f32,
    pub vector: f32,
    pub graph: f32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            sql: 0.4,
            vector: 0.3,
            graph: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorQuery {
    pub column: String,
    pub top_k: u32,
    #[serde(default)]
    pub filter: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQuery {
    pub start_nodes: Vec<String>,
    pub traversal: TraversalType,
    pub max_depth: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraversalType {
    BFS,
    DFS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedQueryRequest {
    pub query: String,
    pub mode: QueryMode,
    #[serde(default)]
    pub filters: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub weights: Option<Weights>,
    #[serde(default)]
    pub vector_query: Option<VectorQuery>,
    #[serde(default)]
    pub graph_query: Option<GraphQuery>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedQueryResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql_results: Option<Vec<Vec<serde_json::Value>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_results: Option<Vec<VectorResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_results: Option<Vec<GraphResult>>,
    pub fusion_scores: Vec<FusionScore>,
    pub total: u64,
    pub execution_time_ms: u64,
    pub query_plan: QueryPlanDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorResult {
    pub id: String,
    pub score: f32,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResult {
    pub path: Vec<String>,
    pub score: f32,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionScore {
    pub id: String,
    pub score: f32,
    pub source: Vec<String>,  // which engines contributed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlanDetail {
    pub mode: String,
    pub weights: Weights,
    pub steps: Vec<QueryStep>,
    pub fusion_time_ms: u64,
    pub total_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStep {
    pub source: String,
    pub step: String,
    pub time_ms: u64,
    #[serde(default)]
    pub rows_affected: Option<u64>,
    #[serde(default)]
    pub nodes_visited: Option<u64>,
    #[serde(default)]
    pub nodes_traversed: Option<u64>,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-unified-query api::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/unified-query/src/api.rs
git commit -m "feat(unified-query): add API types and request/response structs"
```

---

## Task 4: Implement QueryStats

**Files:**
- Create: `crates/unified-query/src/stats.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_stats_default() {
        let stats = QueryStats::default();
        assert!(!stats.cache_hit);
        assert_eq!(stats.total_time_ms, 0);
    }

    #[test]
    fn test_query_stats_builder() {
        let stats = QueryStats::builder()
            .with_sql_time(10)
            .with_vector_time(5)
            .with_graph_time(3)
            .with_cache_hit(true)
            .build();
        
        assert_eq!(stats.sql_time_ms, 10);
        assert_eq!(stats.vector_time_ms, 5);
        assert_eq!(stats.graph_time_ms, 3);
        assert!(stats.cache_hit);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-unified-query stats::tests`
Expected: FAIL

**Step 3: Write minimal implementation**

```rust
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct QueryStats {
    pub query_id: Uuid,
    pub total_time_ms: u64,
    pub sql_time_ms: u64,
    pub vector_time_ms: u64,
    pub graph_time_ms: u64,
    pub sql_hit: bool,
    pub vector_hit: bool,
    pub graph_hit: bool,
    pub cache_hit: bool,
}

impl Default for QueryStats {
    fn default() -> Self {
        Self {
            query_id: Uuid::new_v4(),
            total_time_ms: 0,
            sql_time_ms: 0,
            vector_time_ms: 0,
            graph_time_ms: 0,
            sql_hit: false,
            vector_hit: false,
            graph_hit: false,
            cache_hit: false,
        }
    }
}

pub struct QueryStatsBuilder {
    stats: QueryStats,
}

impl QueryStatsBuilder {
    pub fn new() -> Self {
        Self {
            stats: QueryStats::default(),
        }
    }

    pub fn with_sql_time(mut self, ms: u64) -> Self {
        self.stats.sql_time_ms = ms;
        self
    }

    pub fn with_vector_time(mut self, ms: u64) -> Self {
        self.stats.vector_time_ms = ms;
        self
    }

    pub fn with_graph_time(mut self, ms: u64) -> Self {
        self.stats.graph_time_ms = ms;
        self
    }

    pub fn with_cache_hit(mut self, hit: bool) -> Self {
        self.stats.cache_hit = hit;
        self
    }

    pub fn build(self) -> QueryStats {
        self.stats.total_time_ms = self.stats.sql_time_ms 
            + self.stats.vector_time_ms 
            + self.stats.graph_time_ms;
        self.stats
    }
}

impl Default for QueryStatsBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-unified-query stats::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/unified-query/src/stats.rs
git commit -m "feat(unified-query): add QueryStats for execution tracking"
```

---

## Task 5: Implement QueryCache

**Files:**
- Create: `crates/unified-query/src/cache.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic() {
        let cache = QueryCache::new(2);
        cache.insert("key1".to_string(), vec![1, 2, 3]);
        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = QueryCache::new(2);
        cache.insert("key1".to_string(), vec![1]);
        cache.insert("key2".to_string(), vec![2]);
        cache.insert("key3".to_string(), vec![3]); // Should evict key1
        assert!(cache.get("key1").is_none());
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
    }

    #[test]
    fn test_cache_miss() {
        let cache = QueryCache::new(1);
        assert!(cache.get("nonexistent").is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-unified-query cache::tests`
Expected: FAIL

**Step 3: Write minimal implementation**

```rust
use lru_cache::LruCache;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct QueryCache {
    cache: Arc<RwLock<LruCache<String, Vec<i32>>>>,
    capacity: usize,
}

impl QueryCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            capacity,
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<i32>> {
        self.cache.read().get(key).cloned()
    }

    pub fn insert(&self, key: String, value: Vec<i32>) {
        let mut cache = self.cache.write();
        if cache.len() >= self.capacity {
            // Remove oldest entry
            if let Some(oldest_key) = cache.iter().next().map(|(k, _)| k.clone()) {
                cache.remove(&oldest_key);
            }
        }
        cache.insert(key, value);
    }

    pub fn clear(&self) {
        self.cache.write().clear();
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-unified-query cache::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/unified-query/src/cache.rs
git commit -m "feat(unified-query): add QueryCache with LRU eviction"
```

---

## Task 6: Implement Router

**Files:**
- Create: `crates/unified-query/src/router.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_sql_mode() {
        let router = QueryRouter::new();
        let request = UnifiedQueryRequest {
            query: "SELECT * FROM users".to_string(),
            mode: QueryMode::SQL,
            // ... other fields default
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: None,
            offset: None,
        };
        
        let plan = router.route(&request).unwrap();
        assert!(plan.execute_sql);
        assert!(!plan.execute_vector);
        assert!(!plan.execute_graph);
    }

    #[test]
    fn test_router_unified_mode() {
        let router = QueryRouter::new();
        let request = UnifiedQueryRequest {
            query: "test".to_string(),
            mode: QueryMode::SQLVectorGraph,
            // ... other fields default
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: None,
            offset: None,
        };
        
        let plan = router.route(&request).unwrap();
        assert!(plan.execute_sql);
        assert!(plan.execute_vector);
        assert!(plan.execute_graph);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-unified-query router::tests`
Expected: FAIL

**Step 3: Write minimal implementation**

```rust
use crate::api::{QueryMode, UnifiedQueryRequest, Weights};
use crate::error::UnifiedQueryError;

#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub execute_sql: bool,
    pub execute_vector: bool,
    pub execute_graph: bool,
    pub weights: Weights,
    pub top_k: u32,
    pub offset: u32,
}

pub struct QueryRouter;

impl QueryRouter {
    pub fn new() -> Self {
        Self
    }

    pub fn route(&self, request: &UnifiedQueryRequest) -> Result<QueryPlan, UnifiedQueryError> {
        let (execute_sql, execute_vector, execute_graph) = match request.mode {
            QueryMode::SQL => (true, false, false),
            QueryMode::Vector => (false, true, false),
            QueryMode::Graph => (false, false, true),
            QueryMode::SQLVector => (true, true, false),
            QueryMode::SQLGraph => (true, false, true),
            QueryMode::VectorGraph => (false, true, true),
            QueryMode::SQLVectorGraph => (true, true, true),
        };

        let weights = request.weights.clone().unwrap_or_default();
        let top_k = request.top_k.unwrap_or(10);
        let offset = request.offset.unwrap_or(0);

        Ok(QueryPlan {
            execute_sql,
            execute_vector,
            execute_graph,
            weights,
            top_k,
            offset,
        })
    }
}

impl Default for QueryRouter {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-unified-query router::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/unified-query/src/router.rs
git commit -m "feat(unified-query): add QueryRouter for mode-based routing"
```

---

## Task 7: Implement adapters (Storage, Vector, Graph)

**Files:**
- Create: `crates/unified-query/src/adapters/mod.rs`
- Create: `crates/unified-query/src/adapters/storage.rs`
- Create: `crates/unified-query/src/adapters/vector.rs`
- Create: `crates/unified-query/src/adapters/graph.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_adapter_result() {
        let result: QueryResult<Vec<Vec<serde_json::Value>>> = QueryResult::Ok(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_vector_adapter_result() {
        let result: QueryResult<Vec<VectorResult>> = QueryResult::Err("connection failed".to_string());
        assert!(!result.is_ok());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-unified-query adapters::tests`
Expected: FAIL

**Step 3: Write minimal implementation**

```rust
// adapters/mod.rs
pub mod storage;
pub mod vector;
pub mod graph;

pub use storage::StorageAdapter;
pub use vector::VectorAdapter;
pub use graph::GraphAdapter;
```

```rust
// adapters/storage.rs
use crate::api::{QueryPlan, UnifiedQueryRequest};
use crate::error::{QueryResult, UnifiedQueryError};
use serde_json::Value;

pub struct StorageAdapter;

impl StorageAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(
        &self,
        _request: &UnifiedQueryRequest,
        _plan: &QueryPlan,
    ) -> QueryResult<Vec<Vec<Value>>> {
        // TODO: Integrate with sqlrustgo-storage
        QueryResult::Err("Not implemented".to_string())
    }
}

impl Default for StorageAdapter {
    fn default() -> Self {
        Self::new()
    }
}
```

```rust
// adapters/vector.rs
use crate::api::{QueryPlan, UnifiedQueryRequest, VectorResult};
use crate::error::{QueryResult, UnifiedQueryError};

pub struct VectorAdapter;

impl VectorAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(
        &self,
        _request: &UnifiedQueryRequest,
        _plan: &QueryPlan,
    ) -> QueryResult<Vec<VectorResult>> {
        // TODO: Integrate with sqlrustgo-vector
        QueryResult::Err("Not implemented".to_string())
    }
}

impl Default for VectorAdapter {
    fn default() -> Self {
        Self::new()
    }
}
```

```rust
// adapters/graph.rs
use crate::api::{GraphResult, QueryPlan, UnifiedQueryRequest};
use crate::error::{QueryResult, UnifiedQueryError};

pub struct GraphAdapter;

impl GraphAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(
        &self,
        _request: &UnifiedQueryRequest,
        _plan: &QueryPlan,
    ) -> QueryResult<Vec<GraphResult>> {
        // TODO: Integrate with sqlrustgo-graph
        QueryResult::Err("Not implemented".to_string())
    }
}

impl Default for GraphAdapter {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-unified-query adapters::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/unified-query/src/adapters/
git commit -m "feat(unified-query): add adapters for Storage, Vector, Graph"
```

---

## Task 8: Implement ParallelExecutor

**Files:**
- Create: `crates/unified-query/src/executor.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_executor_all_engines() {
        let executor = ParallelExecutor::new();
        let plan = QueryPlan {
            execute_sql: true,
            execute_vector: true,
            execute_graph: true,
            weights: Weights::default(),
            top_k: 10,
            offset: 0,
        };
        
        let request = UnifiedQueryRequest {
            query: "test".to_string(),
            mode: QueryMode::SQLVectorGraph,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: None,
            offset: None,
        };
        
        let results = executor.execute(&request, &plan).await;
        assert!(results.sql_results.is_some());
        assert!(results.vector_results.is_some());
        assert!(results.graph_results.is_some());
    }

    #[tokio::test]
    async fn test_parallel_executor_sql_only() {
        let executor = ParallelExecutor::new();
        let plan = QueryPlan {
            execute_sql: true,
            execute_vector: false,
            execute_graph: false,
            weights: Weights::default(),
            top_k: 10,
            offset: 0,
        };
        
        let request = UnifiedQueryRequest {
            query: "test".to_string(),
            mode: QueryMode::SQL,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: None,
            offset: None,
        };
        
        let results = executor.execute(&request, &plan).await;
        assert!(results.sql_results.is_some());
        assert!(results.vector_results.is_none());
        assert!(results.graph_results.is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-unified-query executor::tests`
Expected: FAIL

**Step 3: Write minimal implementation**

```rust
use crate::adapters::{GraphAdapter, StorageAdapter, VectorAdapter};
use crate::api::{GraphResult, QueryPlan, UnifiedQueryRequest, VectorResult};
use crate::error::QueryResult;
use serde_json::Value;

pub struct ParallelExecutor {
    storage: StorageAdapter,
    vector: VectorAdapter,
    graph: GraphAdapter,
}

pub struct ParallelQueryResults {
    pub sql_results: Option<QueryResult<Vec<Vec<Value>>>>,
    pub vector_results: Option<QueryResult<Vec<VectorResult>>>,
    pub graph_results: Option<QueryResult<Vec<GraphResult>>>,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self {
            storage: StorageAdapter::new(),
            vector: VectorAdapter::new(),
            graph: GraphAdapter::new(),
        }
    }

    pub async fn execute(
        &self,
        request: &UnifiedQueryRequest,
        plan: &QueryPlan,
    ) -> ParallelQueryResults {
        let sql_future = if plan.execute_sql {
            Some(self.storage.execute(request, plan))
        } else {
            None
        };

        let vector_future = if plan.execute_vector {
            Some(self.vector.execute(request, plan))
        } else {
            None
        };

        let graph_future = if plan.execute_graph {
            Some(self.graph.execute(request, plan))
        } else {
            None
        };

        let (sql_results, vector_results, graph_results) = tokio::join!(
            async { sql_future.map(|f| f.await).unwrap_or(QueryResult::Err("Not executed".to_string())) },
            async { vector_future.map(|f| f.await).unwrap_or(QueryResult::Err("Not executed".to_string())) },
            async { graph_future.map(|f| f.await).unwrap_or(QueryResult::Err("Not executed".to_string())) },
        );

        ParallelQueryResults {
            sql_results: if plan.execute_sql { Some(sql_results) } else { None },
            vector_results: if plan.execute_vector { Some(vector_results) } else { None },
            graph_results: if plan.execute_graph { Some(graph_results) } else { None },
        }
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-unified-query executor::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/unified-query/src/executor.rs
git commit -m "feat(unified-query): add ParallelExecutor with tokio::join!"
```

---

## Task 9: Implement ResultFusion

**Files:**
- Create: `crates/unified-query/src/fusion.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fusion_single_source() {
        let fusion = ResultFusion::new();
        let results = ParallelQueryResults {
            sql_results: Some(QueryResult::Ok(vec![
                vec![serde_json::json!("id1")],
                vec![serde_json::json!("id2")],
            ])),
            vector_results: None,
            graph_results: None,
        };
        
        let fused = fusion.fuse(results, &Weights::default(), 10);
        assert_eq!(fused.scores.len(), 2);
    }

    #[test]
    fn test_weights_sum_to_one() {
        let weights = Weights::default();
        let sum = weights.sql + weights.vector + weights.graph;
        assert!((sum - 1.0).abs() < 0.001);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-unified-query fusion::tests`
Expected: FAIL

**Step 3: Write minimal implementation**

```rust
use crate::api::{FusionScore, GraphResult, QueryPlan, VectorResult, Weights};
use crate::executor::ParallelQueryResults;
use crate::error::QueryResult;
use serde_json::Value;

pub struct ResultFusion;

pub struct FusionResult {
    pub scores: Vec<FusionScore>,
    pub total: usize,
}

impl ResultFusion {
    pub fn new() -> Self {
        Self
    }

    pub fn fuse(
        &self,
        results: ParallelQueryResults,
        weights: &Weights,
        top_k: u32,
    ) -> FusionResult {
        let mut all_scores: Vec<FusionScore> = Vec::new();

        // Process SQL results
        if let Some(sql_results) = results.sql_results {
            if let QueryResult::Ok(rows) = sql_results {
                for (idx, row) in rows.iter().enumerate() {
                    let sql_score = 1.0 - (idx as f32 * 0.01).min(0.5);
                    all_scores.push(FusionScore {
                        id: format!("sql_{}", idx),
                        score: weights.sql * sql_score,
                        source: vec!["sql".to_string()],
                    });
                }
            }
        }

        // Process Vector results
        if let Some(vector_results) = results.vector_results {
            if let QueryResult::Ok(results) = vector_results {
                for result in results {
                    all_scores.push(FusionScore {
                        id: result.id.clone(),
                        score: weights.vector * result.score,
                        source: vec!["vector".to_string()],
                    });
                }
            }
        }

        // Process Graph results
        if let Some(graph_results) = results.graph_results {
            if let QueryResult::Ok(results) = graph_results {
                for result in results {
                    let path_id = result.path.join("_");
                    all_scores.push(FusionScore {
                        id: path_id,
                        score: weights.graph * result.score,
                        source: vec!["graph".to_string()],
                    });
                }
            }
        }

        // Sort by score descending
        all_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate by id, keeping highest score
        let mut seen: std::collections::HashMap<String, FusionScore> = std::collections::HashMap::new();
        for score in &all_scores {
            seen.entry(score.id.clone())
                .and_modify(|existing| {
                    if score.score > existing.score {
                        *existing = score.clone();
                    }
                })
                .or_insert_with(|| score.clone());
        }

        let mut final_scores: Vec<FusionScore> = seen.into_values().collect();
        final_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        let total = final_scores.len();
        final_scores.truncate(top_k as usize);

        FusionResult {
            total,
            scores: final_scores,
        }
    }
}

impl Default for ResultFusion {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-unified-query fusion::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/unified-query/src/fusion.rs
git commit -m "feat(unified-query): add ResultFusion with unified scoring"
```

---

## Task 10: Implement UnifiedQueryEngine

**Files:**
- Create: `crates/unified-query/src/engine.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_execute() {
        let engine = UnifiedQueryEngine::new();
        let request = UnifiedQueryRequest {
            query: "test".to_string(),
            mode: QueryMode::SQL,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: Some(10),
            offset: Some(0),
        };
        
        let response = engine.execute(request).await;
        assert!(response.query_plan.mode.contains("sql"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-unified-query engine::tests`
Expected: FAIL

**Step 3: Write minimal implementation**

```rust
use crate::api::{UnifiedQueryRequest, UnifiedQueryResponse};
use crate::cache::QueryCache;
use crate::executor::ParallelExecutor;
use crate::fusion::ResultFusion;
use crate::router::QueryRouter;
use crate::stats::QueryStats;
use std::sync::Arc;

pub struct UnifiedQueryEngine {
    router: QueryRouter,
    executor: ParallelExecutor,
    fusion: ResultFusion,
    cache: Arc<QueryCache>,
    stats: Arc<QueryStats>,
}

impl UnifiedQueryEngine {
    pub fn new() -> Self {
        Self {
            router: QueryRouter::new(),
            executor: ParallelExecutor::new(),
            fusion: ResultFusion::new(),
            cache: Arc::new(QueryCache::new(1000)),
            stats: Arc::new(QueryStats::default()),
        }
    }

    pub async fn execute(&self, request: UnifiedQueryRequest) -> UnifiedQueryResponse {
        let plan = self.router.route(&request).expect("Invalid request");
        
        // Execute in parallel
        let results = self.executor.execute(&request, &plan).await;
        
        // Fuse results
        let fusion_result = self.fusion.fuse(results, &plan.weights, plan.top_k);
        
        UnifiedQueryResponse {
            sql_results: None,
            vector_results: None,
            graph_results: None,
            fusion_scores: fusion_result.scores,
            total: fusion_result.total as u64,
            execution_time_ms: 0,
            query_plan: crate::api::QueryPlanDetail {
                mode: format!("{:?}", request.mode),
                weights: plan.weights,
                steps: vec![],
                fusion_time_ms: 0,
                total_time_ms: 0,
            },
        }
    }
}

impl Default for UnifiedQueryEngine {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-unified-query engine::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/unified-query/src/engine.rs
git commit -m "feat(unified-query): add UnifiedQueryEngine entry point"
```

---

## Task 11: Update lib.rs exports

**Files:**
- Modify: `crates/unified-query/src/lib.rs`

**Step 1: Update lib.rs**

```rust
pub mod api;
pub mod cache;
pub mod engine;
pub mod error;
pub mod executor;
pub mod fusion;
pub mod router;
pub mod stats;
pub mod adapters;

pub use api::{
    FusionScore, GraphQuery, GraphResult, QueryMode, QueryPlanDetail, QueryStep,
    QueryStatsResponse, UnifiedQueryRequest, UnifiedQueryResponse, VectorQuery, VectorResult,
    Weights,
};
pub use cache::QueryCache;
pub use engine::UnifiedQueryEngine;
pub use error::{QueryResult, UnifiedQueryError};
pub use executor::ParallelExecutor;
pub use fusion::ResultFusion;
pub use router::QueryRouter;
pub use stats::{QueryStats, QueryStatsBuilder};
```

**Step 2: Run build verification**

Run: `cargo build -p sqlrustgo-unified-query`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add crates/unified-query/src/lib.rs
git commit -m "feat(unified-query): export public API"
```

---

## Task 12: Add to workspace

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add to workspace members**

Add `"crates/unified-query"` to the workspace members list.

**Step 2: Run build verification**

Run: `cargo build --workspace`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "feat(unified-query): add to workspace"
```

---

## Task 13: Add integration test

**Files:**
- Create: `tests/integration/unified_query_test.rs`

**Step 1: Write integration test**

```rust
#[cfg(test)]
mod tests {
    use sqlrustgo_unified_query::*;

    #[tokio::test]
    async fn test_unified_query_integration() {
        let engine = UnifiedQueryEngine::new();
        
        let request = UnifiedQueryRequest {
            query: "test query".to_string(),
            mode: QueryMode::SQL,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: Some(10),
            offset: Some(0),
        };
        
        let response = engine.execute(request).await;
        assert_eq!(response.query_plan.mode, "SQL");
    }
}
```

**Step 2: Run test**

Run: `cargo test --test unified_query_test`
Expected: Test runs (may fail due to adapters not being fully implemented)

**Step 3: Commit**

```bash
git add tests/integration/unified_query_test.rs
git commit -m "test(unified-query): add integration test"
```

---

## Summary

**Files to Create:**
- `crates/unified-query/Cargo.toml`
- `crates/unified-query/src/lib.rs`
- `crates/unified-query/src/error.rs`
- `crates/unified-query/src/api.rs`
- `crates/unified-query/src/stats.rs`
- `crates/unified-query/src/cache.rs`
- `crates/unified-query/src/router.rs`
- `crates/unified-query/src/adapters/mod.rs`
- `crates/unified-query/src/adapters/storage.rs`
- `crates/unified-query/src/adapters/vector.rs`
- `crates/unified-query/src/adapters/graph.rs`
- `crates/unified-query/src/executor.rs`
- `crates/unified-query/src/fusion.rs`
- `crates/unified-query/src/engine.rs`
- `tests/integration/unified_query_test.rs`

**Files to Modify:**
- `Cargo.toml` (workspace)
- `docs/plans/2026-04-09-unified-query-api-design.md` (already created)

**Total Commits:** 13 tasks × 1 commit = ~13 commits
