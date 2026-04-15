# 图查询性能基准设计

> 日期: 2026-04-16
> Issue: #1344
> 状态: Approved

## 1. 概述

实现图查询性能基准测试套件，验证 BFS/DFS/多跳查询达到目标性能：
- BFS 1000 节点 < 50ms
- DFS 1000 节点 < 100ms
- 3-hop 查询 < 500ms

## 2. 架构

```
crates/graph/
├── benches/
│   └── graph_benchmark.rs    # 基准测试入口
├── src/
│   ├── graph_generator.rs    # 确定性伪随机图生成器 (新增)
│   └── traversal/
│       ├── bfs.rs           # 已有 BFS
│       ├── dfs.rs           # 已有 DFS
│       └── multi_hop.rs     # 多跳查询 (新增)
```

## 3. 图生成器 (GraphGenerator)

### 3.1 算法选择

使用 **Barabási-Albert 无标度网络生成器**：
- 幂律度分布，符合真实图数据特征
- 确定性输出，可通过 seed 复现

### 3.2 接口

```rust
pub struct GraphGenerator {
    seed: u64,
}

impl GraphGenerator {
    pub fn new(seed: u64) -> Self;
    pub fn generate(&self, node_count: usize, edges_per_node: usize) -> InMemoryGraphStore;
}
```

## 4. 多跳查询 (Multi-Hop)

### 4.1 接口

```rust
pub fn multi_hop<G>(graph: &G, start: NodeId, depth: usize) -> Vec<NodeId>
where
    G: Fn(NodeId) -> Vec<NodeId>;
```

### 4.2 行为

- 2-hop: A → B → C (收集所有 B 再到 C)
- 3-hop: A → B → C → D
- 以此类推

## 5. 基准测试结构

### 5.1 测试用例

| 测试类型 | 节点规模 | 迭代次数 |
|---------|---------|---------|
| BFS | 100/1000/10000 | 100 |
| DFS | 100/1000/10000 | 100 |
| 2-hop | 100/1000/10000 | 100 |
| 3-hop | 100/1000/10000 | 100 |
| 4-hop | 100/1000/10000 | 100 |

### 5.2 输出格式

```
Graph Benchmark Results
======================
BFS_100    avg: 1.23ms  p95: 1.89ms  p99: 2.12ms  QPS: 8120
BFS_1000   avg: 12.34ms p95: 18.90ms p99: 25.12ms QPS: 810
BFS_10000  avg: 89.23ms p95: 120.45ms p99: 150.78ms QPS: 112

DFS_100    avg: 1.45ms  p95: 2.01ms  p99: 2.34ms  QPS: 6890
...

Report saved to: benchmark_results/graph_benchmark_20260416.json
```

### 5.3 指标定义

- **avg**: 平均耗时 (ms)
- **p95**: 第 95 百分位耗时 (ms)
- **p99**: 第 99 百分位耗时 (ms)
- **QPS**: 每秒查询数

## 6. 实现计划

### Task 1: 实现 GraphGenerator
- 文件: `crates/graph/src/graph_generator.rs`
- 实现 Barabási-Albert 算法
- 返回 `InMemoryGraphStore`

### Task 2: 实现 multi_hop 查询
- 文件: `crates/graph/src/traversal/multi_hop.rs`
- 支持任意深度多跳查询

### Task 3: 实现基准测试
- 文件: `crates/graph/benches/graph_benchmark.rs`
- 使用 `Bencher` 接口
- 输出表格和 JSON 报告

### Task 4: 集成测试
- 运行基准测试验证性能目标
- 如未达标，分析瓶颈并优化

## 7. 性能目标

| 指标 | 目标 | 验收标准 |
|------|------|---------|
| BFS 1000 节点 | < 50ms | avg < 50ms |
| DFS 1000 节点 | < 100ms | avg < 100ms |
| 3-hop 1000 节点 | < 500ms | avg < 500ms |

## 8. 依赖

- `crates/graph`: 现有图引擎
- Rust `std::collections`: HashMap, HashSet
- 无新增外部依赖
