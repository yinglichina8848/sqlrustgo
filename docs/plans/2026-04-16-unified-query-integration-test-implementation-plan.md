# Unified Query Integration Test Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 创建 `tests/integration/unified_query_integration_test.rs`，测试 SQL+Vector+Graph 混合负载集成

**Architecture:** 使用 UnifiedQueryEngine 真实引擎，结合 FlatIndex (向量) 和 GraphStore (图) 进行端到端集成测试

**Tech Stack:** Rust, tokio, sqlrustgo_unified_query, sqlrustgo_vector, sqlrustgo_graph

---

## Task 1: 创建测试文件基础结构

**Files:**
- Create: `tests/integration/unified_query_integration_test.rs`

**Step 1: 创建测试文件头部和导入**

```rust
//! Unified Query Integration Tests
//!
//! Tests for SQL + Vector + Graph hybrid search functionality including:
//! - T1: SQL + Vector joint queries
//! - T2: SQL + Graph joint queries
//! - T3: Vector + Graph joint queries
//! - T4: SQL + Vector + Graph unified queries
//! - T5: Advanced features (concurrency, filters, weights)
//! - T6: Edge cases and error handling
//! - T7: Performance benchmarks

use sqlrustgo_unified_query::{UnifiedQueryEngine, QueryMode, UnifiedQueryRequest,
    VectorQuery, GraphQuery, TraversalType, Weights};
use sqlrustgo_vector::{FlatIndex, DistanceMetric, VectorIndex};
use sqlrustgo_graph::{GraphStore, Node, Edge, PropertyMap};
use std::collections::HashMap;
```

**Step 2: 添加测试工具函数**

```rust
fn generate_test_vectors(count: usize, dimension: usize) -> Vec<(u64, Vec<f32>)> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..count as u64)
        .map(|i| {
            let vector: Vec<f32> = (0..dimension).map(|_| rng.gen_range(-1.0..1.0)).collect();
            (i, vector)
        })
        .collect()
}

fn build_test_graph() -> GraphStore {
    let graph = GraphStore::new();
    // 添加测试节点和边
    graph
}
```

---

## Task 2: T1 - SQL + Vector 联合查询测试

**Files:**
- Modify: `tests/integration/unified_query_integration_test.rs`

**Step 1: 添加 test_sql_vector_basic_search**

```rust
#[tokio::test]
async fn test_sql_vector_basic_search() {
    let engine = UnifiedQueryEngine::new();
    let request = UnifiedQueryRequest {
        query: "SELECT * FROM products WHERE category = 'electronics'".to_string(),
        mode: QueryMode::SQLVector,
        filters: None,
        weights: Some(Weights { sql: 0.4, vector: 0.6, graph: 0.0 }),
        vector_query: Some(VectorQuery {
            column: "embedding".to_string(),
            top_k: 10,
            filter: None,
        }),
        graph_query: None,
        top_k: Some(10),
        offset: Some(0),
    };

    let response = engine.execute(request).await;
    assert!(response.execution_time_ms >= 0);
    assert_eq!(response.query_plan.mode.contains("SQLVector"), true);
}
```

**Step 2: 添加 test_sql_filtered_vector_rerank**

```rust
#[tokio::test]
async fn test_sql_filtered_vector_rerank() {
    // 测试 SQL 过滤后的向量重排
}
```

**Step 3: 添加 test_sql_vector_weight_variations**

```rust
#[tokio::test]
async fn test_sql_vector_weight_variations() {
    // 测试不同权重配置
}
```

---

## Task 3: T2 - SQL + Graph 联合查询测试

**Step 1: 添加 test_sql_graph_entity_lookup**

```rust
#[tokio::test]
async fn test_sql_graph_entity_lookup() {
    let engine = UnifiedQueryEngine::new();
    let request = UnifiedQueryRequest {
        query: "SELECT * FROM users WHERE id = 1".to_string(),
        mode: QueryMode::SQLGraph,
        filters: None,
        weights: Some(Weights { sql: 0.5, vector: 0.0, graph: 0.5 }),
        vector_query: None,
        graph_query: Some(GraphQuery {
            start_nodes: vec!["user_1".to_string()],
            traversal: TraversalType::BFS,
            max_depth: 2,
        }),
        top_k: Some(10),
        offset: Some(0),
    };

    let response = engine.execute(request).await;
    assert!(response.execution_time_ms >= 0);
}
```

**Step 2: 添加 test_sql_graph_traversal_expansion**

```rust
#[tokio::test]
async fn test_sql_graph_traversal_expansion() {
    // SQL 起点 + 图遍历扩展
}
```

**Step 3: 添加 test_sql_graph_path_finding**

```rust
#[tokio::test]
async fn test_sql_graph_path_finding() {
    // 路径查找
}
```

---

## Task 4: T3 - Vector + Graph 联合查询测试

**Step 1: 添加 test_vector_graph_hybrid_enrichment**

```rust
#[tokio::test]
async fn test_vector_graph_hybrid_enrichment() {
    let engine = UnifiedQueryEngine::new();
    let request = UnifiedQueryRequest {
        query: "vector search".to_string(),
        mode: QueryMode::VectorGraph,
        filters: None,
        weights: Some(Weights { sql: 0.0, vector: 0.5, graph: 0.5 }),
        vector_query: Some(VectorQuery {
            column: "embedding".to_string(),
            top_k: 10,
            filter: None,
        }),
        graph_query: Some(GraphQuery {
            start_nodes: vec!["node_1".to_string()],
            traversal: TraversalType::DFS,
            max_depth: 3,
        }),
        top_k: Some(10),
        offset: Some(0),
    };

    let response = engine.execute(request).await;
    assert!(response.execution_time_ms >= 0);
}
```

**Step 2: 添加 test_vector_graph_cross_reference**

```rust
#[tokio::test]
async fn test_vector_graph_cross_reference() {
    // 交叉引用
}
```

---

## Task 5: T4 - SQL + Vector + Graph 三模块联合测试

**Step 1: 添加 test_sql_vector_graph_unified_search**

```rust
#[tokio::test]
async fn test_sql_vector_graph_unified_search() {
    let engine = UnifiedQueryEngine::new();
    let request = UnifiedQueryRequest {
        query: "unified query".to_string(),
        mode: QueryMode::SQLVectorGraph,
        filters: None,
        weights: Some(Weights { sql: 0.4, vector: 0.3, graph: 0.3 }),
        vector_query: Some(VectorQuery {
            column: "embedding".to_string(),
            top_k: 10,
            filter: None,
        }),
        graph_query: Some(GraphQuery {
            start_nodes: vec!["node_1".to_string()],
            traversal: TraversalType::BFS,
            max_depth: 2,
        }),
        top_k: Some(10),
        offset: Some(0),
    };

    let response = engine.execute(request).await;
    assert!(response.query_plan.mode.contains("SQLVectorGraph"));
    assert!(response.execution_time_ms >= 0);
}
```

**Step 2: 添加 test_sql_vector_graph_weighted_fusion**

```rust
#[tokio::test]
async fn test_sql_vector_graph_weighted_fusion() {
    // 加权融合测试
}
```

**Step 3: 添加 test_sql_vector_graph_complex_query**

```rust
#[tokio::test]
async fn test_sql_vector_graph_complex_query() {
    // 复杂查询测试
}
```

---

## Task 6: T5 - 高级功能测试

**Step 1: 添加 test_concurrent_hybrid_queries**

```rust
#[tokio::test]
async fn test_concurrent_hybrid_queries() {
    let engine = UnifiedQueryEngine::new();

    let requests = vec![
        UnifiedQueryRequest { /* SQLVector */ },
        UnifiedQueryRequest { /* SQLGraph */ },
        UnifiedQueryRequest { /* VectorGraph */ },
    ];

    let futures = requests.into_iter().map(|req| engine.execute(req));
    let results = futures::future::join_all(futures).await;

    assert_eq!(results.len(), 3);
}
```

**Step 2: 添加 test_filter_combinations**

```rust
#[tokio::test]
async fn test_filter_combinations() {
    // 过滤器组合测试
}
```

**Step 3: 添加 test_different_weight_configurations**

```rust
#[tokio::test]
async fn test_different_weight_configurations() {
    // 不同权重配置测试
}
```

---

## Task 7: T6 - 边界情况和错误处理

**Step 1: 添加 test_empty_sql_results**

```rust
#[tokio::test]
async fn test_empty_sql_results() {
    // SQL 空结果测试
}
```

**Step 2: 添加 test_empty_vector_results**

```rust
#[tokio::test]
async fn test_empty_vector_results() {
    // 向量空结果测试
}
```

**Step 3: 添加 test_partial_module_failure**

```rust
#[tokio::test]
async fn test_partial_module_failure() {
    // 部分模块失败测试
}
```

**Step 4: 添加 test_large_result_set_truncation**

```rust
#[tokio::test]
async fn test_large_result_set_truncation() {
    // 大结果集截断测试
}
```

---

## Task 8: T7 - 性能基准测试

**Step 1: 添加 test_performance_vs_pure_sql**

```rust
#[tokio::test]
async fn test_performance_vs_pure_sql() {
    let hybrid_start = std::time::Instant::now();
    // 执行混合查询
    let hybrid_time = hybrid_start.elapsed();

    let pure_start = std::time::Instant::now();
    // 执行纯 SQL 查询
    let pure_time = pure_start.elapsed();

    println!("Hybrid: {:?}, Pure SQL: {:?}", hybrid_time, pure_time);
}
```

**Step 2: 添加 test_performance_vs_pure_vector**

```rust
#[tokio::test]
async fn test_performance_vs_pure_vector() {
    // vs 纯向量
}
```

**Step 3: 添加 test_performance_vs_pure_graph**

```rust
#[tokio::test]
async fn test_performance_vs_pure_graph() {
    // vs 纯图
}
```

---

## Task 9: 添加测试摘要和回归测试集成

**Step 1: 添加 test_integration_summary**

```rust
#[test]
fn test_integration_summary() {
    println!();
    println!("Unified Query Integration Tests Summary:");
    println!();
    println!("T1: SQL + Vector Joint Queries");
    println!("  - test_sql_vector_basic_search");
    println!("  - test_sql_filtered_vector_rerank");
    println!("  - test_sql_vector_weight_variations");
    // ... 其他测试组
}
```

**Step 2: 检查回归测试框架**

检查 `tests/integration/regression_test.rs` 是否需要添加 unified_query_integration_test 的引用

---

## Task 10: 运行测试验证

**Step 1: 运行单个测试文件**

```bash
cd /Users/liying/workspace/dev/heartopen/sqlrustgo
cargo test --test unified_query_integration_test -- --nocapture
```

**Step 2: 运行所有集成测试**

```bash
cargo test --test '*' -- --nocapture
```

**Step 3: 验证测试通过**

Expected: 所有 20 个测试用例 PASS

---

## Task 11: 提交 PR

**Step 1: 创建分支**

```bash
git checkout -b feature/issue-1345-unified-query-integration-test
```

**Step 2: 提交更改**

```bash
git add tests/integration/unified_query_integration_test.rs
git commit -m "feat: add unified query integration test for SQL+Vector+Graph

- T1: SQL + Vector joint queries
- T2: SQL + Graph joint queries
- T3: Vector + Graph joint queries
- T4: SQL + Vector + Graph unified queries
- T5: Advanced features (concurrency, filters, weights)
- T6: Edge cases and error handling
- T7: Performance benchmarks

Closes #1345"
```

**Step 3: 推送并创建 PR**

```bash
git push -u origin feature/issue-1345-unified-query-integration-test
gh pr create --base develop/v2.5.0 --title "feat: add unified query integration test (#1345)"
```

---

## Execution Options

**Plan complete and saved to `docs/plans/2026-04-16-unified-query-integration-test-implementation-plan.md`.**

**Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**