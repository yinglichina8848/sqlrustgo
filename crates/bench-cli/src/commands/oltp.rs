use crate::cli::OltpArgs;
use crate::metrics::LatencyCollector;
use crate::reporter::BenchmarkResult;
use sqlrustgo::{parse, ExecutionEngine};
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub fn run(args: OltpArgs) -> BenchmarkResult {
    let mut result = BenchmarkResult::new(
        "oltp".to_string(),
        serde_json::json!({
            "threads": args.threads,
            "duration": args.duration,
            "workload": args.workload,
        }),
    );

    let mut collector = LatencyCollector::new();

    let sql = "SELECT * FROM orders WHERE order_id = 1";

    let start = Instant::now();
    while start.elapsed().as_secs() < args.duration {
        let iteration_start = Instant::now();
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::new(storage);
        let _ = engine.execute(parse(sql).unwrap());
        collector.record(iteration_start.elapsed().as_nanos() as u64);
    }

    result.metrics = collector.into_metrics(args.duration as u32);
    result
}
