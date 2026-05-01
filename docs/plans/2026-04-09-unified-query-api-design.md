# Unified Query API Design

**Issue**: #1337  
**Date**: 2026-04-09  
**Status**: Approved

## Overview

Implement a unified query API that supports SQL + Vector + Graph hybrid queries for v2.5.0. The API provides parallel execution of multiple query engines with unified scoring and result fusion.

## Architecture

```
User Request
    │
    ▼
┌─────────────────────────────────────┐
│  UnifiedQueryEngine                 │
│  ┌─────────────────────────────┐   │
│  │  QueryRouter                │   │
│  │  Parse mode parameter       │   │
│  └──────────┬──────────────────┘   │
│             │                        │
│  ┌─────────▼─────────┐              │
│  │ ParallelExecutor   │              │
│  │ tokio::join!      │              │
│  └─────────┬─────────┘              │
└────────────┼────────────────────────┘
             │
    ┌────────┴────────┬────────┐
    ▼                 ▼        ▼
 SQL Engine    VectorEngine  GraphEngine
 (storage)     (vector)     (graph)
    │                 │        │
    └────────┬────────┴────────┘
             ▼
┌─────────────────────────┐
│  ResultFusion          │
│  Unified scoring       │
│  Deduplication         │
│  Pagination            │
└─────────────────────────┘
```

## API Design

### Endpoint: `POST /api/v2/query/unified`

**Request**:
```json
{
  "query": "查找批准GMP清洁验证SOP的设备",
  "mode": "sql_vector_graph",
  "filters": {
    "date_range": {"start": "2026-01-01", "end": "2026-12-31"},
    "department": "生产部"
  },
  "weights": {
    "sql": 0.4,
    "vector": 0.3,
    "graph": 0.3
  },
  "vector_query": {
    "column": "embedding",
    "top_k": 10
  },
  "graph_query": {
    "start_nodes": ["equipment_001"],
    "traversal": "BFS",
    "max_depth": 3
  },
  "top_k": 10,
  "offset": 0
}
```

**Response**:
```json
{
  "sql_results": [...],
  "vector_results": [...],
  "graph_results": [...],
  "fusion_scores": [...],
  "total": 100,
  "execution_time_ms": 125,
  "query_plan": {
    "mode": "sql_vector_graph",
    "weights": {"sql": 0.4, "vector": 0.3, "graph": 0.3},
    "steps": [
      {
        "source": "sql",
        "step": "IndexScan on idx_equipment",
        "time_ms": 12,
        "rows_affected": 50
      },
      {
        "source": "vector",
        "step": "HNSW Search top_k=10",
        "time_ms": 8,
        "nodes_visited": 150
      },
      {
        "source": "graph",
        "step": "BFS depth=3 from [equipment_001]",
        "time_ms": 5,
        "nodes_traversed": 25
      }
    ],
    "fusion_time_ms": 2,
    "total_time_ms": 27
  }
}
```

### Query Modes

| Mode | Description |
|------|-------------|
| `sql` | SQL only query |
| `vector` | Vector only search |
| `graph` | Graph only traversal |
| `sql_vector` | SQL + Vector hybrid |
| `sql_graph` | SQL + Graph hybrid |
| `vector_graph` | Vector + Graph hybrid |
| `sql_vector_graph` | All three engines |

## Core Components

### 1. UnifiedQueryEngine

Entry point that routes and coordinates query execution.

```rust
pub struct UnifiedQueryEngine {
    storage: Arc<RwLock<dyn StorageEngine>>,
    vector_store: Arc<VectorStore>,
    graph_store: Arc<GraphStore>,
    cache: Arc<QueryCache>,
    stats: Arc<QueryStats>,
}
```

### 2. QueryRouter

Parses mode parameter and distributes tasks.

```rust
pub struct QueryRouter;

impl QueryRouter {
    pub fn route(&self, request: &UnifiedQueryRequest) -> QueryPlan;
    pub fn validate_request(&self, request: &UnifiedQueryRequest) -> Result<()>;
}
```

### 3. ParallelExecutor

Executes SQL/Vector/Graph in parallel using `tokio::join!`.

```rust
pub struct ParallelExecutor;

impl ParallelExecutor {
    pub async fn execute(
        &self,
        plan: &QueryPlan,
    ) -> ParallelQueryResults {
        let (sql_result, vector_result, graph_result) = tokio::join!(
            self.execute_sql(plan),
            self.execute_vector(plan),
            self.execute_graph(plan)
        );
        // ...
    }
}
```

### 4. ResultFusion

Unified scoring, deduplication, and pagination.

```rust
pub struct ResultFusion;

impl ResultFusion {
    // score = w_sql * sql_normalized_score 
    //       + w_vector * vector_normalized_score 
    //       + w_graph * graph_normalized_score
    pub fn fuse(
        &self,
        results: ParallelQueryResults,
        weights: &Weights,
        top_k: u32,
    ) -> FusionResult;
    
    fn normalize_scores(&self, results: &mut ParallelQueryResults);
    fn deduplicate(&self, results: &mut FusionResult);
}
```

### 5. QueryCache

LRU cache for query results.

```rust
pub struct QueryCache {
    cache: Arc<RwLock<LruCache<String, FusionResult>>>,
    capacity: usize,
}

impl QueryCache {
    pub fn get(&self, key: &str) -> Option<FusionResult>;
    pub fn insert(&self, key: String, result: FusionResult);
}
```

### 6. QueryStats

Records execution statistics.

```rust
pub struct QueryStats {
    query_id: Uuid,
    total_time_ms: u64,
    sql_time_ms: u64,
    vector_time_ms: u64,
    graph_time_ms: u64,
    sql_hit: bool,
    vector_hit: bool,
    graph_hit: bool,
    cache_hit: bool,
}
```

## Error Handling

Each adapter returns `QueryResult<T>` for error isolation:

```rust
pub enum QueryResult<T> {
    Ok(T),
    Partial(Vec<String>),  // Partial success with warnings
    Err(String),           // Failure but doesn't block other queries
}

impl<T> QueryResult<T> {
    pub fn is_ok(&self) -> bool;
    pub fn is_partial(&self) -> bool;
    pub fn unwrap_or_default(&self) -> Option<T>;
}
```

## Directory Structure

```
crates/unified-query/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── engine.rs           # UnifiedQueryEngine
    ├── router.rs           # QueryRouter
    ├── executor.rs         # ParallelExecutor
    ├── fusion.rs           # ResultFusion
    ├── cache.rs            # QueryCache
    ├── stats.rs            # QueryStats
    ├── adapters/
    │   ├── mod.rs
    │   ├── storage.rs     # StorageAdapter
    │   ├── vector.rs       # VectorAdapter
    │   └── graph.rs        # GraphAdapter
    ├── api.rs              # API types
    └── error.rs            # QueryResult<T>
```

## Dependencies

```toml
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
```

## Testing

1. Unit tests for each component
2. Integration tests for API endpoints
3. Performance benchmarks for ParallelExecutor
4. Cache hit rate tests

## Acceptance Criteria

- [ ] `POST /api/v2/query/unified` endpoint works
- [ ] All 7 query modes function correctly
- [ ] Parallel execution with `tokio::join!`
- [ ] Unified scoring with configurable weights
- [ ] Query caching with LRU policy
- [ ] Execution statistics recorded
- [ ] Error isolation in adapters
- [ ] Integration tests pass
