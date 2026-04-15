# 图查询性能基准实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现图查询性能基准测试套件，支持 BFS/DFS/多跳查询的性能测试

**Architecture:**
- 使用 Barabási-Albert 无标度网络生成测试图数据
- 基于现有 InMemoryGraphStore 实现基准测试
- 输出表格和 JSON 格式报告

**Tech Stack:** Rust, cargo bench, serde_json

---

## 文件结构

```
crates/graph/
├── benches/
│   └── graph_benchmark.rs    # 基准测试入口 (创建)
├── src/
│   ├── graph_generator.rs    # 图生成器 (创建)
│   └── traversal/
│       ├── multi_hop.rs     # 多跳查询 (创建)
```

---

## Task 1: 实现 GraphGenerator

**Files:**
- Create: `crates/graph/src/graph_generator.rs`
- Modify: `crates/graph/src/lib.rs` (导出新模块)
- Modify: `crates/graph/Cargo.toml` (如需依赖)

- [ ] **Step 1: 实现 GraphGenerator 结构体**

```rust
use crate::model::NodeId;
use crate::store::InMemoryGraphStore;
use std::collections::HashMap;

/// Barabási-Albert 无标度网络生成器
pub struct GraphGenerator {
    seed: u64,
}

impl GraphGenerator {
    pub fn new(seed: u64) -> Self {
        GraphGenerator { seed }
    }

    /// 生成无标度网络
    /// - node_count: 节点数量
    /// - edges_per_node: 每个新节点添加的边数
    pub fn generate(&self, node_count: usize, edges_per_node: usize) -> InMemoryGraphStore {
        // 实现 Barabási-Albert 算法
        // 1. 从初始连通图开始
        // 2. 新节点优先连接到高度数节点
    }
}
```

- [ ] **Step 2: 运行 cargo check 验证编译**

Run: `cargo check --package sqlrustgo-graph`
Expected: 编译成功

- [ ] **Step 3: 添加单元测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_generator_100_nodes() {
        let gen = GraphGenerator::new(42);
        let store = gen.generate(100, 3);
        assert_eq!(store.node_count(), 100);
    }
}
```

- [ ] **Step 4: 运行测试**

Run: `cargo test --package sqlrustgo-graph graph_generator`
Expected: PASS

- [ ] **Step 5: 提交**

```bash
git add crates/graph/src/graph_generator.rs crates/graph/src/lib.rs
git commit -m "feat(graph): add GraphGenerator with Barabási-Albert algorithm"
```

---

## Task 2: 实现 multi_hop 查询

**Files:**
- Create: `crates/graph/src/traversal/multi_hop.rs`
- Modify: `crates/graph/src/traversal/mod.rs` (导出新模块)

- [ ] **Step 1: 实现 multi_hop 函数**

```rust
use crate::model::NodeId;
use std::collections::HashSet;

/// 执行多跳查询
/// - start: 起始节点
/// - depth: 跳数 (2-hop, 3-hop, etc.)
pub fn multi_hop<G>(graph: G, start: NodeId, depth: usize) -> Vec<NodeId>
where
    G: Fn(NodeId) -> Vec<NodeId>,
{
    let mut current_layer: Vec<NodeId> = vec![start];
    let mut visited: HashSet<NodeId> = HashSet::new();
    visited.insert(start);

    for _ in 0..depth {
        let mut next_layer: Vec<NodeId> = Vec::new();
        for node in current_layer {
            let neighbors = graph(node);
            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    next_layer.push(neighbor);
                }
            }
        }
        current_layer = next_layer;
    }

    current_layer
}
```

- [ ] **Step 2: 在 mod.rs 中导出**

```rust
pub mod multi_hop;
pub use multi_hop::*;
```

- [ ] **Step 3: 添加单元测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2hop_query() {
        // Graph: 1 -> 2 -> 3 -> 4
        let graph = |n: NodeId| -> Vec<NodeId> {
            match n {
                NodeId(1) => vec![NodeId(2)],
                NodeId(2) => vec![NodeId(3)],
                NodeId(3) => vec![NodeId(4)],
                _ => vec![],
            }
        };

        let result = multi_hop(graph, NodeId(1), 2);
        assert!(result.contains(&NodeId(3)));
    }
}
```

- [ ] **Step 4: 运行测试验证**

Run: `cargo test --package sqlrustgo-graph multi_hop`
Expected: PASS

- [ ] **Step 5: 提交**

```bash
git add crates/graph/src/traversal/multi_hop.rs crates/graph/src/traversal/mod.rs
git commit -m "feat(graph): add multi_hop traversal support"
```

---

## Task 3: 实现基准测试

**Files:**
- Create: `crates/graph/benches/graph_benchmark.rs`
- Modify: `crates/graph/Cargo.toml` (添加 bench feature)

- [ ] **Step 1: 检查 Cargo.toml 配置**

确认 `[[bench]]` 部分存在或添加：

```toml
[[bench]]
name = "graph_benchmark"
harness = false
```

- [ ] **Step 2: 实现基准测试入口**

```rust
use sqlrustgo_graph::{bfs_with_distances, dfs_collect, multi_hop, GraphGenerator, InMemoryGraphStore};
use std::time::Instant;

struct BenchmarkResult {
    name: String,
    avg_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    qps: f64,
}

fn run_benchmark<F>(name: &str, iterations: usize, f: F) -> BenchmarkResult
where
    F: Fn() + Copy,
{
    let mut times: Vec<u64> = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        f();
        times.push(start.elapsed().as_nanos() as u64);
    }

    times.sort();
    let avg = times.iter().sum::<u64>() as f64 / iterations as f64;
    let p95 = times[(iterations as f64 * 0.95) as usize] as f64;
    let p99 = times[(iterations as f64 * 0.99) as usize] as f64;
    let qps = 1_000_000_000.0 / avg;

    BenchmarkResult {
        name: name.to_string(),
        avg_ms: avg / 1_000_000.0,
        p95_ms: p95 / 1_000_000.0,
        p99_ms: p99 / 1_000_000.0,
        qps,
    }
}

fn main() {
    println!("Graph Benchmark Results");
    println!("======================\n");

    // 测试用例配置
    let test_cases = vec![
        ("BFS_100", 100, 3),
        ("BFS_1000", 1000, 3),
        ("BFS_10000", 10000, 3),
        ("DFS_100", 100, 3),
        ("DFS_1000", 1000, 3),
        ("DFS_10000", 10000, 3),
        ("2HOP_100", 100, 2),
        ("2HOP_1000", 1000, 2),
        ("3HOP_100", 100, 3),
        ("3HOP_1000", 1000, 3),
        ("4HOP_100", 100, 4),
        ("4HOP_1000", 1000, 4),
    ];

    for (name, nodes, hops) in test_cases {
        let gen = GraphGenerator::new(42);
        let store = gen.generate(nodes, 3);

        let get_neighbors = |n: sqlrustgo_graph::NodeId| -> Vec<sqlrustgo_graph::NodeId> {
            store.neighbors_by_edge_label(n, sqlrustgo_graph::LabelId(0))
        };

        let start_node = sqlrustgo_graph::NodeId(0);

        let result = run_benchmark(name, 100, || {
            let _ = bfs_with_distances(&get_neighbors, start_node);
        });

        println!(
            "{:12} avg: {:7.2}ms  p95: {:7.2}ms  p99: {:7.2}ms  QPS: {:6.0}",
            name, result.avg_ms, result.p95_ms, result.p99_ms, result.qps
        );
    }
}
```

- [ ] **Step 3: 运行基准测试**

Run: `cargo bench --package sqlrustgo-graph --bench graph_benchmark`
Expected: 输出基准测试结果表格

- [ ] **Step 4: 添加 JSON 报告导出**

在 `BenchmarkResult` 结构体后添加：

```rust
impl BenchmarkResult {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "avg_ms": self.avg_ms,
            "p95_ms": self.p95_ms,
            "p99_ms": self.p99_ms,
            "qps": self.qps,
        })
    }
}
```

- [ ] **Step 5: 修改 main() 输出 JSON 文件**

```rust
use std::fs::File;
use std::io::Write;
use chrono::Local;

fn main() {
    // ... existing benchmark code ...

    // 输出到 JSON 文件
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("benchmark_results/graph_benchmark_{}.json", timestamp);

    let json_results: Vec<_> = results.iter().map(|r| r.to_json()).collect();
    let json = serde_json::to_string_pretty(&json_results).unwrap();

    let mut file = File::create(&filename).unwrap();
    file.write_all(json.as_bytes()).unwrap();

    println!("\nReport saved to: {}", filename);
}
```

- [ ] **Step 6: 运行并验证 JSON 输出**

Run: `cargo bench --package sqlrustgo-graph --bench graph_benchmark`
Expected: 生成 `benchmark_results/graph_benchmark_*.json` 文件

- [ ] **Step 7: 提交**

```bash
git add crates/graph/benches/graph_benchmark.rs crates/graph/Cargo.toml
git commit -m "feat(graph): add graph benchmark suite"
```

---

## Task 4: 性能验证

- [ ] **Step 1: 运行完整基准测试**

Run: `cargo bench --package sqlrustgo-graph --bench graph_benchmark 2>&1 | tee graph_bench.log`

- [ ] **Step 2: 检查结果是否满足目标**

目标:
- BFS 1000 节点 < 50ms
- DFS 1000 节点 < 100ms
- 3-hop 1000 节点 < 500ms

- [ ] **Step 3: 如未达标，分析瓶颈**

检查:
- AdjacencyIndex 是否有优化空间
- 是否需要添加缓存
- 算法复杂度是否可优化

- [ ] **Step 4: 提交最终状态**

```bash
git add -A
git commit -m "perf(graph): complete graph benchmark and verify performance targets"
```

---

## 验收标准

1. `cargo test --package sqlrustgo-graph` 全部通过
2. `cargo bench --package sqlrustgo-graph --bench graph_benchmark` 输出基准测试结果
3. BFS/DFS/multi-hop 测试覆盖所有节点规模
4. JSON 报告正确生成

## 性能目标

| 指标 | 目标 |
|------|------|
| BFS 1000 | < 50ms |
| DFS 1000 | < 100ms |
| 3-hop 1000 | < 500ms |
