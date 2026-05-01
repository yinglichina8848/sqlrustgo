use sqlrustgo_graph::{
    bfs_with_distances, dfs_collect, graph_generator::GraphGenerator, multi_hop, GraphStore, NodeId,
};
use std::time::Instant;

struct BenchmarkResult {
    name: String,
    avg_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    qps: f64,
}

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
    let p95_idx = ((iterations as f64 * 0.95) as usize).min(iterations - 1);
    let p99_idx = ((iterations as f64 * 0.99) as usize).min(iterations - 1);
    let p95 = times[p95_idx] as f64;
    let p99 = times[p99_idx] as f64;
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

    std::fs::create_dir_all("benchmark_results").ok();

    // Performance test scale: 10x, 100x, 1000x, 10000x
    // Base: 100, 1000, 10000
    // 10x: 1K, 10K, 100K
    // 100x: 10K, 100K, 1M
    // 1000x: 100K, 1M, 10M
    // 10000x: 1M, 10M, 100M
    let test_cases = vec![
        // Baseline (1x)
        ("BFS_100", 100, "bfs"),
        ("BFS_1000", 1000, "bfs"),
        ("BFS_10000", 10000, "bfs"),
        ("DFS_100", 100, "dfs"),
        ("DFS_1000", 1000, "dfs"),
        ("DFS_10000", 10000, "dfs"),
        // 10x scale
        ("BFS_1K", 1_000, "bfs"),
        ("BFS_10K", 10_000, "bfs"),
        ("BFS_100K", 100_000, "bfs"),
        ("DFS_1K", 1_000, "dfs"),
        ("DFS_10K", 10_000, "dfs"),
        ("DFS_100K", 100_000, "dfs"),
        // 100x scale
        ("BFS_10K_100x", 10_000, "bfs"),
        ("BFS_100K_100x", 100_000, "bfs"),
        ("BFS_1M", 1_000_000, "bfs"),
        ("DFS_10K_100x", 10_000, "dfs"),
        ("DFS_100K_100x", 100_000, "dfs"),
        ("DFS_1M", 1_000_000, "dfs"),
        // 1000x scale
        ("BFS_100K_1Kx", 100_000, "bfs"),
        ("BFS_1M_1Kx", 1_000_000, "bfs"),
        ("BFS_10M", 10_000_000, "bfs"),
        ("DFS_100K_1Kx", 100_000, "dfs"),
        ("DFS_1M_1Kx", 1_000_000, "dfs"),
        ("DFS_10M", 10_000_000, "dfs"),
        // Multi-hop tests at various scales
        ("2HOP_100", 100, "2hop"),
        ("2HOP_1K", 1_000, "2hop"),
        ("2HOP_10K", 10_000, "2hop"),
        ("2HOP_100K", 100_000, "2hop"),
        ("2HOP_1M", 1_000_000, "2hop"),
        ("3HOP_100", 100, "3hop"),
        ("3HOP_1K", 1_000, "3hop"),
        ("3HOP_10K", 10_000, "3hop"),
        ("3HOP_100K", 100_000, "3hop"),
        ("3HOP_1M", 1_000_000, "3hop"),
        ("4HOP_100", 100, "4hop"),
        ("4HOP_1K", 1_000, "4hop"),
        ("4HOP_10K", 10_000, "4hop"),
        ("4HOP_100K", 100_000, "4hop"),
        ("4HOP_1M", 1_000_000, "4hop"),
    ];

    let mut results: Vec<BenchmarkResult> = Vec::new();

    for (name, nodes, query_type) in test_cases {
        let gen = GraphGenerator::new(42);
        let store = gen.generate(nodes, 3);

        let get_neighbors =
            |n: NodeId| -> Vec<NodeId> { store.neighbors_by_edge_label(n, "connects") };

        let start_node = NodeId(0);

        // Adjust iterations based on scale to keep runtime reasonable
        let iterations = if nodes <= 100_000 {
            100
        } else if nodes <= 1_000_000 {
            50
        } else {
            10
        };

        let result = match query_type {
            "bfs" => run_benchmark(name, iterations, || {
                let _ = bfs_with_distances(&get_neighbors, start_node);
            }),
            "dfs" => run_benchmark(name, iterations, || {
                let _ = dfs_collect(&get_neighbors, start_node);
            }),
            "2hop" => run_benchmark(name, iterations, || {
                let _ = multi_hop(get_neighbors, start_node, 2);
            }),
            "3hop" => run_benchmark(name, iterations, || {
                let _ = multi_hop(get_neighbors, start_node, 3);
            }),
            "4hop" => run_benchmark(name, iterations, || {
                let _ = multi_hop(get_neighbors, start_node, 4);
            }),
            _ => continue,
        };

        println!(
            "{:12} avg: {:7.2}ms  p95: {:7.2}ms  p99: {:7.2}ms  QPS: {:6.0}",
            name, result.avg_ms, result.p95_ms, result.p99_ms, result.qps
        );

        results.push(result);
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("benchmark_results/graph_benchmark_{}.json", timestamp);

    let json_results: Vec<_> = results.iter().map(|r| r.to_json()).collect();
    let json = serde_json::to_string_pretty(&json_results).unwrap();

    std::fs::write(&filename, &json).unwrap();

    println!("\nReport saved to: {}", filename);
}
