//! Storage Layer Performance Test
//!
//! Tests FileStorage scan performance with SF=0.1 data (866K rows).

use sqlrustgo_storage::{FileStorage, StorageEngine};
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let data_dir = PathBuf::from("/tmp/tpch_import_test");

    println!("==============================================");
    println!("  Storage Layer Performance Test (SF=0.1)");
    println!("==============================================");
    println!("Data dir: {:?}", data_dir);

    // Check if data exists
    let storage = match FileStorage::new(data_dir.clone()) {
        Ok(s) => s,
        Err(e) => {
            println!("❌ Failed to open FileStorage: {:?}", e);
            println!("Please run: cargo run -p sqlrustgo-storage --example tpch_import");
            std::process::exit(1);
        }
    };

    println!("\nScanning tables...\n");

    let tables = vec![
        ("nation", 25),
        ("region", 5),
        ("supplier", 1000),
        ("part", 20000),
        ("partsupp", 80000),
        ("customer", 15000),
        ("orders", 150000),
        ("lineitem", 600572),
    ];

    let mut results = Vec::new();
    let total_start = Instant::now();

    for (table, expected) in tables {
        print!("  {}: ", table);

        let start = Instant::now();
        let rows = storage
            .scan(table)
            .expect(&format!("Failed to scan {}", table));
        let elapsed = start.elapsed();

        let status = if rows.len() >= expected { "✅" } else { "❌" };
        let rate = rows.len() as f64 / elapsed.as_secs_f64();

        println!(
            "{:>7} rows in {:>8.2}ms ({:>10.0} rows/s) {}",
            rows.len(),
            elapsed.as_secs_f64() * 1000.0,
            rate,
            status
        );

        results.push((table, rows.len(), elapsed, rows.len() >= expected));
    }

    drop(storage);

    let total_elapsed = total_start.elapsed();
    let total_rows: usize = results.iter().map(|(_, r, _, _)| r).sum();

    println!("\n==============================================");
    println!("  Summary");
    println!("==============================================");
    println!("Total rows scanned: {}", total_rows);
    println!("Total time: {:.2}ms", total_elapsed.as_secs_f64() * 1000.0);
    println!(
        "Overall rate: {:.0} rows/s",
        total_rows as f64 / total_elapsed.as_secs_f64()
    );

    let all_passed = results.iter().all(|(_, _, _, ok)| *ok);
    if all_passed {
        println!("\n✅ ALL SCANS PASSED!");
    } else {
        println!("\n❌ SOME SCANS FAILED!");
    }
    println!("==============================================");
}
