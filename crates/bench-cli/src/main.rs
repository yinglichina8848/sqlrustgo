use clap::Parser;
use sqlrustgo_bench::db::postgres_benchmark::PostgresBenchmark;
use std::fs;
use std::path::Path;
use tokio::runtime::Runtime;

mod cli;
mod commands;
mod metrics;
mod reporter;

use cli::BenchmarkConfig;
use commands::{custom, oltp, tpch};

#[derive(Parser, Debug)]
#[command(name = "benchmark")]
#[command(about = "SQLRustGo Benchmark CLI", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    Tpch(cli::TpchArgs),
    Oltp(cli::OltpArgs),
    Custom(cli::CustomArgs),
}

/// Load configuration from file, CLI args take precedence
fn load_config(path: &str) -> Option<BenchmarkConfig> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Run PostgreSQL comparison benchmark
fn run_pg_comparison(conn_str: &str, operations: u64) {
    let rt = Runtime::new().expect("Failed to create runtime");
    rt.block_on(async {
        let pg = PostgresBenchmark::new(conn_str);
        match pg.run_reads(operations).await {
            Ok(result) => {
                println!("\nPostgreSQL Results:");
                println!("  Total Time: {} ms", result.total_time_ms);
                println!("  QPS: {:.2}", result.qps);
                println!("  P50: {:.3} ms", result.latency_stats.p50_ms);
                println!("  P95: {:.3} ms", result.latency_stats.p95_ms);
                println!("  P99: {:.3} ms", result.latency_stats.p99_ms);
            }
            Err(e) => {
                eprintln!("PostgreSQL benchmark failed: {}", e);
            }
        }
    });
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Tpch(tpch_args) => {
            // Load config file if provided
            let config = tpch_args.config.as_ref().and_then(|p| load_config(p));
            let output = tpch_args
                .output
                .clone()
                .or_else(|| config.as_ref().and_then(|c| c.output.clone()));
            let pg_conn = tpch_args
                .pg_conn
                .clone()
                .or_else(|| config.as_ref().and_then(|c| c.pg_conn.clone()));

            let result = tpch::run(tpch_args);
            if let Some(output_path) = output {
                result.save(Path::new(&output_path)).unwrap();
                println!("Results saved to: {}", output_path);
            } else {
                result.print_json();
            }

            // Run PostgreSQL comparison if connection provided
            if let Some(ref conn) = pg_conn {
                println!("\n=== PostgreSQL Comparison ===");
                run_pg_comparison(conn, 100);
            }
        }
        Command::Oltp(oltp_args) => {
            let config = oltp_args.config.as_ref().and_then(|p| load_config(p));
            let output = oltp_args
                .output
                .clone()
                .or_else(|| config.as_ref().and_then(|c| c.output.clone()));
            let pg_conn = oltp_args
                .pg_conn
                .clone()
                .or_else(|| config.as_ref().and_then(|c| c.pg_conn.clone()));

            let result = oltp::run(oltp_args);
            if let Some(output_path) = output {
                result.save(Path::new(&output_path)).unwrap();
                println!("Results saved to: {}", output_path);
            } else {
                result.print_json();
            }

            if let Some(ref conn) = pg_conn {
                println!("\n=== PostgreSQL Comparison ===");
                run_pg_comparison(conn, 100);
            }
        }
        Command::Custom(custom_args) => {
            let config = custom_args.config.as_ref().and_then(|p| load_config(p));
            let output = custom_args
                .output
                .clone()
                .or_else(|| config.as_ref().and_then(|c| c.output.clone()));
            let pg_conn = custom_args
                .pg_conn
                .clone()
                .or_else(|| config.as_ref().and_then(|c| c.pg_conn.clone()));

            let result = custom::run(custom_args);
            if let Some(output_path) = output {
                result.save(Path::new(&output_path)).unwrap();
                println!("Results saved to: {}", output_path);
            } else {
                result.print_json();
            }

            if let Some(ref conn) = pg_conn {
                println!("\n=== PostgreSQL Comparison ===");
                run_pg_comparison(conn, 100);
            }
        }
    };
}
