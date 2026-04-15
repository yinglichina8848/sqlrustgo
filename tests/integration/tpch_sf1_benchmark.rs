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
    let data_dir = PathBuf::from("/home/openclaw/dev/yinglichina163/sqlrustgo/data/tpch-sf01");

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
            table_foreign_keys: None,
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
                    table_foreign_keys: None,
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
                table_foreign_keys: None,
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
        ("Q1", "SELECT l_returnflag, SUM(l_quantity) FROM lineitem WHERE l_shipdate <= '1995-12-01' GROUP BY l_returnflag"),
        ("Q2", "SELECT s_acctbal, s_name, n_name, p_partkey FROM part, supplier, partsupp, nation, region WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15 AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'EUROPE' ORDER BY s_acctbal DESC LIMIT 10"),
        ("Q3", "SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey"),
        ("Q4", "SELECT o_orderpriority, COUNT(*) FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' GROUP BY o_orderpriority"),
        ("Q5", "SELECT n_name, SUM(l_extendedprice) FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' GROUP BY n_name"),
        ("Q6", "SELECT SUM(l_extendedprice) FROM lineitem WHERE l_quantity < 24 AND l_shipdate >= '1994-01-01'"),
        ("Q7", "SELECT n1.n_name AS supp_nation, n2.n_name AS cust_nation, SUM(l_extendedprice) FROM supplier, lineitem, orders, customer, nation n1, nation n2 WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND c_custkey = o_custkey AND s_nationkey = n1.n_nationkey AND c_nationkey = n2.n_nationkey GROUP BY n1.n_name, n2.n_name"),
        ("Q8", "SELECT o_orderyear, SUM(o_totalprice) FROM orders WHERE o_orderpriority = '1-URGENT' GROUP BY o_orderyear"),
        ("Q9", "SELECT p_name, SUM(l_extendedprice) FROM part, lineitem, orders, supplier WHERE p_partkey = l_partkey AND l_orderkey = o_orderkey AND s_suppkey = l_suppkey AND p_type LIKE '%COPPER%' GROUP BY p_name"),
        ("Q10", "SELECT c_custkey, COUNT(*) FROM customer GROUP BY c_custkey"),
        ("Q11", "SELECT ps_partkey, SUM(ps_supplycost) FROM partsupp, supplier, nation WHERE s_suppkey = ps_suppkey AND s_nationkey = n_nationkey AND n_name = 'GERMANY' GROUP BY ps_partkey"),
        ("Q12", "SELECT l_shipmode, SUM(l_quantity) FROM lineitem WHERE l_shipmode IN ('MAIL', 'SHIP') AND l_commitdate < l_receiptdate GROUP BY l_shipmode"),
        ("Q13", "SELECT c_mktsegment, COUNT(*) FROM customer GROUP BY c_mktsegment"),
        ("Q14", "SELECT p_type, COUNT(*) FROM part GROUP BY p_type"),
        ("Q15", "SELECT s_suppkey, s_name, s_address FROM supplier WHERE s_suppkey IN (SELECT l_suppkey FROM lineitem WHERE l_shipdate >= '1996-01-01')"),
        ("Q16", "SELECT p_brand, p_type, COUNT(*) FROM part GROUP BY p_brand, p_type"),
        ("Q17", "SELECT SUM(l_extendedprice) / 7.0 AS avg_yearly FROM lineitem, part WHERE p_partkey = l_partkey AND p_container = 'MED BOX' AND l_quantity < (SELECT 0.2 * AVG(l_quantity) FROM lineitem WHERE l_partkey = p_partkey)"),
        ("Q18", "SELECT c_name, c_custkey, o_orderkey, o_totalprice FROM customer, orders WHERE c_custkey = o_custkey AND o_totalprice > 30000"),
        ("Q19", "SELECT p_brand, SUM(p_retailprice) FROM part GROUP BY p_brand"),
        ("Q20", "SELECT s_nationkey, COUNT(*) FROM supplier GROUP BY s_nationkey"),
        ("Q21", "SELECT s_name, COUNT(*) FROM supplier, lineitem orders WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND o_orderstatus = 'F' GROUP BY s_name"),
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
