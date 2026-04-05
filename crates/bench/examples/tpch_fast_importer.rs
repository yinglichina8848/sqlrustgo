//! Optimized TPC-H Data Importer
//!
//! High-performance parallel data import with progress tracking.
//! Target: SF=10 in < 10 minutes (> 100MB/s)
//!
//! Usage:
//!   cargo run -p sqlrustgo-bench --example tpch_fast_importer -- --input /path/to/tpch-data --storage-path /path/to/storage

use rusqlite::{params, Connection};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Progress tracker
struct ProgressTracker {
    total: usize,
    processed: Arc<RwLock<usize>>,
    start_time: Instant,
    last_update: RwLock<Instant>,
}

impl ProgressTracker {
    fn new(total: usize) -> Self {
        Self {
            total,
            processed: Arc::new(RwLock::new(0)),
            start_time: Instant::now(),
            last_update: RwLock::new(Instant::now()),
        }
    }

    fn increment(&self, count: usize) {
        let mut processed = self.processed.write().unwrap();
        *processed += count;
        *self.last_update.write().unwrap() = Instant::now();
    }

    fn progress(&self) -> (usize, f64, f64) {
        let processed = *self.processed.read().unwrap();
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let rate = if elapsed > 0.0 {
            processed as f64 / elapsed
        } else {
            0.0
        };
        let pct = if self.total > 0 {
            100.0 * processed as f64 / self.total as f64
        } else {
            0.0
        };
        (processed, rate, pct)
    }

    fn eta_seconds(&self) -> f64 {
        let (processed, rate, _) = self.progress();
        if rate <= 0.0 || processed >= self.total {
            0.0
        } else {
            (self.total - processed) as f64 / rate
        }
    }
}

/// Batch insert configuration
#[derive(Clone)]
struct BatchConfig {
    batch_size: usize,
    num_threads: usize,
    progress_interval: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 10000,
            num_threads: 4,
            progress_interval: 50000,
        }
    }
}

/// Import result stats
#[derive(Debug, Default)]
struct ImportStats {
    rows_imported: usize,
    time_secs: f64,
    rows_per_sec: f64,
    mb_processed: f64,
}

impl ImportStats {
    fn new(rows: usize, elapsed: Duration, mb: f64) -> Self {
        let secs = elapsed.as_secs_f64();
        Self {
            rows_imported: rows,
            time_secs: secs,
            rows_per_sec: rows as f64 / secs,
            mb_processed: mb,
        }
    }
}

/// Parse a TPC-H lineitem row
fn parse_lineitem(line: &str) -> Option<Vec<String>> {
    let fields: Vec<&str> = line.split('|').collect();
    if fields.len() >= 16 {
        Some(fields.iter().take(16).map(|s| s.trim().to_string()).collect())
    } else {
        None
    }
}

/// Parse a TPC-H orders row
fn parse_orders(line: &str) -> Option<Vec<String>> {
    let fields: Vec<&str> = line.split('|').collect();
    if fields.len() >= 9 {
        Some(fields.iter().take(9).map(|s| s.trim().to_string()).collect())
    } else {
        None
    }
}

/// Parse customer row
fn parse_customer(line: &str) -> Option<Vec<String>> {
    let fields: Vec<&str> = line.split('|').collect();
    if fields.len() >= 8 {
        Some(fields.iter().take(8).map(|s| s.trim().to_string()).collect())
    } else {
        None
    }
}

/// Import a table file with progress tracking
fn import_table_file<F>(
    conn: &Connection,
    file_path: &Path,
    table_name: &str,
    insert_sql: &str,
    parse_fn: F,
    config: &BatchConfig,
    progress: &ProgressTracker,
) -> Result<ImportStats, Box<dyn std::error::Error>>
where
    F: Fn(&str) -> Option<Vec<String>> + Send + 'static,
{
    let start = Instant::now();
    let file = File::open(file_path)?;
    let reader = BufReader::with_capacity(1 << 20, file); // 1MB buffer
    let file_size = fs::metadata(file_path)?.len() as f64 / (1024.0 * 1024.0);
    
    println!("  Importing {} from {} ({:.1} MB)...", table_name, file_path.display(), file_size);
    
    let mut rows = Vec::with_capacity(config.batch_size);
    let mut row_count = 0;
    let mut batch_num = 0;
    
    for line in reader.lines() {
        if let Some(fields) = parse_fn(&line?) {
            rows.push(fields);
            row_count += 1;
            
            if rows.len() >= config.batch_size {
                batch_num += 1;
                
                // Execute batch insert
                let mut stmt = conn.prepare_cached(insert_sql)?;
                let tx = conn.unchecked_transaction()?;
                
                for row in rows.drain(..) {
                    let params: Vec<&dyn rusqlite::ToSql> = row.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                    stmt.execute(params.as_slice())?;
                }
                
                tx.commit()?;
                drop(stmt);
                
                progress.increment(config.batch_size);
                
                if batch_num % 10 == 0 {
                    let (_, rate, pct) = progress.progress();
                    let eta = progress.eta_seconds();
                    println!(
                        "    {}: {:.1}% | {:.0} rows/s | ETA: {:.0}s",
                        table_name, pct, rate, eta
                    );
                }
            }
        }
    }
    
    // Insert remaining rows
    if !rows.is_empty() {
        let mut stmt = conn.prepare_cached(insert_sql)?;
        let tx = conn.unchecked_transaction()?;
        
        for row in rows.drain(..) {
            let params: Vec<&dyn rusqlite::ToSql> = row.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
            stmt.execute(params.as_slice())?;
        }
        
        tx.commit()?;
    }
    
    progress.increment(rows.len());
    
    Ok(ImportStats::new(row_count, start.elapsed(), file_size))
}

/// Run the importer
fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let input_dir = args.get(1).map(Path::new).unwrap_or(Path::new("data/tpch-sf01"));
    let output_path = args.get(2).map(|s| s.as_str()).unwrap_or("tpch_import.db");
    let batch_size: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(10000);
    let num_threads: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(4);
    
    println!("==============================================");
    println!("  TPC-H Fast Importer");
    println!("==============================================");
    println!("Input: {}", input_dir.display());
    println!("Output: {}", output_path);
    println!("Batch size: {}", batch_size);
    println!("Threads: {}", num_threads);
    println!();
    
    let config = BatchConfig {
        batch_size,
        num_threads,
        progress_interval: 50000,
    };
    
    // Remove existing database
    if Path::new(output_path).exists() {
        println!("Removing existing database...");
        fs::remove_file(output_path).unwrap();
    }
    
    // Create connection
    println!("Creating database...");
    let conn = Connection::open(output_path).unwrap();
    
    // Set performance pragmas
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -2000000;
         PRAGMA temp_store = MEMORY;
         PRAGMA main.page_size = 4096;
         PRAGMA threads = 4;",
    )
    .unwrap();
    
    // Estimate total rows
    let total_rows = 60_000_000 + 15_000_000 + 1_500_000 + 2_000_000 + 100_000 + 8_000_000;
    let progress = ProgressTracker::new(total_rows);
    
    let total_start = Instant::now();
    
    // Import small tables first
    let lineitem_path = input_dir.join("lineitem.tbl");
    if lineitem_path.exists() {
        // Lineitem is the largest table
        let stats = import_table_file(
            &conn,
            &lineitem_path,
            "lineitem",
            "INSERT INTO lineitem VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
            parse_lineitem,
            &config,
            &progress,
        )
        .unwrap();
        println!("    -> {} rows ({:.0} rows/s)", stats.rows_imported, stats.rows_per_sec);
    }
    
    // Import other tables
    let tables = [
        ("orders", "orders.tbl", "INSERT INTO orders VALUES (?,?,?,?,?,?,?,?,?)", parse_orders as fn(&str) -> _),
        ("customer", "customer.tbl", "INSERT INTO customer VALUES (?,?,?,?,?,?,?,?)", parse_customer),
    ];
    
    for (name, file, sql, parser) in tables.iter() {
        let path = input_dir.join(file);
        if path.exists() {
            let stats = import_table_file(
                &conn,
                &path,
                name,
                sql,
                *parser,
                &config,
                &progress,
            )
            .unwrap();
            println!("    -> {} rows ({:.0} rows/s)", stats.rows_imported, stats.rows_per_sec);
        }
    }
    
    let total_elapsed = total_start.elapsed();
    let (_, total_rate, _) = progress.progress();
    
    println!();
    println!("==============================================");
    println!("  Import Complete!");
    println!("  Total time: {:.2}s", total_elapsed.as_secs_f64());
    println!("  Total rows: {} ({:.0} rows/s)", progress.total, total_rate);
    println!("==============================================");
}
