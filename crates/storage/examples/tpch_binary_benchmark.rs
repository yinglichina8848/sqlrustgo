//! TPC-H Binary Storage Benchmark
//!
//! Measures BinaryTableStorage read/write performance with TPC-H scale data.
//!
//! Usage:
//!   cargo run --example tpch_binary_benchmark -p sqlrustgo-storage

use sqlrustgo_storage::BinaryTableStorage;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let binary_dir = PathBuf::from("/tmp/tpch_binary");

    println!("==============================================");
    println!("  TPC-H Binary Storage Benchmark");
    println!("==============================================");

    // Check if binary data exists
    let binary = match BinaryTableStorage::new(binary_dir.clone()) {
        Ok(b) => b,
        Err(e) => {
            println!("Failed to open binary storage at {:?}: {:?}", binary_dir, e);
            println!("Run: cargo run --example tpch_binary_import -- /path/to/tpch-sf1");
            std::process::exit(1);
        }
    };

    // Benchmark table loads
    let tables = [
        "nation", "region", "supplier", "part", "partsupp", "customer", "orders", "lineitem",
    ];
    let mut total_rows = 0u64;
    let mut total_time = std::time::Duration::new(0, 0);

    println!("\nLoading tables from binary storage...");
    for table in tables {
        let start = Instant::now();
        match binary.load(table) {
            Ok(data) => {
                let elapsed = start.elapsed();
                total_rows += data.rows.len() as u64;
                total_time += elapsed;
                println!(
                    "  {}: {} rows in {:.2}ms",
                    table,
                    data.rows.len(),
                    elapsed.as_secs_f64() * 1000.0
                );
            }
            Err(e) => {
                println!("  {}: FAILED - {:?}", table, e);
            }
        }
    }

    println!("\n==============================================");
    println!("  Summary");
    println!("==============================================");
    println!("Total rows: {}", total_rows);
    println!(
        "Total load time: {:.2}ms",
        total_time.as_secs_f64() * 1000.0
    );
    if total_rows > 0 {
        println!(
            "Throughput: {:.0} rows/sec",
            total_rows as f64 / total_time.as_secs_f64()
        );
    }
    println!("==============================================");
}
