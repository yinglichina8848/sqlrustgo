# 统一查询 API 设计文档

**版本**: v2.5.0
**Issue**: #1326

---

## 概述

统一查询 API 实现 SQL + 向量 + 图的融合查询，支持并行执行和结果评分融合。

## 架构

### 模块结构

```
┌─────────────────────────────────────────────────────────┐
│              UnifiedQueryEngine                      │
│  - QueryRouter: 根据请求路由                         │
│  - ParallelExecutor: 并行执行子查询                  │
│  - ResultFusion: 结果评分融合                       │
│  - QueryCache: 查询缓存                             │
│  - QueryStats: 统计信息                            │
└─────────────────────────────────────────────────────────┘
```

### 核心类型

```rust
// 查询模式
pub enum QueryMode {
    SqlOnly,
    VectorOnly,
    GraphOnly,
    SqlVector,    // SQL + 向量
    SqlGraph,     // SQL + 图
    VectorGraph,  // 向量 + 图
    Unified,     // SQL + 向量 + 图
}

// 执行计划
pub struct QueryPlan {
    pub execute_sql: bool,
    pub execute_vector: bool,
    pub execute_graph: bool,
    pub weights: Weights,
    pub top_k: usize,
    pub offset: usize,
}

// 并行执行结果
pub struct ParallelQueryResults {
    pub sql_results: Option<SqlQueryResult>,
    pub vector_results: Option<VectorQueryResult>,
    pub graph_results: Option<GraphQueryResult>,
}

// 融合结果
pub struct FusionResult {
    pub scores: Vec<FusionScore>,
    pub total: usize,
}
```

## 执行流程

### 1. 路由 (Route)

```rust
impl QueryRouter {
    pub fn route(&self, request: &UnifiedQueryRequest) -> QueryPlan {
        QueryPlan {
            execute_sql: request.mode.contains("sql"),
            execute_vector: request.mode.contains("vector"),
            execute_graph: request.mode.contains("graph"),
            weights: request.weights.clone(),
            top_k: request.top_k,
            offset: request.offset,
        }
    }
}
```

### 2. 并行执行 (Parallel Execution)

```rust
impl ParallelExecutor {
    pub async fn execute(
        &self,
        request: &UnifiedQueryRequest,
        plan: &QueryPlan,
    ) -> ParallelQueryResults {
        // 根据计划并行执行各子查询
        let sql_task = async {
            if plan.execute_sql {
                Some(self.execute_sql(&request).await?)
            } else {
                None
            }
        };
        let vector_task = async {
            if plan.execute_vector {
                Some(self.execute_vector(&request).await?)
            } else {
                None
            }
        };
        let graph_task = async {
            if plan.execute_graph {
                Some(self.execute_graph(&request).await?)
            } else {
                None
            }
        };

        // 并行等待所有结果
        let (sql, vector, graph) = tokio::join!(sql_task, vector_task, graph_task);

        ParallelQueryResults {
            sql_results: sql,
            vector_results: vector,
            graph_results: graph,
        }
    }
}
```

### 3. 结果融合 (Result Fusion)

```rust
impl ResultFusion {
    pub fn fuse(
        &self,
        results: &ParallelQueryResults,
        weights: &Weights,
        top_k: usize,
    ) -> FusionResult {
        let mut all_scores = Vec::new();

        // SQL 结果加权
        if let Some(ref sql) = results.sql_results {
            for item in &sql.items {
                all_scores.push(FusionScore {
                    id: item.id.clone(),
                    score: item.score * weights.sql_weight,
                    source: "sql".to_string(),
                });
            }
        }

        // 向量结果加权
        if let Some(ref vector) = results.vector_results {
            for item in &vector.items {
                all_scores.push(FusionScore {
                    id: item.id.clone(),
                    score: item.distance * weights.vector_weight,
                    source: "vector".to_string(),
                });
            }
        }

        // 图结果加权
        if let Some(ref graph) = results.graph_results {
            for item in &graph.items {
                all_scores.push(FusionScore {
                    id: item.id.clone(),
                    score: item.score * weights.graph_weight,
                    source: "graph".to_string(),
                });
            }
        }

        // 按 ID 去重
        let mut id_map: HashMap<String, f32> = HashMap::new();
        for fs in &all_scores {
            let entry = id_map.entry(fs.id.clone()).or_insert(0.0);
            *entry = (*entry).max(fs.score);
        }

        // 转为向量并排序
        let mut scores: Vec<FusionScore> = id_map
            .into_iter()
            .map(|(id, score)| FusionScore { id, score, source: "fused".to_string() })
            .collect();
        scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scores.truncate(top_k);

        FusionResult {
            total: scores.len(),
            scores,
        }
    }
}
```

## HTTP API

### 统一查询端点

```bash
POST /api/v2/query/unified
{
    "query": "推荐技术相关文章",
    "mode": "SqlVector",
    "weights": {
        "sql_weight": 0.3,
        "vector_weight": 0.7,
        "graph_weight": 0.0
    },
    "top_k": 10
}
```

### 向量查询端点

```bash
POST /api/v2/query/vector
{
    "table": "documents",
    "column": "embedding",
    "query": [0.1, 0.2, ...],
    "k": 10
}
```

### 图查询端点

```bash
POST /api/v2/query/graph
{
    "query": "MATCH (p:Person)-[:KNOWS]->(f) RETURN f"
}
```

---

## 配置

```rust
pub struct UnifiedQueryConfig {
    pub default_weights: Weights,
    pub default_top_k: usize,
    pub cache_enabled: bool,
    pub parallel_execution: bool,
    pub thread_pool_size: usize,
}
```

---

## 性能

| 模式 | 执行时间 | 备注 |
|------|----------|------|
| SqlOnly | ~50ms | 单 SQL 执行 |
| VectorOnly | ~30ms | 单向量搜索 |
| SqlVector | ~60ms | 并行执行 |
| Unified | ~80ms | 三路并行 |

---

*文档版本: 1.0*
*最后更新: 2026-04-16*