# Issue #1345: Unified Query Integration Test Design

**Date**: 2026-04-16
**Issue**: #1345 - 全模块集成测试 - SQL+Vector+Graph混合负载
**Status**: Approved

## 1. Overview

创建 `tests/integration/unified_query_integration_test.rs`，测试 SQL + Vector + Graph 混合负载的集成功能。

## 2. Architecture

```
UnifiedQueryEngine (真实引擎)
├── StorageAdapter (SQL)
├── VectorAdapter (Vector)
└── GraphAdapter (Graph)
```

**测试文件位置**: `tests/integration/unified_query_integration_test.rs`

## 3. Test Categories

### T1: SQL + Vector 联合查询
- `test_sql_vector_basic_search` - SQL 预过滤 + 向量搜索
- `test_sql_filtered_vector_rerank` - SQL 结果重排列向量
- `test_sql_vector_weight_variations` - 不同权重配置

### T2: SQL + Graph 联合查询
- `test_sql_graph_entity_lookup` - SQL 实体查询 + 图扩展
- `test_sql_graph_traversal_expansion` - SQL 起点 + 图遍历
- `test_sql_graph_path_finding` - 路径查找

### T3: Vector + Graph 联合查询
- `test_vector_graph_hybrid_enrichment` - 向量结果用图关系增强
- `test_vector_graph_cross_reference` - 交叉引用

### T4: SQL + Vector + Graph 三模块联合
- `test_sql_vector_graph_unified_search` - 三模块同时查询
- `test_sql_vector_graph_weighted_fusion` - 加权融合
- `test_sql_vector_graph_complex_query` - 复杂查询

### T5: 高级功能测试
- `test_concurrent_hybrid_queries` - 并发查询
- `test_filter_combinations` - 过滤器组合
- `test_different_weight_configurations` - 权重配置

### T6: 边界情况和错误处理
- `test_empty_sql_results` - SQL 空结果
- `test_empty_vector_results` - 向量空结果
- `test_partial_module_failure` - 部分模块失败
- `test_large_result_set_truncation` - 大结果集截断

### T7: 性能基准
- `test_performance_vs_pure_sql` - vs 纯 SQL
- `test_performance_vs_pure_vector` - vs 纯向量
- `test_performance_vs_pure_graph` - vs 纯图

## 4. Technical Implementation

### 依赖
```rust
use sqlrustgo_unified_query::{UnifiedQueryEngine, QueryMode, ...};
use sqlrustgo_vector::{FlatIndex, DistanceMetric, VectorIndex, ...};
use sqlrustgo_graph::{GraphStore, Node, Edge, ...};
```

### 测试工具
- 向量生成: `generate_test_vectors(count, dimension)` - 128维随机向量
- 图构建: `build_test_graph()` - 内存测试图
- 引擎: `UnifiedQueryEngine::new()`

### 测试框架
- 使用 `tokio::test` 异步测试
- 遵循 `hybrid_search_integration_test.rs` 模式

## 5. Acceptance Criteria

1. 所有测试用例在 `cargo test` 中通过
2. 测试覆盖 A (基础), B (高级), C (边界) 所有场景
3. 测试集成到回归测试框架
4. 提交 PR 到 develop/v2.5.0

## 6. File Structure

```
tests/integration/unified_query_integration_test.rs
```

包含 7 个测试组，共约 20 个测试用例。