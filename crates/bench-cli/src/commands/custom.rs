use crate::cli::CustomArgs;
use crate::metrics::LatencyCollector;
use crate::reporter::BenchmarkResult;
use sqlrustgo::{parse, ExecutionEngine};
use sqlrustgo_storage::MemoryStorage;
use std::fs;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub fn run(args: CustomArgs) -> BenchmarkResult {
    let sql_content = fs::read_to_string(&args.file).expect("Failed to read SQL file");

    let queries: Vec<&str> = sql_content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with("--"))
        .collect();

    let mut result = BenchmarkResult::new(
        "custom".to_string(),
        serde_json::json!({
            "file": args.file,
            "iterations": args.iterations,
            "parallel": args.parallel,
            "query_count": queries.len(),
        }),
    );

    let mut collector = LatencyCollector::new();

    for sql in &queries {
        for _ in 0..args.iterations {
            let iteration_start = Instant::now();
            let storage = Arc::new(RwLock::new(MemoryStorage::new()));
            let mut engine = ExecutionEngine::new(storage);
            let _ = engine.execute(parse(sql).unwrap());
            collector.record(iteration_start.elapsed().as_nanos() as u64);
        }
    }

    result.metrics = collector.into_metrics((queries.len() * args.iterations as usize) as u32);
    result
}
