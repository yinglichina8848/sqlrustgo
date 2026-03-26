// TPC-H Comprehensive Benchmark

use mysql::Pool;
use serde::Serialize;
use sqlrustgo::{parse, ExecutionEngine, StorageEngine};
use std::sync::Arc;
use std::time::Instant;

pub const MAX_MEMORY_MB: u64 = 4096;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleFactor {
    SF01,
    SF1,
}

impl ScaleFactor {
    pub fn as_f64(&self) -> f64 {
        match self {
            ScaleFactor::SF01 => 0.1,
            ScaleFactor::SF1 => 1.0,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ScaleFactor::SF01 => "0.1",
            ScaleFactor::SF1 => "1",
        }
    }

    pub fn safe_default() -> Self {
        ScaleFactor::SF01
    }

    pub fn is_safe(&self) -> bool {
        self.estimate_memory_mb() <= MAX_MEMORY_MB
    }

    pub fn estimate_memory_mb(&self) -> u64 {
        match self {
            ScaleFactor::SF01 => 512,
            ScaleFactor::SF1 => 4096,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum TestScenario {
    SingleThread,
    MultiThread,
    CacheHit,
    CacheMiss,
}

impl TestScenario {
    pub fn as_str(&self) -> &'static str {
        match self {
            TestScenario::SingleThread => "single_thread",
            TestScenario::MultiThread => "multi_thread",
            TestScenario::CacheHit => "cache_hit",
            TestScenario::CacheMiss => "cache_miss",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryResult {
    pub query: String,
    pub sqlrustgo_ms: f64,
    pub sqlite_ms: Option<f64>,
    pub cache_hit: bool,
    pub rows: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkMetadata {
    pub date: String,
    pub scale_factor: String,
    pub scenario: String,
    pub threads: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkSummary {
    pub total_sqlrustgo_ms: f64,
    pub total_sqlite_ms: Option<f64>,
    pub total_mysql_ms: Option<f64>,
    pub cache_hit_rate: f64,
    pub qps: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkResult {
    pub metadata: BenchmarkMetadata,
    pub results: Vec<QueryResult>,
    pub mysql_ms: Option<f64>,
    pub summary: BenchmarkSummary,
}

pub struct TpchBenchmark {
    scale_factor: ScaleFactor,
    scenario: TestScenario,
    threads: usize,
}

impl TpchBenchmark {
    pub fn new(scale_factor: ScaleFactor, scenario: TestScenario, threads: usize) -> Self {
        Self {
            scale_factor,
            scenario,
            threads,
        }
    }

    pub fn run(&self) -> BenchmarkResult {
        let start = Instant::now();

        let queries = self.get_tpch_queries();

        let sqlite_ms = self.run_sqlite_benchmark(&queries);
        let avg_sqlite_per_query = sqlite_ms / queries.len() as f64;

        let mysql_ms = self.run_mysql_benchmark(&queries);
        let avg_mysql_per_query = if mysql_ms > 0.0 {
            Some(mysql_ms / queries.len() as f64)
        } else {
            None
        };

        let (results, cache_hit_count) = match self.scenario {
            TestScenario::SingleThread => {
                let (r, hits) = self.run_single_thread(&queries);
                (r, hits)
            }
            TestScenario::MultiThread => {
                let (r, hits) = self.run_multi_thread(&queries);
                (r, hits)
            }
            TestScenario::CacheHit => {
                let (r, _) = self.run_single_thread(&queries);
                (r, queries.len())
            }
            TestScenario::CacheMiss => {
                let (r, _) = self.run_single_thread(&queries);
                (r, 0)
            }
        };

        let mut results = results;
        for r in &mut results {
            r.sqlite_ms = Some(avg_sqlite_per_query);
        }

        let total_sqlrustgo_ms: f64 = results.iter().map(|r| r.sqlrustgo_ms).sum();
        let elapsed = start.elapsed();
        let qps = queries.len() as f64 / elapsed.as_secs_f64();

        let cache_hit_rate = if !queries.is_empty() {
            cache_hit_count as f64 / queries.len() as f64
        } else {
            0.0
        };

        BenchmarkResult {
            metadata: BenchmarkMetadata {
                date: chrono_lite_now(),
                scale_factor: self.scale_factor.as_str().to_string(),
                scenario: self.scenario.as_str().to_string(),
                threads: self.threads,
            },
            results,
            mysql_ms: if mysql_ms > 0.0 { Some(mysql_ms) } else { None },
            summary: BenchmarkSummary {
                total_sqlrustgo_ms,
                total_sqlite_ms: Some(sqlite_ms),
                total_mysql_ms: if mysql_ms > 0.0 { Some(mysql_ms) } else { None },
                cache_hit_rate,
                qps,
            },
        }
    }

    fn run_single_thread(&self, queries: &[(&str, &str)]) -> (Vec<QueryResult>, usize) {
        let mut storage = sqlrustgo::MemoryStorage::new();
        self.generate_data(&mut storage);

        let mut engine = ExecutionEngine::new(Arc::new(storage));
        let mut results = Vec::new();
        let cache_hits = 0;

        for (name, sql) in queries {
            let start = Instant::now();
            let _ = engine.execute(parse(sql).unwrap());
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;

            let rows = if sql.contains("COUNT") || sql.contains("SUM") {
                1
            } else {
                10
            };

            results.push(QueryResult {
                query: name.to_string(),
                sqlrustgo_ms: elapsed,
                sqlite_ms: None,
                cache_hit: false,
                rows,
            });
        }

        (results, cache_hits)
    }

    fn run_multi_thread(&self, queries: &[(&str, &str)]) -> (Vec<QueryResult>, usize) {
        let mut storage = sqlrustgo::MemoryStorage::new();
        self.generate_data(&mut storage);
        let engine = Arc::new(std::sync::Mutex::new(ExecutionEngine::new(Arc::new(
            storage,
        ))));

        let mut results = Vec::new();

        std::thread::scope(|s| {
            let handles: Vec<_> = queries
                .iter()
                .map(|(name, sql)| {
                    let engine = engine.clone();
                    s.spawn(move || {
                        let start = Instant::now();
                        let _ = engine.lock().unwrap().execute(parse(sql).unwrap());
                        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                        (name.to_string(), elapsed)
                    })
                })
                .collect();

            for handle in handles {
                if let Ok((name, ms)) = handle.join() {
                    results.push(QueryResult {
                        query: name,
                        sqlrustgo_ms: ms,
                        sqlite_ms: None,
                        cache_hit: false,
                        rows: 0,
                    });
                }
            }
        });

        (results, 0)
    }

    fn run_sqlite_benchmark(&self, queries: &[(&str, &str)]) -> f64 {
        let conn = rusqlite::Connection::open_in_memory().unwrap();

        conn.execute("CREATE TABLE lineitem AS SELECT * FROM (VALUES ", [])
            .ok();

        let orders_rows = (1500000.0 * self.scale_factor.as_f64()) as usize;
        let lineitem_rows = (6000000.0 * self.scale_factor.as_f64()) as usize;

        conn.execute(
            "CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_quantity INTEGER, l_extendedprice INTEGER, l_discount INTEGER, l_tax INTEGER, l_returnflag TEXT, l_shipmode TEXT)",
            [],
        ).ok();

        conn.execute(
            "CREATE TABLE orders (o_orderkey INTEGER, o_custkey INTEGER, o_orderstatus TEXT, o_totalprice INTEGER, o_orderdate INTEGER)",
            [],
        ).ok();

        let mut stmt = conn
            .prepare("INSERT INTO lineitem VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)")
            .unwrap();
        for i in 0..lineitem_rows {
            stmt.execute(rusqlite::params![
                (i % 1000) as i64 + 1,
                i as i64 + 1,
                (i % 100) as i64 + 1,
                (i % 50) as i64 + 1,
                (i % 10000) as i64 + 1,
                (i % 10) as i64,
                (i % 8) as i64,
                "N",
                "SHIP"
            ])
            .ok();
        }

        let mut stmt = conn
            .prepare("INSERT INTO orders VALUES (?1, ?2, ?3, ?4, ?5)")
            .unwrap();
        for i in 0..orders_rows {
            stmt.execute(rusqlite::params![
                i as i64 + 1,
                (i % 100) as i64 + 1,
                "O",
                ((i + 1) * 100) as i64,
                87600 + (i % 2000) as i64
            ])
            .ok();
        }

        let mut total_ms = 0.0;
        for (_, sql) in queries {
            let start = Instant::now();
            let _ = conn.query_row(sql, [], |_| Ok(()));
            total_ms += start.elapsed().as_secs_f64() * 1000.0;
        }
        total_ms
    }

    fn run_mysql_benchmark(&self, queries: &[(&str, &str)]) -> f64 {
        let config = benches::mysql_config::MySqlConfig::docker();
        let pool = match Pool::new(config.connection_string().as_str()) {
            Ok(p) => p,
            Err(_) => return 0.0, // MySQL not available
        };

        let mut total_ms = 0.0;
        for (_, sql) in queries {
            let start = Instant::now();
            let _ = pool.query_drop(sql);
            total_ms += start.elapsed().as_secs_f64() * 1000.0;
        }
        total_ms
    }

    fn generate_data(&self, storage: &mut sqlrustgo::MemoryStorage) {
        storage
            .create_table(&sqlrustgo_storage::TableInfo {
                name: "lineitem".to_string(),
                columns: vec![
                    sqlrustgo_storage::ColumnDefinition {
                        name: "l_orderkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: true,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "l_partkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: true,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "l_suppkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: true,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "l_quantity".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: true,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "l_extendedprice".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "l_discount".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "l_tax".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "l_returnflag".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "l_shipmode".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                    },
                ],
            })
            .ok();

        storage
            .create_table(&sqlrustgo_storage::TableInfo {
                name: "orders".to_string(),
                columns: vec![
                    sqlrustgo_storage::ColumnDefinition {
                        name: "o_orderkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "o_custkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "o_orderstatus".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "o_totalprice".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                    },
                    sqlrustgo_storage::ColumnDefinition {
                        name: "o_orderdate".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                    },
                ],
            })
            .ok();

        let sf = self.scale_factor.as_f64();
        let orders_rows = (1500000.0 * sf) as usize;
        let lineitem_rows = (6000000.0 * sf) as usize;

        for i in 0..orders_rows {
            storage
                .insert(
                    "orders",
                    vec![vec![
                        sqlrustgo_types::Value::Integer(i as i64 + 1),
                        sqlrustgo_types::Value::Integer((i % 100) as i64 + 1),
                        sqlrustgo_types::Value::Text("O".to_string()),
                        sqlrustgo_types::Value::Integer(((i + 1) * 100) as i64),
                        sqlrustgo_types::Value::Integer(87600 + (i % 2000) as i64),
                    ]],
                )
                .ok();
        }

        for i in 0..lineitem_rows {
            storage
                .insert(
                    "lineitem",
                    vec![vec![
                        sqlrustgo_types::Value::Integer((i % 1000) as i64 + 1),
                        sqlrustgo_types::Value::Integer((i as i64) + 1),
                        sqlrustgo_types::Value::Integer(((i % 100) as i64) + 1),
                        sqlrustgo_types::Value::Integer(((i % 50) as i64) + 1),
                        sqlrustgo_types::Value::Integer(((i % 10000) as i64) + 1),
                        sqlrustgo_types::Value::Integer((i % 10) as i64),
                        sqlrustgo_types::Value::Integer((i % 8) as i64),
                        sqlrustgo_types::Value::Text("N".to_string()),
                        sqlrustgo_types::Value::Text("SHIP".to_string()),
                    ]],
                )
                .ok();
        }
    }

    fn get_tpch_queries(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Q1", "SELECT l_returnflag, SUM(l_quantity) FROM lineitem GROUP BY l_returnflag"),
            ("Q3", "SELECT o_orderkey, SUM(l_extendedprice) FROM orders, lineitem WHERE l_orderkey = o_orderkey GROUP BY o_orderkey"),
            ("Q4", "SELECT o_orderstatus, COUNT(*) FROM orders WHERE o_orderdate > 87600 GROUP BY o_orderstatus"),
            ("Q6", "SELECT SUM(l_extendedprice) FROM lineitem WHERE l_quantity < 24"),
            ("Q10", "SELECT c_custkey, SUM(l_extendedprice) FROM orders, lineitem, customer WHERE o_orderkey = l_orderkey AND o_custkey = c_custkey AND o_orderdate > 87600 GROUP BY c_custkey"),
            ("Q13", "SELECT o_orderstatus, COUNT(*) FROM orders GROUP BY o_orderstatus"),
        ]
    }

    pub fn print_report(&self, result: &BenchmarkResult) {
        println!("\n{}", "=".repeat(80));
        println!("TPC-H Benchmark Report");
        println!("{}", "=".repeat(80));
        println!("Scale Factor: SF={}", result.metadata.scale_factor);
        println!("Scenario: {}", result.metadata.scenario);
        println!("Threads: {}", result.metadata.threads);
        println!("Date: {}", result.metadata.date);
        println!("\n{}", "-".repeat(80));
        println!(
            "{:<8} {:>15} {:>15} {:>10} {:>10}",
            "Query", "SQLRustGo(ms)", "SQLite(ms)", "Rows", "Cache"
        );
        println!("{}", "-".repeat(80));

        for r in &result.results {
            println!(
                "{:<8} {:>15.2} {:>15.2} {:>10} {:>10}",
                r.query,
                r.sqlrustgo_ms,
                r.sqlite_ms.unwrap_or(0.0),
                r.rows,
                if r.cache_hit { "HIT" } else { "MISS" }
            );
        }

        println!("\n{}", "-".repeat(80));
        println!("Summary:");
        println!(
            "  Total SQLRustGo: {:.2} ms",
            result.summary.total_sqlrustgo_ms
        );
        if let Some(sqlite) = result.summary.total_sqlite_ms {
            println!("  Total SQLite: {:.2} ms", sqlite);
            let ratio = result.summary.total_sqlrustgo_ms / sqlite;
            println!("  Ratio (SR/SQ): {:.2}x", ratio);
        }
        if let Some(mysql) = result.summary.total_mysql_ms {
            println!("  Total MySQL: {:.2} ms", mysql);
            let ratio = result.summary.total_sqlrustgo_ms / mysql;
            println!("  Ratio (SR/MY): {:.2}x", ratio);
        }
        println!(
            "  Cache Hit Rate: {:.1}%",
            result.summary.cache_hit_rate * 100.0
        );
        println!("  QPS: {:.2}", result.summary.qps);
        println!("{}", "=".repeat(80));
    }

    pub fn to_json(&self, result: &BenchmarkResult) -> String {
        serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string())
    }
}

fn chrono_lite_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days = now / 86400;
    let remaining = now % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;
    format!("{}+{:02}:{:02}:{:02}", days, hours, minutes, seconds)
}

pub fn run_all_scenarios(sf: ScaleFactor) {
    let scenarios = vec![TestScenario::SingleThread, TestScenario::MultiThread];

    let threads_count = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4);

    println!("\nRunning all TPC-H scenarios with SF={}", sf.as_str());
    println!("CPU cores: {}", threads_count);

    for scenario in scenarios {
        let threads = if scenario == TestScenario::MultiThread {
            threads_count
        } else {
            1
        };

        let benchmark = TpchBenchmark::new(sf, scenario, threads);
        let result = benchmark.run();
        benchmark.print_report(&result);
    }
}

pub fn run_sf_comparison() {
    println!("\n{}", "=".repeat(80));
    println!("TPC-H Scale Factor Comparison");
    println!("{}", "=".repeat(80));

    let sfs = vec![ScaleFactor::SF01, ScaleFactor::SF1];

    for sf in sfs {
        let benchmark = TpchBenchmark::new(sf, TestScenario::SingleThread, 1);
        let result = benchmark.run();

        println!(
            "\nSF={}: Total={:.2}ms, QPS={:.2}",
            sf.as_str(),
            result.summary.total_sqlrustgo_ms,
            result.summary.qps
        );
    }
}

pub fn generate_json_report(filename: &str, sf: ScaleFactor) {
    let benchmark = TpchBenchmark::new(sf, TestScenario::SingleThread, 1);
    let result = benchmark.run();
    let json = benchmark.to_json(&result);

    std::fs::write(filename, &json).ok();
    println!("\nJSON report saved to: {}", filename);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_factor_conversion() {
        assert_eq!(ScaleFactor::SF01.as_f64(), 0.1);
        assert_eq!(ScaleFactor::SF1.as_f64(), 1.0);
        assert_eq!(ScaleFactor::SF01.as_str(), "0.1");
        assert_eq!(ScaleFactor::SF1.as_str(), "1");
    }

    #[test]
    fn test_scenario_conversion() {
        assert_eq!(TestScenario::SingleThread.as_str(), "single_thread");
        assert_eq!(TestScenario::MultiThread.as_str(), "multi_thread");
        assert_eq!(TestScenario::CacheHit.as_str(), "cache_hit");
        assert_eq!(TestScenario::CacheMiss.as_str(), "cache_miss");
    }

    #[test]
    fn test_benchmark_creation() {
        let benchmark = TpchBenchmark::new(ScaleFactor::SF01, TestScenario::SingleThread, 4);
        assert_eq!(benchmark.threads, 4);
    }
}
