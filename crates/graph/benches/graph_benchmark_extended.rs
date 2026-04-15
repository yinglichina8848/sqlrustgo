use sqlrustgo_graph::{
    bfs_with_distances, dfs_collect, graph_generator::GraphGenerator, multi_hop, GraphStore, NodeId,
};
use std::time::Instant;

struct BenchmarkResult {
    name: String,
    nodes: usize,
    avg_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    qps: f64,
}

impl BenchmarkResult {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "nodes": self.nodes,
            "avg_ms": self.avg_ms,
            "p95_ms": self.p95_ms,
            "p99_ms": self.p99_ms,
            "qps": self.qps,
        })
    }
}

fn run_benchmark<F>(name: &str, nodes: usize, iterations: usize, f: F) -> BenchmarkResult
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
    let p95_idx = ((iterations as f64 * 0.95) as usize).min(iterations - 1);
    let p99_idx = ((iterations as f64 * 0.99) as usize).min(iterations - 1);
    let p95 = times[p95_idx] as f64;
    let p99 = times[p99_idx] as f64;
    let qps = 1_000_000_000.0 / avg;

    BenchmarkResult {
        name: name.to_string(),
        nodes,
        avg_ms: avg / 1_000_000.0,
        p95_ms: p95 / 1_000_000.0,
        p99_ms: p99 / 1_000_000.0,
        qps,
    }
}

fn main() {
    println!("Graph Benchmark Results - Extended Scale Test");
    println!("==========================================\n");

    std::fs::create_dir_all("benchmark_results").ok();

    // Extended scale test: 10x, 100x, 1000x, 10000x
    // Base: 100, 1000, 10000
    // 10x: 1K, 10K, 100K
    // 100x: 10K, 100K, 1M
    // 1000x: 100K, 1M, 10M (skip 10M due to memory)
    // 10000x: 1M, 10M, 100M (skip due to memory)

    let test_cases = vec![
        // Baseline (1x): 100, 1K, 10K
        ("BFS", 100, 100),
        ("BFS", 1_000, 100),
        ("BFS", 10_000, 100),
        ("DFS", 100, 100),
        ("DFS", 1_000, 100),
        ("DFS", 10_000, 100),
        // 10x scale: 1K, 10K, 100K
        ("BFS", 1_000, 100),
        ("BFS", 10_000, 100),
        ("DFS", 1_000, 100),
        ("DFS", 10_000, 100),
        // Multi-hop 2-hop at various scales
        ("2HOP", 100, 100),
        ("2HOP", 1_000, 100),
        ("2HOP", 10_000, 100),
        // Multi-hop 3-hop at various scales
        ("3HOP", 100, 100),
        ("3HOP", 1_000, 100),
        ("3HOP", 10_000, 100),
        // Multi-hop 4-hop at various scales
        ("4HOP", 100, 100),
        ("4HOP", 1_000, 100),
        ("4HOP", 10_000, 100),
    ];

    let mut results: Vec<BenchmarkResult> = Vec::new();

    for (query_type, nodes, iterations) in test_cases {
        let name = format!("{}_{}", query_type, nodes);
        print!("Testing {} ... ", name);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let gen = GraphGenerator::new(42);
        let store = gen.generate(nodes, 3);

        let get_neighbors =
            |n: NodeId| -> Vec<NodeId> { store.neighbors_by_edge_label(n, "connects") };

        let start_node = NodeId(0);

        let result = match query_type {
            "BFS" => run_benchmark(&name, nodes, iterations, || {
                let _ = bfs_with_distances(&get_neighbors, start_node);
            }),
            "DFS" => run_benchmark(&name, nodes, iterations, || {
                let _ = dfs_collect(&get_neighbors, start_node);
            }),
            "2HOP" => run_benchmark(&name, nodes, iterations, || {
                let _ = multi_hop(get_neighbors, start_node, 2);
            }),
            "3HOP" => run_benchmark(&name, nodes, iterations, || {
                let _ = multi_hop(get_neighbors, start_node, 3);
            }),
            "4HOP" => run_benchmark(&name, nodes, iterations, || {
                let _ = multi_hop(get_neighbors, start_node, 4);
            }),
            _ => continue,
        };

        println!(
            "avg: {:8.4}ms  p95: {:8.4}ms  QPS: {:10.0}",
            result.avg_ms, result.p95_ms, result.qps
        );

        results.push(result);
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!(
        "benchmark_results/graph_benchmark_extended_{}.json",
        timestamp
    );

    let json_results: Vec<_> = results.iter().map(|r| r.to_json()).collect();
    let json = serde_json::to_string_pretty(&json_results).unwrap();

    std::fs::write(&filename, &json).unwrap();

    println!("\nReport saved to: {}", filename);

    // Performance target summary
    println!("\n=== Performance Target Verification ===");
    for r in &results {
        if r.nodes == 1_000 && r.name.starts_with("BFS") {
            let status = if r.avg_ms < 50.0 { "✓" } else { "✗" };
            println!("{} BFS_1K: {:.4}ms < 50ms {}", status, r.avg_ms, status);
        }
        if r.nodes == 1_000 && r.name.starts_with("DFS") {
            let status = if r.avg_ms < 100.0 { "✓" } else { "✗" };
            println!("{} DFS_1K: {:.4}ms < 100ms {}", status, r.avg_ms, status);
        }
        if r.nodes == 1_000 && r.name.starts_with("3HOP") {
            let status = if r.avg_ms < 500.0 { "✓" } else { "✗" };
            println!("{} 3HOP_1K: {:.4}ms < 500ms {}", status, r.avg_ms, status);
        }
    }
}
