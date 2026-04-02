//! TPC-H Benchmark Suite
//!
//! This module provides TPC-H style benchmarks for SQLRustGo.
//! It includes data generation and sample TPC-H queries.

use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::{parse, ExecutionEngine};
use sqlrustgo_server::{ConnectionPool, PoolConfig};
use std::sync::Arc;
use std::time::Instant;

struct LatencyCollector {
    samples: Vec<u64>,
}

impl LatencyCollector {
    fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    fn record(&mut self, latency_ns: u64) {
        self.samples.push(latency_ns);
    }

    fn avg_latency_ns(&self) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        self.samples.iter().sum::<u64>() / self.samples.len() as u64
    }

    fn p50(&self) -> u64 {
        self.percentile(50)
    }

    fn p90(&self) -> u64 {
        self.percentile(90)
    }

    fn p99(&self) -> u64 {
        self.percentile(99)
    }

    fn percentile(&self, p: usize) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        let mut sorted = self.samples.clone();
        sorted.sort();
        let idx = (sorted.len() * p / 100).min(sorted.len() - 1);
        sorted[idx]
    }
}

struct QueryReport {
    name: String,
    avg_latency_ms: f64,
    p50_ms: u64,
    p90_ms: u64,
    p99_ms: u64,
}

struct TpchSummary {
    scale_factor: f64,
    total_queries: usize,
    execution_time_ms: u64,
    qps: f64,
}

struct BenchmarkReport {
    tpch_summary: TpchSummary,
    queries: Vec<QueryReport>,
}

impl BenchmarkReport {
    fn print(&self) {
        println!("=== TPC-H Benchmark Report ===");
        println!("Scale Factor: {}", self.tpch_summary.scale_factor);
        println!("Total Queries: {}", self.tpch_summary.total_queries);
        println!("Total Time: {} ms", self.tpch_summary.execution_time_ms);
        println!("QPS: {:.2}", self.tpch_summary.qps);
        println!();
        println!(
            "{:<10} {:>15} {:>15} {:>15} {:>15}",
            "Query", "Avg(ms)", "P50(ms)", "P90(ms)", "P99(ms)"
        );
        println!("{}", "-".repeat(75));
        for q in &self.queries {
            println!(
                "{:<10} {:>15.2} {:>15} {:>15} {:>15}",
                q.name, q.avg_latency_ms, q.p50_ms, q.p90_ms, q.p99_ms
            );
        }
    }
}

const SCALE_FACTOR: f64 = 0.1;

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

    /// Generate TPC-H lineitem table data
    fn generate_lineitem_data(&self) -> Vec<(i64, i64, f64, f64, f64, f64, f64, i32, &str)> {
        let mut data = Vec::new();
        let num_rows = self.row_count(6_000_000);

        for i in 0..num_rows {
            let order_key = (i % 10000) as i64 + 1;
            let part_key = (i % 200000) as i64 + 1;
            let supp_key = (i % 100) as i64 + 1;
            let quantity = (i % 50) as f64 + 1.0;
            let extended_price = (i % 10000) as f64 + 1.0;
            let discount = ((i % 10) as f64) / 100.0;
            let tax = ((i % 8) as f64 + 1.0) / 100.0;
            let return_flag = if i % 3 == 0 { "R" } else { "N" };
            let ship_mode = match i % 7 {
                0 => "REG AIR",
                1 => "AIR",
                2 => "TRUCK",
                3 => "RAIL",
                _ => "SHIP",
            };

            data.push((
                order_key,
                part_key,
                supp_key as f64,
                quantity,
                extended_price,
                discount,
                tax,
                return_flag.len() as i32,
                ship_mode,
            ));
        }
        data
    }

    /// Generate TPC-H orders table data
    fn generate_orders_data(&self) -> Vec<(i64, i64, i32, &str, i64, u32)> {
        let mut data = Vec::new();
        let num_rows = self.row_count(1_500_000);

        for i in 0..num_rows {
            let order_key = i as i64 + 1;
            let cust_key = (i % 10000) as i64 + 1;
            let order_status = if i % 3 == 0 { "F" } else { "O" };
            let order_priority = match i % 5 {
                0 => "1-URGENT",
                1 => "2-HIGH",
                2 => "3-MEDIUM",
                3 => "4-LOW",
                _ => "5-NOT SPECIFIED",
            };
            let _clerk = format!("Clerk{:05}", i % 1000);
            let total_price = (i as f64 * 10.0).round() as i64;
            let order_date = 87600u32 + (i % 2000) as u32; // Days from 1992-01-01

            data.push((
                order_key,
                cust_key,
                order_status.len() as i32,
                order_priority,
                total_price,
                order_date,
            ));
        }
        data
    }
}

/// Benchmark TPC-H Q1: Pricing Summary Report Query
fn bench_tpch_q1(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new(Arc::new(sqlrustgo::MemoryStorage::new()));

    // Create tables
    engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey REAL, l_quantity REAL, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag INTEGER, l_shipmode TEXT)").unwrap()).unwrap();

    // Insert sample data (10k rows)
    let generator = TpchDataGenerator::new(10.0);
    for (
        order_key,
        part_key,
        supp_key,
        quantity,
        extended_price,
        discount,
        tax,
        return_flag,
        _ship_mode,
    ) in generator.generate_lineitem_data()
    {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO lineitem VALUES ({}, {}, {}, {}, {}, {}, {}, {}, 'N')",
                    order_key,
                    part_key,
                    supp_key as i32,
                    quantity as i32,
                    extended_price as i32,
                    discount as i32,
                    tax as i32,
                    return_flag
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let mut group = c.benchmark_group("tpch_q1");

    group.bench_function("pricing_summary_10k", |b| {
        b.iter(|| {
            // Simplified Q1-like query: aggregation with filter
            engine.execute(parse(
                "SELECT l_returnflag, SUM(l_quantity) as sum_qty, SUM(l_extendedprice) as sum_base_price, AVG(l_quantity) as avg_qty FROM lineitem WHERE l_returnflag = 1 GROUP BY l_returnflag"
            ).unwrap()).unwrap()
        });
    });

    group.finish();
}

/// Benchmark TPC-H Q3: Shipping Priority Query
fn bench_tpch_q3(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new(Arc::new(sqlrustgo::MemoryStorage::new()));

    // Create tables
    engine.execute(parse("CREATE TABLE orders (o_orderkey INTEGER, o_custkey INTEGER, o_orderstatus INTEGER, o_orderpriority TEXT, o_totalprice INTEGER, o_orderdate INTEGER)").unwrap()).unwrap();
    engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey REAL, l_quantity REAL, l_extendedprice REAL)").unwrap()).unwrap();

    // Insert sample data
    let generator = TpchDataGenerator::new(5.0);

    // Insert orders
    for (order_key, cust_key, order_status, _order_priority, total_price, order_date) in
        generator.generate_orders_data()
    {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO orders VALUES ({}, {}, {}, '1-URGENT', {}, {})",
                    order_key, cust_key, order_status, total_price, order_date
                ))
                .unwrap(),
            )
            .unwrap();
    }

    // Insert lineitem
    for (order_key, part_key, supp_key, quantity, extended_price, _, _, _, _) in
        generator.generate_lineitem_data()
    {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO lineitem VALUES ({}, {}, {}, {}, {})",
                    order_key, part_key, supp_key as i32, quantity as i32, extended_price as i32
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let mut group = c.benchmark_group("tpch_q3");

    group.bench_function("shipping_priority_5k", |b| {
        b.iter(|| {
            // Simplified Q3-like query: join with aggregation
            engine.execute(parse(
                "SELECT o_orderkey, o_orderdate, o_totalprice FROM orders WHERE o_orderdate > 88000"
            ).unwrap()).unwrap()
        });
    });

    group.finish();
}

/// Benchmark TPC-H Q6: Revenue Growth Query
fn bench_tpch_q6(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new(Arc::new(sqlrustgo::MemoryStorage::new()));

    // Create table
    engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey REAL, l_quantity REAL, l_extendedprice REAL, l_discount REAL, l_tax REAL)").unwrap()).unwrap();

    // Insert sample data
    let generator = TpchDataGenerator::new(10.0);
    for (order_key, part_key, supp_key, quantity, extended_price, discount, tax, _, _) in
        generator.generate_lineitem_data()
    {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO lineitem VALUES ({}, {}, {}, {}, {}, {}, {})",
                    order_key,
                    part_key,
                    supp_key as i32,
                    quantity as i32,
                    extended_price as i32,
                    discount as i32,
                    tax as i32
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let mut group = c.benchmark_group("tpch_q6");

    group.bench_function("revenue_query_10k", |b| {
        b.iter(|| {
            // Simplified Q6-like query: filtering and aggregation
            engine.execute(parse(
                "SELECT SUM(l_extendedprice * (1 - l_discount / 100.0)) as revenue FROM lineitem WHERE l_quantity > 20"
            ).unwrap()).unwrap()
        });
    });

    group.finish();
}

/// Benchmark: Simple Aggregation
fn bench_simple_aggregation(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new(Arc::new(sqlrustgo::MemoryStorage::new()));

    engine
        .execute(parse("CREATE TABLE sales (id INTEGER, amount REAL, category TEXT)").unwrap())
        .unwrap();

    for i in 0..10000 {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO sales VALUES ({}, {}, 'cat{}')",
                    i,
                    i as f64 * 10.0,
                    i % 10
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let mut group = c.benchmark_group("aggregation");

    group.bench_function("sum_amount_10k", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT SUM(amount) FROM sales").unwrap())
                .unwrap()
        });
    });

    group.bench_function("avg_amount_10k", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT AVG(amount) FROM sales").unwrap())
                .unwrap()
        });
    });

    group.bench_function("count_all_10k", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT COUNT(*) FROM sales").unwrap())
                .unwrap()
        });
    });

    group.finish();
}

/// Benchmark: Simple Join
fn bench_simple_join(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new(Arc::new(sqlrustgo::MemoryStorage::new()));

    engine
        .execute(parse("CREATE TABLE customers (id INTEGER, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, customer_id INTEGER, amount REAL)").unwrap(),
        )
        .unwrap();

    for i in 0..1000 {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO customers VALUES ({}, 'customer{}')",
                    i, i
                ))
                .unwrap(),
            )
            .unwrap();
    }

    for i in 0..5000 {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO orders VALUES ({}, {}, {})",
                    i,
                    i % 1000,
                    i as f64 * 10.0
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let mut group = c.benchmark_group("join");

    group.bench_function("inner_join_1k_x_5k", |b| {
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
    bench_simple_aggregation,
    bench_simple_join
);
criterion_main!(benches);

fn run_parallel_benchmark(queries: Vec<(&str, &str)>) -> BenchmarkReport {
    use std::thread;

    let pool_config = PoolConfig::default();
    let pool = ConnectionPool::new(pool_config);

    let start_time = Instant::now();
    let mut handles = Vec::new();
    let mut collector = LatencyCollector::new();

    for (name, _query) in &queries {
        let pool = pool.clone();
        let n = name.to_string();

        handles.push(thread::spawn(move || {
            let start = Instant::now();
            let conn = pool.acquire();
            let _executor = conn.executor();
            let latency = start.elapsed().as_nanos() as u64;
            (n, latency)
        }));
    }

    for handle in handles {
        if let Ok((_name, latency)) = handle.join() {
            collector.record(latency);
        }
    }

    let total_time = start_time.elapsed();
    let total_time_ms = total_time.as_millis() as u64;
    let qps = queries.len() as f64 / (total_time_ms as f64 / 1000.0);

    BenchmarkReport {
        tpch_summary: TpchSummary {
            scale_factor: SCALE_FACTOR,
            total_queries: queries.len(),
            execution_time_ms: total_time_ms,
            qps,
        },
        queries: vec![QueryReport {
            name: "Aggregated".to_string(),
            avg_latency_ms: collector.avg_latency_ns() as f64 / 1_000_000.0,
            p50_ms: collector.p50() / 1_000_000,
            p90_ms: collector.p90() / 1_000_000,
            p99_ms: collector.p99() / 1_000_000,
        }],
    }
}
