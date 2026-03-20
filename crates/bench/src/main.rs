//! SQLRustGo Benchmark Runner
//!
//! A unified benchmark CLI for OLTP and OLAP workloads.
//!
//! # Usage
//!
//! ```bash
//! cargo run -p sqlrustgo-bench -- \
//!   --db sqlrustgo \
//!   --workload oltp \
//!   --threads 10 \
//!   --duration 60 \
//!   --scale 10000
//! ```

mod cli;
mod runner;
mod workload;
mod metrics;
mod db;
mod analysis;

use anyhow::Result;
use clap::Parser;
use cli::BenchArgs;
use runner::run_benchmark;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 tracing 日志
    tracing_subscriber::fmt::init();

    tracing::info!("Starting SQLRustGo Benchmark Runner...");

    let args = BenchArgs::parse();
    run_benchmark(args).await?;

    Ok(())
}
