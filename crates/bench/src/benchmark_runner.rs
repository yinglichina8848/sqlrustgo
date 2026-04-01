//! Benchmark runner - orchestrates the benchmark execution

use crate::analysis::analyze;
use crate::cli::BenchArgs;
use crate::db::{create_db, DbConfig};
use crate::memory::MemoryLimiter;
use crate::metrics::LatencyRecorder;
use crate::workload::create_workload;

use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{Duration, Instant};

/// Run the benchmark
pub async fn run_benchmark(args: BenchArgs) -> Result<()> {
    tracing::info!("Starting benchmark: {:?}", args);

    // Initialize memory limiter with 10GB default limit
    let memory_limiter = MemoryLimiter::default_limit();
    tracing::info!(
        "Memory limit: {} bytes (8 GB)",
        memory_limiter.limit_bytes()
    );

    // Initial memory check
    let initial_memory = memory_limiter.get_current_usage_string();
    tracing::info!("Initial memory usage: {}", initial_memory);

    // Check if cache is disabled (benchmark mode)
    if !args.enable_cache {
        tracing::info!("⚠️ Benchmark mode: Query cache disabled");
    }

    // Create database connection
    let db_config = DbConfig::from(&args);
    let db = create_db(&args.db, &db_config).await?;

    // Create workload
    let workload = create_workload(&args.workload, args.scale);

    // Initialize metrics
    let latency = Arc::new(LatencyRecorder::new());
    let ops_count = Arc::new(AtomicU64::new(0));

    tracing::info!(
        "Starting workload: {} with {} threads for {}s",
        args.workload,
        args.threads,
        args.duration
    );

    // Run benchmark
    let start = Instant::now();
    let duration = Duration::from_secs(args.duration);
    let check_interval = 1000; // Check memory every 1000 operations

    let mut handles = vec![];

    for _ in 0..args.threads {
        let db = db.clone();
        let workload = workload.clone();
        let latency = latency.clone();
        let ops_count = ops_count.clone();
        let memory_limiter = MemoryLimiter::default_limit();

        let handle = tokio::spawn(async move {
            while Instant::now().duration_since(start) < duration {
                let t0 = Instant::now();

                match workload.execute(db.as_ref()).await {
                    Ok(_) => {
                        let elapsed = t0.elapsed().as_micros() as u64;
                        latency.record(elapsed);
                        let count = ops_count.fetch_add(1, Ordering::Relaxed);

                        // Check memory every `check_interval` operations
                        if count > 0 && count % check_interval == 0 {
                            if memory_limiter.is_exceeded() {
                                tracing::error!(
                                    "❌ Memory limit exceeded! Current: {}",
                                    memory_limiter.get_current_usage_string()
                                );
                                tracing::error!(
                                    "❌ Stopping benchmark to prevent system instability"
                                );
                                break;
                            }
                            // Warn if memory usage is high
                            memory_limiter.check_and_warn();
                        }
                    }
                    Err(e) => {
                        tracing::debug!("Operation error: {}", e);
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    for h in handles {
        h.await?;
    }

    // Final memory check
    tracing::info!(
        "Final memory usage: {}",
        memory_limiter.get_current_usage_string()
    );

    // Calculate results
    let elapsed_secs = start.elapsed().as_secs_f64();
    let total_ops = ops_count.load(Ordering::Relaxed);
    let tps = total_ops as f64 / elapsed_secs;
    let stats = latency.snapshot();

    // Print results
    println!();
    println!("=== BENCHMARK RESULT ===");
    println!("Database:     {}", args.db);
    println!("Workload:     {}", args.workload);
    println!("Threads:      {}", args.threads);
    println!("Duration:      {}s", args.duration);
    println!("Scale:        {}", args.scale);
    println!();
    println!("=== PERFORMANCE ===");
    println!("TPS:          {:.2}", tps);
    println!("Total Ops:    {}", total_ops);
    println!();
    println!("=== LATENCY (µs) ===");
    stats.print();

    // Output JSON if requested
    if args.output == "json" {
        let result = serde_json::json!({
            "database": args.db,
            "workload": args.workload,
            "threads": args.threads,
            "duration": args.duration,
            "scale": args.scale,
            "tps": tps,
            "total_ops": total_ops,
            "memory": {
                "limit_bytes": memory_limiter.limit_bytes(),
                "final_usage": memory_limiter.get_current_usage_string()
            },
            "latency": {
                "p50": stats.p50,
                "p95": stats.p95,
                "p99": stats.p99,
                "p999": stats.p999,
                "max": stats.max,
                "count": stats.count
            }
        });

        println!();
        println!("=== JSON OUTPUT ===");
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    // Run analysis
    analyze(&stats);

    Ok(())
}
