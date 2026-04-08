//! Profile scan performance to find bottleneck

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let data_dir = PathBuf::from("/tmp/tpch_import_test");
    let json_path = data_dir.join("lineitem.json");
    
    println!("==============================================");
    println!("  Scan Performance Profile");
    println!("==============================================\n");
    
    // Step 1: Raw file read time
    println!("[1] Raw file read:");
    let start = Instant::now();
    let file = File::open(&json_path).unwrap();
    let mut reader = BufReader::new(file);
    let mut contents = Vec::new();
    reader.read_to_end(&mut contents).unwrap();
    let read_time = start.elapsed();
    println!("  File size: {} bytes", contents.len());
    println!("  Read time: {:.2}ms", read_time.as_secs_f64() * 1000.0);
    println!("  Read speed: {:.0} MB/s", contents.len() as f64 / 1024.0 / 1024.0 / read_time.as_secs_f64());
    
    // Step 2: JSON parsing time
    println!("\n[2] JSON parsing:");
    let start = Instant::now();
    let parsed: serde_json::Value = serde_json::from_slice(&contents).unwrap();
    let parse_time = start.elapsed();
    println!("  Parse time: {:.2}ms", parse_time.as_secs_f64() * 1000.0);
    
    // Summary
    let total = read_time + parse_time;
    println!("\n==============================================");
    println!("  Summary (lineitem 600K rows)");
    println!("==============================================");
    println!("{:<20} {:>10} {:>8}", "Step", "Time (ms)", "%");
    println!("----------------------------------------");
    println!("{:<20} {:>10.2} {:>7.1}%", "File read", read_time.as_secs_f64() * 1000.0, 
             100.0 * read_time.as_secs_f64() / total.as_secs_f64());
    println!("{:<20} {:>10.2} {:>7.1}%", "JSON parse", parse_time.as_secs_f64() * 1000.0,
             100.0 * parse_time.as_secs_f64() / total.as_secs_f64());
    println!("----------------------------------------");
    println!("{:<20} {:>10.2}", "TOTAL", total.as_secs_f64() * 1000.0);
    println!("==============================================");
}
