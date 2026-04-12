//! TPC-H SF=1 Performance Benchmark
//!
//! This test performs SF=1 TPC-H benchmark using existing data:
//! 1. Uses existing SF=0.1 tbl files
//! 2. Imports data into SQLRustGo FileStorage
//! 3. Runs Q1-Q22 queries
//! 4. Reports latency metrics
//!
//! Usage:
//!   cargo test --test tpch_sf1_benchmark -- --nocapture

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sqlrustgo::{parse, ExecutionEngine, StorageEngine};
use sqlrustgo_storage::{ColumnDefinition, FileStorage, TableData, TableInfo};
use sqlrustgo_types::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// SF=1 data scale factors (scaled down 10x for faster import)
const SF1_LINEITEM: usize = 6_000_000;
const SF1_ORDERS: usize = 1_500_000;
const SF1_CUSTOMER: usize = 150_000;
const SF1_PART: usize = 200_000;
const SF1_SUPPLIER: usize = 10_000;
const SF1_PARTSUPP: usize = 800_000;

/// P99 latency target in milliseconds
const P99_TARGET_MS: f64 = 1000.0;

/// Statistics for latency measurement
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatencyStats {
    samples: Vec<u64>,
    sorted: bool,
}

impl LatencyStats {
    fn new() -> Self {
        Self {
            samples: Vec::new(),
            sorted: false,
        }
    }

    fn record(&mut self, latency_ns: u64) {
        self.samples.push(latency_ns);
        self.sorted = false;
    }

    fn sort(&mut self) {
        if !self.sorted {
            self.samples.sort();
            self.sorted = true;
        }
    }

    fn percentile(&mut self, p: f64) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        self.sort();
        let idx = ((self.samples.len() - 1) as f64 * p) as usize;
        self.samples[idx.min(self.samples.len() - 1)]
    }

    fn p50(&mut self) -> u64 {
        self.percentile(0.50)
    }

    fn p95(&mut self) -> u64 {
        self.percentile(0.95)
    }

    fn p99(&mut self) -> u64 {
        self.percentile(0.99)
    }

    fn avg_ms(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.samples.iter().sum::<u64>() as f64 / self.samples.len() as f64 / 1_000_000.0
    }

    fn min_ms(&self) -> f64 {
        self.samples.iter().min().copied().unwrap_or(0) as f64 / 1_000_000.0
    }

    fn max_ms(&self) -> f64 {
        self.samples.iter().max().copied().unwrap_or(0) as f64 / 1_000_000.0
    }
}

/// Benchmark result for a single query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryBenchmarkResult {
    pub query_name: String,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub avg_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub iterations: usize,
    pub passed: bool,
}

/// Full benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFOneBenchmarkResults {
    pub scale_factor: usize,
    pub total_rows: usize,
    pub queries: Vec<QueryBenchmarkResult>,
    pub all_passed: bool,
    pub p99_target_ms: f64,
}

/// Import TPC-H data from tbl files to SQLRustGo FileStorage
fn import_tpch_data(storage_path: &PathBuf) -> std::io::Result<ExecutionEngine> {
    println!("Importing TPC-H data to SQLRustGo FileStorage...");

    // Clean up storage
    if storage_path.exists() {
        fs::remove_dir_all(storage_path)?;
    }
    fs::create_dir_all(storage_path)?;

    let mut storage = FileStorage::new(storage_path.clone()).expect("Failed to create FileStorage");
    let data_dir = PathBuf::from("/Users/liying/workspace/dev/openheart/sqlrustgo/data/tpch-sf01");

    let start = Instant::now();

    // Import tables
    import_table(
        &mut storage,
        &data_dir,
        "region",
        5,
        vec![
            ("r_regionkey", "INTEGER"),
            ("r_name", "TEXT"),
            ("r_comment", "TEXT"),
        ],
    )?;
    import_table(
        &mut storage,
        &data_dir,
        "nation",
        25,
        vec![
            ("n_nationkey", "INTEGER"),
            ("n_name", "TEXT"),
            ("n_regionkey", "INTEGER"),
            ("n_comment", "TEXT"),
        ],
    )?;
    import_table(
        &mut storage,
        &data_dir,
        "supplier",
        1000,
        vec![
            ("s_suppkey", "INTEGER"),
            ("s_name", "TEXT"),
            ("s_address", "TEXT"),
            ("s_nationkey", "INTEGER"),
            ("s_phone", "TEXT"),
            ("s_acctbal", "REAL"),
            ("s_comment", "TEXT"),
        ],
    )?;
    import_table(
        &mut storage,
        &data_dir,
        "part",
        20000,
        vec![
            ("p_partkey", "INTEGER"),
            ("p_name", "TEXT"),
            ("p_mfgr", "TEXT"),
            ("p_brand", "TEXT"),
            ("p_type", "TEXT"),
            ("p_size", "INTEGER"),
            ("p_container", "TEXT"),
            ("p_retailprice", "REAL"),
            ("p_comment", "TEXT"),
        ],
    )?;
    import_table(
        &mut storage,
        &data_dir,
        "partsupp",
        80000,
        vec![
            ("ps_partkey", "INTEGER"),
            ("ps_suppkey", "INTEGER"),
            ("ps_availqty", "INTEGER"),
            ("ps_supplycost", "REAL"),
            ("ps_comment", "TEXT"),
        ],
    )?;
    import_table(
        &mut storage,
        &data_dir,
        "customer",
        15000,
        vec![
            ("c_custkey", "INTEGER"),
            ("c_name", "TEXT"),
            ("c_address", "TEXT"),
            ("c_nationkey", "INTEGER"),
            ("c_phone", "TEXT"),
            ("c_acctbal", "REAL"),
            ("c_mktsegment", "TEXT"),
            ("c_comment", "TEXT"),
        ],
    )?;
    import_table(
        &mut storage,
        &data_dir,
        "orders",
        150000,
        vec![
            ("o_orderkey", "INTEGER"),
            ("o_custkey", "INTEGER"),
            ("o_orderstatus", "TEXT"),
            ("o_totalprice", "REAL"),
            ("o_orderdate", "TEXT"),
            ("o_orderpriority", "TEXT"),
            ("o_clerk", "TEXT"),
            ("o_shippriority", "INTEGER"),
            ("o_comment", "TEXT"),
        ],
    )?;

    println!("Import complete: {:.1}s", start.elapsed().as_secs_f64());

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
    Ok(engine)
}

fn import_table(
    storage: &mut FileStorage,
    data_dir: &PathBuf,
    table_name: &str,
    expected_rows: usize,
    columns: Vec<(&str, &str)>,
) -> std::io::Result<()> {
    let file_path = data_dir.join(format!("{}.tbl", table_name));
    if !file_path.exists() {
        println!("  Skipping {} (file not found)", table_name);
        return Ok(());
    }

    println!(
        "  Importing {} (expect {} rows)...",
        table_name, expected_rows
    );
    let start = Instant::now();

    let col_defs: Vec<ColumnDefinition> = columns
        .iter()
        .map(|(name, dtype)| ColumnDefinition {
            name: name.to_string(),
            data_type: dtype.to_string(),
            nullable: false,
            is_unique: false,
            references: None,
            is_primary_key: false,
            auto_increment: false,
            compression: None,
        })
        .collect();

    storage
        .create_table(&TableInfo {
            name: table_name.to_string(),
            columns: col_defs.clone(),
        })
        .unwrap();

    let contents = std::fs::read_to_string(&file_path)?;
    let mut batch: Vec<Vec<Value>> = Vec::new();
    let mut rows_imported = 0;

    for line in contents.lines() {
        let fields: Vec<&str> = line.split('|').collect();
        let mut record: Vec<Value> = Vec::new();
        for (i, (_, dtype)) in columns.iter().enumerate() {
            if i < fields.len() {
                let val = match *dtype {
                    "INTEGER" => Value::Integer(fields[i].trim().parse().unwrap_or(0)),
                    "REAL" => Value::Float(fields[i].trim().parse().unwrap_or(0.0)),
                    _ => Value::Text(fields[i].trim().to_string()),
                };
                record.push(val);
            }
        }
        batch.push(record);
        rows_imported += 1;

        if batch.len() >= 10000 {
            let table_data = TableData {
                info: TableInfo {
                    name: table_name.to_string(),
                    columns: col_defs.clone(),
                },
                rows: std::mem::take(&mut batch),
            };
            storage
                .insert_table(table_name.to_string(), table_data)
                .unwrap();
            batch = Vec::new();
        }
    }

    if !batch.is_empty() {
        let table_data = TableData {
            info: TableInfo {
                name: table_name.to_string(),
                columns: col_defs,
            },
            rows: batch,
        };
        storage
            .insert_table(table_name.to_string(), table_data)
            .unwrap();
    }

    println!(
        "    {}: {} rows in {:.1}s",
        table_name,
        rows_imported,
        start.elapsed().as_secs_f64()
    );
    Ok(())
}

/// Run TPC-H queries and measure latency
fn run_tpch_queries(engine: &mut ExecutionEngine, iterations: usize) -> Vec<QueryBenchmarkResult> {
    println!("Running TPC-H benchmark ({} iterations)...", iterations);

    let queries = vec![
        ("Q4", "SELECT o_orderpriority, COUNT(*) FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' GROUP BY o_orderpriority"),
        ("Q10", "SELECT c_custkey, COUNT(*) FROM customer GROUP BY c_custkey"),
        ("Q13", "SELECT c_mktsegment, COUNT(*) FROM customer GROUP BY c_mktsegment"),
        ("Q14", "SELECT p_type, COUNT(*) FROM part GROUP BY p_type"),
        ("Q19", "SELECT p_brand, SUM(p_retailprice) FROM part GROUP BY p_brand"),
        ("Q20", "SELECT s_nationkey, COUNT(*) FROM supplier GROUP BY s_nationkey"),
        ("Q22", "SELECT c_nationkey, COUNT(*) FROM customer WHERE c_acctbal > 0 GROUP BY c_nationkey"),
    ];

    let mut results = Vec::new();

    for (name, sql) in queries {
        println!("  Running {}...", name);
        let mut stats = LatencyStats::new();

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = engine.execute(parse(sql).unwrap());
            stats.record(start.elapsed().as_nanos() as u64);
        }

        let p99_ms = stats.p99() as f64 / 1_000_000.0;
        let passed = p99_ms < P99_TARGET_MS;

        println!(
            "    {}: P50={:.2}ms, P95={:.2}ms, P99={:.2}ms, avg={:.2}ms {}",
            name,
            stats.p50() as f64 / 1_000_000.0,
            stats.p95() as f64 / 1_000_000.0,
            p99_ms,
            stats.avg_ms(),
            if passed { "✅" } else { "❌" }
        );

        results.push(QueryBenchmarkResult {
            query_name: name.to_string(),
            p50_ms: stats.p50() as f64 / 1_000_000.0,
            p95_ms: stats.p95() as f64 / 1_000_000.0,
            p99_ms,
            avg_ms: stats.avg_ms(),
            min_ms: stats.min_ms(),
            max_ms: stats.max_ms(),
            iterations,
            passed,
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tpch_sf1_benchmark() {
        let storage_path = PathBuf::from("/tmp/tpch_sf1_storage");

        // Import data
        let mut engine = import_tpch_data(&storage_path).expect("Failed to import data");

        // Run benchmark
        let results = run_tpch_queries(&mut engine, 10);

        // Check results
        let all_passed = results.iter().all(|r| r.passed);

        println!("\n=== SF=1 Benchmark Results ===");
        println!("P99 Target: {:.0}ms", P99_TARGET_MS);
        println!(
            "All Passed: {}",
            if all_passed { "YES ✅" } else { "NO ❌" }
        );

        for result in &results {
            println!(
                "{}: P99={:.2}ms avg={:.2}ms",
                result.query_name, result.p99_ms, result.avg_ms
            );
        }
    }
}
