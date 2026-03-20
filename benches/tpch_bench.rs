//! TPC-H Benchmark Suite
//!
//! This module provides TPC-H style benchmarks for SQLRustGo.
//! Supports data generation, query execution, and comparison with PostgreSQL/SQLite.
//!
//! # Usage
//!
//! ```bash
//! # Run all TPC-H benchmarks
//! cargo bench --bench tpch_bench
//!
//! # Run specific query
//! cargo bench --bench tpch_bench -- Q1
//!
//! # Run with comparison
//! cargo run --example tpch_compare
//! ```

use criterion::{criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};
use sqlrustgo::{parse, ExecutionEngine};
use sqlrustgo_storage::MemoryStorage;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    samples: Vec<u64>,
    sorted: bool,
}

impl LatencyStats {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            sorted: false,
        }
    }

    pub fn record(&mut self, latency_us: u64) {
        self.samples.push(latency_us);
        self.sorted = false;
    }

    fn sort(&mut self) {
        if !self.sorted {
            self.samples.sort();
            self.sorted = true;
        }
    }

    fn percentile(&self, p: f64) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        let idx = ((self.samples.len() - 1) as f64 * p) as usize;
        self.samples[idx.min(self.samples.len() - 1)]
    }

    pub fn count(&self) -> usize {
        self.samples.len()
    }

    pub fn min(&self) -> Option<u64> {
        self.samples.iter().min().copied()
    }

    pub fn max(&self) -> Option<u64> {
        self.samples.iter().max().copied()
    }

    pub fn avg(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.samples.iter().sum::<u64>() as f64 / self.samples.len() as f64
    }

    pub fn p50(&self) -> u64 {
        self.percentile(0.50)
    }

    pub fn p95(&self) -> u64 {
        self.percentile(0.95)
    }

    pub fn p99(&self) -> u64 {
        self.percentile(0.99)
    }

    pub fn median(&self) -> u64 {
        self.p50()
    }
}

impl Default for LatencyStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub timestamp: String,
    pub version: String,
    pub workload: String,
    pub scale_factor: f64,
    pub queries: Vec<QueryResult>,
    pub comparison: Option<ComparisonResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub name: String,
    pub avg_latency_ms: f64,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub min_ms: Option<u64>,
    pub max_ms: Option<u64>,
    pub iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub system: String,
    pub queries: Vec<QueryResult>,
}

impl BenchmarkResult {
    pub fn new(workload: String, scale_factor: f64) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            workload,
            scale_factor,
            queries: Vec::new(),
            comparison: None,
        }
    }

    pub fn add_query(&mut self, result: QueryResult) {
        self.queries.push(result);
    }

    pub fn set_comparison(&mut self, comparison: ComparisonResult) {
        self.comparison = Some(comparison);
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn print_json(&self) {
        println!("{}", serde_json::to_string_pretty(self).unwrap());
    }

    pub fn print_summary(&self) {
        println!("\n=== TPC-H Benchmark Summary ===");
        println!("Version: {}", self.version);
        println!("Scale Factor: {}", self.scale_factor);
        println!("Timestamp: {}", self.timestamp);
        println!();

        println!(
            "{:<10} {:>12} {:>12} {:>12} {:>12}",
            "Query", "Avg(ms)", "P50(ms)", "P95(ms)", "P99(ms)"
        );
        println!("{}", "-".repeat(62));

        for q in &self.queries {
            println!(
                "{:<10} {:>12.2} {:>12} {:>12} {:>12}",
                q.name, q.avg_latency_ms, q.p50_ms, q.p95_ms, q.p99_ms
            );
        }

        if let Some(ref comp) = self.comparison {
            println!();
            println!("=== Comparison: {} ===", comp.system);
            println!(
                "{:<10} {:>12} {:>12} {:>12} {:>12}",
                "Query", "Avg(ms)", "P50(ms)", "P95(ms)", "P99(ms)"
            );
            println!("{}", "-".repeat(62));

            for q in &comp.queries {
                println!(
                    "{:<10} {:>12.2} {:>12} {:>12} {:>12}",
                    q.name, q.avg_latency_ms, q.p50_ms, q.p95_ms, q.p99_ms
                );
            }
        }
    }
}

const SCALE_FACTOR: f64 = 1.0;
const DEFAULT_ITERATIONS: u32 = 10;

struct TpchDataGenerator {
    scale_factor: f64,
}

impl TpchDataGenerator {
    fn new(scale_factor: f64) -> Self {
        Self { scale_factor }
    }

    fn row_count(&self, base_count: usize) -> usize {
        (base_count as f64 * self.scale_factor) as usize
    }

    fn table_row_counts(&self) -> Vec<(&'static str, usize)> {
        vec![
            ("nation", 25),
            ("region", 5),
            ("part", self.row_count(200_000)),
            ("supplier", self.row_count(10_000)),
            ("partsupp", self.row_count(800_000)),
            ("customer", self.row_count(150_000)),
            ("orders", self.row_count(1_500_000)),
            ("lineitem", self.row_count(6_000_000)),
        ]
    }

    fn generate_lineitem_data(&self) -> Vec<LineItemRow> {
        let mut data = Vec::new();
        let num_rows = self.row_count(6000.min(1000));

        for i in 0..num_rows {
            let row = LineItemRow {
                order_key: (i % 10000) as i64 + 1,
                part_key: (i % 200000) as i64 + 1,
                supp_key: (i % 100) as i64 + 1,
                quantity: (i % 50) as f64 + 1.0,
                extended_price: (i % 10000) as f64 + 1.0,
                discount: ((i % 10) as f64) / 100.0,
                tax: ((i % 8) as f64 + 1.0) / 100.0,
                return_flag: if i % 3 == 0 { "R" } else { "N" },
                ship_date: 87600 + (i % 2000) as i32,
            };
            data.push(row);
        }
        data
    }

    fn generate_orders_data(&self) -> Vec<OrdersRow> {
        let mut data = Vec::new();
        let num_rows = self.row_count(1500.min(500));

        for i in 0..num_rows {
            let row = OrdersRow {
                order_key: i as i64 + 1,
                cust_key: (i % 10000) as i64 + 1,
                order_status: if i % 3 == 0 { "F" } else { "O" },
                order_priority: match i % 5 {
                    0 => "1-URGENT",
                    1 => "2-HIGH",
                    2 => "3-MEDIUM",
                    3 => "4-LOW",
                    _ => "5-NOT SPECIFIED",
                },
                total_price: (i as f64 * 10.0).round() as i64,
                order_date: 87600 + (i % 2000) as i32,
            };
            data.push(row);
        }
        data
    }

    fn generate_customer_data(&self) -> Vec<CustomerRow> {
        let mut data = Vec::new();
        let num_rows = self.row_count(150.min(50));

        for i in 0..num_rows {
            let row = CustomerRow {
                cust_key: i as i64 + 1,
                name: format!("Customer{:05}", i),
                nation_key: (i % 25) as i32,
            };
            data.push(row);
        }
        data
    }
}

#[derive(Debug, Clone)]
struct LineItemRow {
    order_key: i64,
    part_key: i64,
    supp_key: i64,
    quantity: f64,
    extended_price: f64,
    discount: f64,
    tax: f64,
    return_flag: &'static str,
    ship_date: i32,
}

#[derive(Debug, Clone)]
struct OrdersRow {
    order_key: i64,
    cust_key: i64,
    order_status: &'static str,
    order_priority: &'static str,
    total_price: i64,
    order_date: i32,
}

#[derive(Debug, Clone)]
struct CustomerRow {
    cust_key: i64,
    name: String,
    nation_key: i32,
}

fn create_lineitem_schema(engine: &mut ExecutionEngine<MemoryStorage>) {
    engine
        .execute(
            parse(
                "CREATE TABLE lineitem (
            l_orderkey INTEGER,
            l_partkey INTEGER,
            l_suppkey INTEGER,
            l_quantity REAL,
            l_extendedprice REAL,
            l_discount REAL,
            l_tax REAL,
            l_returnflag TEXT,
            l_shipdate INTEGER
        )",
            )
            .unwrap(),
        )
        .unwrap();
}

fn create_orders_schema(engine: &mut ExecutionEngine<MemoryStorage>) {
    engine
        .execute(
            parse(
                "CREATE TABLE orders (
            o_orderkey INTEGER,
            o_custkey INTEGER,
            o_orderstatus TEXT,
            o_orderpriority TEXT,
            o_totalprice INTEGER,
            o_orderdate INTEGER
        )",
            )
            .unwrap(),
        )
        .unwrap();
}

fn create_customer_schema(engine: &mut ExecutionEngine<MemoryStorage>) {
    engine
        .execute(
            parse(
                "CREATE TABLE customer (
            c_custkey INTEGER,
            c_name TEXT,
            c_nationkey INTEGER
        )",
            )
            .unwrap(),
        )
        .unwrap();
}

fn insert_lineitem_data(engine: &mut ExecutionEngine<MemoryStorage>, data: &[LineItemRow]) {
    for row in data {
        let sql = format!(
            "INSERT INTO lineitem VALUES ({}, {}, {}, {}, {}, {}, {}, '{}', {})",
            row.order_key,
            row.part_key,
            row.supp_key,
            row.quantity as i32,
            row.extended_price as i32,
            (row.discount * 100.0) as i32,
            (row.tax * 100.0) as i32,
            row.return_flag,
            row.ship_date
        );
        let _ = engine.execute(parse(&sql).unwrap());
    }
}

fn insert_orders_data(engine: &mut ExecutionEngine<MemoryStorage>, data: &[OrdersRow]) {
    for row in data {
        let sql = format!(
            "INSERT INTO orders VALUES ({}, {}, '{}', '{}', {}, {})",
            row.order_key,
            row.cust_key,
            row.order_status,
            row.order_priority,
            row.total_price,
            row.order_date
        );
        let _ = engine.execute(parse(&sql).unwrap());
    }
}

fn insert_customer_data(engine: &mut ExecutionEngine<MemoryStorage>, data: &[CustomerRow]) {
    for row in data {
        let sql = format!(
            "INSERT INTO customer VALUES ({}, '{}', {})",
            row.cust_key, row.name, row.nation_key
        );
        let _ = engine.execute(parse(&sql).unwrap());
    }
}

fn run_query_benchmark(
    name: &str,
    sql: &str,
    engine: &mut ExecutionEngine<MemoryStorage>,
    iterations: u32,
) -> QueryResult {
    let mut latencies = LatencyStats::new();

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = engine.execute(parse(sql).unwrap());
        latencies.record(start.elapsed().as_micros() as u64);
    }

    QueryResult {
        name: name.to_string(),
        avg_latency_ms: latencies.avg() / 1000.0,
        p50_ms: latencies.p50() / 1000,
        p95_ms: latencies.p95() / 1000,
        p99_ms: latencies.p99() / 1000,
        min_ms: latencies.min().map(|v| v / 1000),
        max_ms: latencies.max().map(|v| v / 1000),
        iterations,
    }
}

fn bench_tpch_q1(c: &mut Criterion) {
    let generator = TpchDataGenerator::new(SCALE_FACTOR);
    let lineitem_data = generator.generate_lineitem_data();

    let mut group = c.benchmark_group("tpch_q1");

    group.bench_function("pricing_summary", |b| {
        b.iter(|| {
            let storage = Arc::new(MemoryStorage::new());
            let mut engine = ExecutionEngine::new(storage);
            create_lineitem_schema(&mut engine);
            insert_lineitem_data(&mut engine, &lineitem_data);

            let sql = "SELECT l_returnflag, SUM(l_quantity) as sum_qty, \
                SUM(l_extendedprice) as sum_base_price, \
                AVG(l_quantity) as avg_qty \
                FROM lineitem WHERE l_returnflag = 'N' \
                GROUP BY l_returnflag";
            let _ = engine.execute(parse(sql).unwrap());
        });
    });

    group.finish();
}

fn bench_tpch_q3(c: &mut Criterion) {
    let generator = TpchDataGenerator::new(SCALE_FACTOR);
    let orders_data = generator.generate_orders_data();
    let lineitem_data = generator.generate_lineitem_data();

    let mut group = c.benchmark_group("tpch_q3");

    group.bench_function("shipping_priority", |b| {
        b.iter(|| {
            let storage = Arc::new(MemoryStorage::new());
            let mut engine = ExecutionEngine::new(storage);
            create_orders_schema(&mut engine);
            create_lineitem_schema(&mut engine);
            insert_orders_data(&mut engine, &orders_data);
            insert_lineitem_data(&mut engine, &lineitem_data);

            let sql = "SELECT o_orderkey, o_orderdate, o_totalprice \
                FROM orders WHERE o_orderdate > 88000";
            let _ = engine.execute(parse(sql).unwrap());
        });
    });

    group.finish();
}

fn bench_tpch_q6(c: &mut Criterion) {
    let generator = TpchDataGenerator::new(SCALE_FACTOR);
    let lineitem_data = generator.generate_lineitem_data();

    let mut group = c.benchmark_group("tpch_q6");

    group.bench_function("revenue_query", |b| {
        b.iter(|| {
            let storage = Arc::new(MemoryStorage::new());
            let mut engine = ExecutionEngine::new(storage);
            create_lineitem_schema(&mut engine);
            insert_lineitem_data(&mut engine, &lineitem_data);

            let sql = "SELECT SUM(l_extendedprice * (1 - l_discount / 100.0)) as revenue \
                FROM lineitem WHERE l_quantity > 20";
            let _ = engine.execute(parse(sql).unwrap());
        });
    });

    group.finish();
}

fn bench_tpch_q10(c: &mut Criterion) {
    let generator = TpchDataGenerator::new(SCALE_FACTOR);
    let customer_data = generator.generate_customer_data();
    let orders_data = generator.generate_orders_data();
    let lineitem_data = generator.generate_lineitem_data();

    let mut group = c.benchmark_group("tpch_q10");

    group.bench_function("customer_revenue", |b| {
        b.iter(|| {
            let storage = Arc::new(MemoryStorage::new());
            let mut engine = ExecutionEngine::new(storage);
            create_customer_schema(&mut engine);
            create_orders_schema(&mut engine);
            create_lineitem_schema(&mut engine);
            insert_customer_data(&mut engine, &customer_data);
            insert_orders_data(&mut engine, &orders_data);
            insert_lineitem_data(&mut engine, &lineitem_data);

            let sql = "SELECT c_custkey, SUM(l_extendedprice) as revenue \
                FROM customer \
                JOIN orders ON c_custkey = o_custkey \
                JOIN lineitem ON o_orderkey = l_orderkey \
                WHERE o_orderdate >= 87800 \
                GROUP BY c_custkey";
            let _ = engine.execute(parse(sql).unwrap());
        });
    });

    group.finish();
}

fn bench_aggregation(c: &mut Criterion) {
    let storage = Arc::new(MemoryStorage::new());
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute(parse("CREATE TABLE sales (id INTEGER, amount REAL, category TEXT)").unwrap())
        .unwrap();

    for i in 0..1000 {
        let sql = format!(
            "INSERT INTO sales VALUES ({}, {}, 'cat{}')",
            i,
            i as f64 * 10.0,
            i % 10
        );
        let _ = engine.execute(parse(&sql).unwrap());
    }

    let mut group = c.benchmark_group("aggregation");

    group.bench_function("sum_amount", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT SUM(amount) FROM sales").unwrap())
                .unwrap()
        });
    });

    group.bench_function("avg_amount", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT AVG(amount) FROM sales").unwrap())
                .unwrap()
        });
    });

    group.bench_function("count_all", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT COUNT(*) FROM sales").unwrap())
                .unwrap()
        });
    });

    group.finish();
}

fn bench_join(c: &mut Criterion) {
    let storage = Arc::new(MemoryStorage::new());
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute(parse("CREATE TABLE customers (id INTEGER, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, customer_id INTEGER, amount REAL)").unwrap(),
        )
        .unwrap();

    for i in 0..100 {
        let sql = format!("INSERT INTO customers VALUES ({}, 'customer{}')", i, i);
        let _ = engine.execute(parse(&sql).unwrap());
    }

    for i in 0..500 {
        let sql = format!(
            "INSERT INTO orders VALUES ({}, {}, {})",
            i,
            i % 100,
            i as f64 * 10.0
        );
        let _ = engine.execute(parse(&sql).unwrap());
    }

    let mut group = c.benchmark_group("join");

    group.bench_function("inner_join", |b| {
        b.iter(|| {
            engine.execute(parse(
                "SELECT c.name, o.amount FROM customers c JOIN orders o ON c.id = o.customer_id"
            ).unwrap()).unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tpch_q1,
    bench_tpch_q3,
    bench_tpch_q6,
    bench_tpch_q10,
    bench_aggregation,
    bench_join
);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_stats() {
        let mut stats = LatencyStats::new();
        stats.record(100);
        stats.record(200);
        stats.record(300);

        assert_eq!(stats.count(), 3);
        assert_eq!(stats.min(), Some(100));
        assert_eq!(stats.max(), Some(300));
        assert_eq!(stats.avg(), 200.0);
    }

    #[test]
    fn test_benchmark_result_json() {
        let mut result = BenchmarkResult::new("tpch".to_string(), 1.0);
        result.add_query(QueryResult {
            name: "Q1".to_string(),
            avg_latency_ms: 10.5,
            p50_ms: 10,
            p95_ms: 15,
            p99_ms: 20,
            min_ms: Some(8),
            max_ms: Some(25),
            iterations: 10,
        });

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Q1"));
    }
}
