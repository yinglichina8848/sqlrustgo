//! Binary Storage Format Performance Test
//!
//! Compare JSON vs Binary storage performance.

use sqlrustgo_storage::{FileStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let data_dir = PathBuf::from("/tmp/tpch_import_test");
    
    println!("==============================================");
    println!("  Storage Format Performance Test");
    println!("==============================================");
    
    // Test 1: Read with FileStorage (JSON-based)
    println!("\n[Test 1] FileStorage (JSON) scan:");
    let start = Instant::now();
    let storage = FileStorage::new(data_dir.clone()).expect("Failed to open FileStorage");
    let rows = storage.scan("lineitem").expect("Failed to scan");
    let json_time = start.elapsed();
    println!("  Scanned {} rows in {:.2}ms", rows.len(), json_time.as_secs_f64() * 1000.0);
    println!("  Throughput: {:.0} rows/s", rows.len() as f64 / json_time.as_secs_f64());
    drop(storage);
    
    // Test 2: Raw binary scan (simulated)
    println!("\n[Test 2] Raw binary file scan:");
    let bin_path = data_dir.join("lineitem.bin");
    let raw_path = PathBuf::from("/Users/liying/workspace/dev/heartopen/SQLRustGo/data/tpch-sf01-generated/lineitem.tbl");
    
    let start = Instant::now();
    let file = File::open(&raw_path).expect("Failed to open raw file");
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    let mut line_count = 0;
    while reader.read_line(&mut buffer).unwrap_or(0) > 0 {
        line_count += 1;
        buffer.clear();
    }
    let raw_time = start.elapsed();
    println!("  Scanned {} lines in {:.2}ms", line_count, raw_time.as_secs_f64() * 1000.0);
    println!("  Throughput: {:.0} lines/s", line_count as f64 / raw_time.as_secs_f64());
    
    // Summary
    println!("\n==============================================");
    println!("  Summary");
    println!("==============================================");
    println!("{:<25} {:>12} {:>15}", "Method", "Time (ms)", "Throughput");
    println!("----------------------------------------");
    println!("{:<25} {:>12.2} {:>15.0}", "FileStorage (JSON)", json_time.as_secs_f64() * 1000.0, rows.len() as f64 / json_time.as_secs_f64());
    println!("{:<25} {:>12.2} {:>15.0}", "Raw file (text)", raw_time.as_secs_f64() * 1000.0, line_count as f64 / raw_time.as_secs_f64());
    
    let speedup = raw_time.as_secs_f64() / json_time.as_secs_f64();
    println!("\nRaw file is {:.1}x faster than FileStorage JSON", speedup);
    println!("(FileStorage has JSON serialization overhead)");
    println!("==============================================");
}
