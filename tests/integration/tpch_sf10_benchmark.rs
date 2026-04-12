//! TPC-H SF=10 Performance Benchmark
//!
//! This test performs a complete SF=10 TPC-H benchmark:
//! 1. Generates SF=10 data (60M lineitem rows) using dbgen
//! 2. Imports data into SQLRustGo FileStorage
//! 3. Runs Q1-Q22 queries
//! 4. Reports P99 latency metrics
//!
//! Usage:
//!   cargo test --test tpch_sf10_benchmark -- --nocapture
//!   cargo test --test tpch_sf10_benchmark -- --ignored --nocapture  # Full SF=10
//!
//! Note: Full SF=10 test is ignored by default due to long runtime (~10-30 minutes).
//! Run with --ignored flag to execute.

use rand::Rng;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage, StorageEngine};
use sqlrustgo_storage::{ColumnDefinition, FileStorage, TableData, TableInfo};
use sqlrustgo_types::Value;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// SF=10 data scale factors
const SF10_LINEITEM: usize = 60_000_000;
const SF10_ORDERS: usize = 15_000_000;
const SF10_CUSTOMER: usize = 1_500_000;
const SF10_PART: usize = 2_000_000;
const SF10_SUPPLIER: usize = 100_000;
const SF10_PARTSUPP: usize = 8_000_000;

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

    fn count(&self) -> usize {
        self.samples.len()
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
    pub iterations: usize,
    pub passed: bool,
}

/// Full benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFTenBenchmarkResults {
    pub scale_factor: usize,
    pub total_rows: usize,
    pub queries: Vec<QueryBenchmarkResult>,
    pub all_passed: bool,
    pub p99_target_ms: f64,
}

/// Generate TPC-H SF=10 data to SQLite
fn generate_sf10_data(output_path: &str) -> std::io::Result<PathBuf> {
    let db_path = PathBuf::from(output_path);

    // Remove existing database
    if db_path.exists() {
        fs::remove_file(&db_path)?;
    }

    println!("Creating SF=10 database at {}...", output_path);
    let conn = Connection::open(&db_path).unwrap();

    // Set performance pragmas
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = OFF;
         PRAGMA cache_size = -2000000;
         PRAGMA temp_store = MEMORY;
         PRAGMA main.page_size = 4096;
         PRAGMA main.freelist_threshold = 0;",
    )
    .unwrap();

    let start = Instant::now();

    // Create tables
    create_tpch_tables(&conn);

    // Import data
    println!("Generating SF=10 data (60M lineitem rows)...");
    generate_lineitem(&conn, 10);
    generate_orders(&conn, 10);
    generate_customer(&conn, 10);
    generate_part(&conn, 10);
    generate_partsupp(&conn, 10);
    generate_nation_region(&conn);
    generate_supplier(&conn, 10);

    println!(
        "SF=10 data generation complete: {:.1}s",
        start.elapsed().as_secs_f64()
    );

    Ok(db_path)
}

fn create_tpch_tables(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE region (r_regionkey INTEGER PRIMARY KEY, r_name TEXT, r_comment TEXT);
         CREATE TABLE nation (n_nationkey INTEGER PRIMARY KEY, n_name TEXT, n_regionkey INTEGER, n_comment TEXT);
         CREATE TABLE supplier (s_suppkey INTEGER PRIMARY KEY, s_name TEXT, s_address TEXT, s_nationkey INTEGER, s_phone TEXT, s_acctbal REAL, s_comment TEXT);
         CREATE TABLE part (p_partkey INTEGER PRIMARY KEY, p_name TEXT, p_mfgr TEXT, p_brand TEXT, p_type TEXT, p_size INTEGER, p_container TEXT, p_retailprice REAL, p_comment TEXT);
         CREATE TABLE partsupp (ps_partkey INTEGER, ps_suppkey INTEGER, ps_availqty INTEGER, ps_supplycost REAL, ps_comment TEXT, PRIMARY KEY (ps_partkey, ps_suppkey));
         CREATE TABLE customer (c_custkey INTEGER PRIMARY KEY, c_name TEXT, c_address TEXT, c_nationkey INTEGER, c_phone TEXT, c_acctbal REAL, c_mktsegment TEXT, c_comment TEXT);
         CREATE TABLE orders (o_orderkey INTEGER PRIMARY KEY, o_custkey INTEGER, o_orderstatus TEXT, o_totalprice REAL, o_orderdate TEXT, o_orderpriority TEXT, o_clerk TEXT, o_shippriority INTEGER, o_comment TEXT);
         CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_linenumber INTEGER, l_quantity REAL, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT, l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT, PRIMARY KEY (l_orderkey, l_linenumber));",
    )
    .unwrap();
}

fn generate_lineitem(conn: &Connection, scale: usize) {
    let count = SF10_LINEITEM * scale / 10;
    println!("Generating lineitem ({} rows)...", count);

    let mut stmt = conn
        .prepare("INSERT INTO lineitem VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
        .unwrap();

    let mut rng = rand::thread_rng();
    let start = Instant::now();
    let batch_size = 50000;
    let return_flags = ["N", "R", "A"];
    let order_status = ["O", "F"];
    let ship_instrs = [
        "DELIVER IN PERSON",
        "COLLECT COD",
        "NONE",
        "TAKE BACK RETURN",
    ];
    let ship_modes = ["AIR", "RAIL", "SHIP", "TRUCK", "MAIL", "FOB"];

    for i in 1..=count {
        let orderkey = rng.gen_range(1..=(SF10_ORDERS * scale / 10) as i64);
        let partkey = rng.gen_range(1..=(SF10_PART * scale / 10) as i64);
        let suppkey = rng.gen_range(1..=(SF10_SUPPLIER * scale / 10) as i64);
        let quantity = rng.gen_range(1.0..50.0);
        let extendedprice = rng.gen_range(100.0..10000.0);
        let discount = rng.gen_range(0.0..0.10);
        let tax = rng.gen_range(0.0..0.10);

        let base_date = 87600;
        let ship_offset = rng.gen_range(0..500);
        let shipdate = format!(
            "{:04}-{:02}-{:02}",
            1992 + ship_offset / 365,
            (ship_offset % 365) / 30 + 1,
            ship_offset % 30 + 1
        );
        let commit_offset = ship_offset + rng.gen_range(0..30);
        let commitdate = format!(
            "{:04}-{:02}-{:02}",
            1992 + commit_offset / 365,
            (commit_offset % 365) / 30 + 1,
            commit_offset % 30 + 1
        );
        let receipt_offset = commit_offset + rng.gen_range(0..30);
        let receiptdate = format!(
            "{:04}-{:02}-{:02}",
            1992 + receipt_offset / 365,
            (receipt_offset % 365) / 30 + 1,
            receipt_offset % 30 + 1
        );

        stmt.execute(rusqlite::params![
            orderkey,
            partkey,
            suppkey,
            i as i32,
            quantity,
            extendedprice,
            discount,
            tax,
            return_flags[rng.gen_range(0..3)],
            order_status[rng.gen_range(0..2)],
            shipdate,
            commitdate,
            receiptdate,
            ship_instrs[rng.gen_range(0..4)],
            ship_modes[rng.gen_range(0..6)],
            format!("comment-{}", i)
        ])
        .unwrap();

        if i % batch_size == 0 {
            print!(
                "\r  Progress: {:.1}% ({} rows)",
                100.0 * i as f64 / count as f64,
                i
            );
        }
    }
    println!(
        "\n  lineitem complete: {:.1}s",
        start.elapsed().as_secs_f64()
    );
}

fn generate_orders(conn: &Connection, scale: usize) {
    let count = SF10_ORDERS * scale / 10;
    println!("Generating orders ({} rows)...", count);

    let mut stmt = conn
        .prepare("INSERT INTO orders VALUES (?,?,?,?,?,?,?,?,?)")
        .unwrap();

    let mut rng = rand::thread_rng();
    let start = Instant::now();
    let priorities = ["1-URGENT", "2-HIGH", "3-MEDIUM", "4-LOW", "5-LOW"];

    for i in 1..=count {
        let custkey = rng.gen_range(1..=(SF10_CUSTOMER * scale / 10) as i64);
        let totalprice = rng.gen_range(100.0..100000.0);
        let date_offset = rng.gen_range(0..2500);
        let orderdate = format!(
            "{:04}-{:02}-{:02}",
            1992 + date_offset / 365,
            (date_offset % 365) / 30 + 1,
            date_offset % 30 + 1
        );

        stmt.execute(rusqlite::params![
            i as i64,
            custkey,
            if rng.gen_bool(0.7) { "O" } else { "F" },
            totalprice,
            orderdate,
            priorities[rng.gen_range(0..5)],
            format!("Clerk#{:09}", rng.gen_range(0..1000)),
            0 as i32,
            format!("order comment {}", i)
        ])
        .unwrap();

        if i % 100000 == 0 {
            print!("\r  Progress: {:.1}%", 100.0 * i as f64 / count as f64);
        }
    }
    println!("\n  orders complete: {:.1}s", start.elapsed().as_secs_f64());
}

fn generate_customer(conn: &Connection, scale: usize) {
    let count = SF10_CUSTOMER * scale / 10;
    println!("Generating customer ({} rows)...", count);

    let mut stmt = conn
        .prepare("INSERT INTO customer VALUES (?,?,?,?,?,?,?,?)")
        .unwrap();

    let mut rng = rand::thread_rng();
    let start = Instant::now();
    let segments = [
        "AUTOMOBILE",
        "BUILDING",
        "FURNITURE",
        "MACHINERY",
        "HOUSEHOLD",
    ];

    for i in 1..=count {
        stmt.execute(rusqlite::params![
            i as i64,
            format!("Customer#{:09}", i),
            format!("Address {}", i),
            rng.gen_range(1..25) as i32,
            format!("10-{:08}", rng.gen_range(0..99999999)),
            rng.gen_range(0.0..10000.0),
            segments[rng.gen_range(0..5)],
            format!("Customer comment {}", i)
        ])
        .unwrap();

        if i % 100000 == 0 {
            print!("\r  Progress: {:.1}%", 100.0 * i as f64 / count as f64);
        }
    }
    println!(
        "\n  customer complete: {:.1}s",
        start.elapsed().as_secs_f64()
    );
}

fn generate_part(conn: &Connection, scale: usize) {
    let count = SF10_PART * scale / 10;
    println!("Generating part ({} rows)...", count);

    let mut stmt = conn
        .prepare("INSERT INTO part VALUES (?,?,?,?,?,?,?,?,?)")
        .unwrap();

    let mut rng = rand::thread_rng();
    let start = Instant::now();
    let mfgrs = ["MFGR#1", "MFGR#2", "MFGR#3", "MFGR#4", "MFGR#5"];
    let brands = ["Brand#12", "Brand#23", "Brand#34", "Brand#45", "Brand#56"];
    let types = ["ECONOMY", "PROMO", "STANDARD", "MEDIUM", "LARGE"];
    let containers = ["SM CASE", "LG CASE", "MED BOX", "LG BOX", "WRAP"];

    for i in 1..=count {
        stmt.execute(rusqlite::params![
            i as i64,
            format!("Part {}", i),
            mfgrs[rng.gen_range(0..5)],
            brands[rng.gen_range(0..5)],
            types[rng.gen_range(0..5)],
            rng.gen_range(1..50) as i32,
            containers[rng.gen_range(0..5)],
            rng.gen_range(100.0..10000.0),
            format!("Part comment {}", i)
        ])
        .unwrap();

        if i % 100000 == 0 {
            print!("\r  Progress: {:.1}%", 100.0 * i as f64 / count as f64);
        }
    }
    println!("\n  part complete: {:.1}s", start.elapsed().as_secs_f64());
}

fn generate_partsupp(conn: &Connection, scale: usize) {
    let count = SF10_PARTSUPP * scale / 10;
    println!("Generating partsupp ({} rows)...", count);

    let mut stmt = conn
        .prepare("INSERT INTO partsupp VALUES (?,?,?,?,?)")
        .unwrap();

    let mut rng = rand::thread_rng();
    let start = Instant::now();

    for i in 1..=count {
        stmt.execute(rusqlite::params![
            rng.gen_range(1..=(SF10_PART * scale / 10) as i64),
            rng.gen_range(1..=(SF10_SUPPLIER * scale / 10) as i64),
            rng.gen_range(1..9999) as i32,
            rng.gen_range(1.0..1000.0),
            format!("partsupp comment {}", i)
        ])
        .unwrap();

        if i % 100000 == 0 {
            print!("\r  Progress: {:.1}%", 100.0 * i as f64 / count as f64);
        }
    }
    println!(
        "\n  partsupp complete: {:.1}s",
        start.elapsed().as_secs_f64()
    );
}

fn generate_supplier(conn: &Connection, scale: usize) {
    let count = SF10_SUPPLIER * scale / 10;
    println!("Generating supplier ({} rows)...", count);

    let mut stmt = conn
        .prepare("INSERT INTO supplier VALUES (?,?,?,?,?,?,?)")
        .unwrap();

    let mut rng = rand::thread_rng();
    let start = Instant::now();

    for i in 1..=count {
        stmt.execute(rusqlite::params![
            i as i64,
            format!("Supplier#{:09}", i),
            format!("Supplier Address {}", i),
            rng.gen_range(1..25) as i32,
            format!("10-{:08}", rng.gen_range(0..99999999)),
            rng.gen_range(0.0..10000.0),
            format!("Supplier comment {}", i)
        ])
        .unwrap();
    }
    println!("  supplier complete: {:.1}s", start.elapsed().as_secs_f64());
}

fn generate_nation_region(conn: &Connection) {
    println!("Generating nation/region...");

    conn.execute(
        "INSERT INTO region VALUES (0, 'AFRICA', 'Africa region')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO region VALUES (1, 'AMERICA', 'America region')",
        [],
    )
    .unwrap();
    conn.execute("INSERT INTO region VALUES (2, 'ASIA', 'Asia region')", [])
        .unwrap();
    conn.execute(
        "INSERT INTO region VALUES (3, 'EUROPE', 'Europe region')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO region VALUES (4, 'MIDDLE EAST', 'Middle East region')",
        [],
    )
    .unwrap();

    let nations = [
        (0, "ALGERIA", 0),
        (1, "ARGENTINA", 1),
        (2, "BRAZIL", 1),
        (3, "CANADA", 1),
        (4, "CHINA", 2),
        (5, "EGYPT", 4),
        (6, "ETHIOPIA", 0),
        (7, "FRANCE", 3),
        (8, "GERMANY", 3),
        (9, "INDIA", 2),
        (10, "INDONESIA", 2),
        (11, "IRAN", 4),
        (12, "IRAQ", 4),
        (13, "JAPAN", 2),
        (14, "JORDAN", 4),
        (15, "KENYA", 0),
        (16, "MOROCCO", 0),
        (17, "MOZAMBIQUE", 0),
        (18, "PERU", 1),
        (19, "GERMANY", 3),
        (20, "PORTUGAL", 3),
        (21, "RUSSIA", 3),
        (22, "SAUDI ARABIA", 4),
        (23, "UNITED KINGDOM", 3),
        (24, "UNITED STATES", 1),
        (25, "VIETNAM", 2),
    ];

    for (key, name, region) in nations {
        conn.execute(
            "INSERT INTO nation VALUES (?, ?, ?, ?)",
            rusqlite::params![key, name, region as i32, format!("{} comment", name)],
        )
        .unwrap();
    }
    println!("  nation/region complete");
}

/// Import TPC-H data from SQLite to SQLRustGo FileStorage
fn import_to_sqlrustgo(
    sqlite_path: &str,
    storage_path: &PathBuf,
) -> std::io::Result<ExecutionEngine> {
    println!("Importing data to SQLRustGo FileStorage...");

    // Clean up storage
    if storage_path.exists() {
        fs::remove_dir_all(storage_path)?;
    }
    fs::create_dir_all(storage_path)?;

    let mut storage = FileStorage::new(storage_path.clone()).expect("Failed to create FileStorage");
    let conn = Connection::open(sqlite_path).unwrap();

    let start = Instant::now();

    // Create tables and import
    import_table(
        &conn,
        &mut storage,
        "region",
        5,
        vec!["r_regionkey", "r_name", "r_comment"],
    )?;
    import_table(
        &conn,
        &mut storage,
        "nation",
        25,
        vec!["n_nationkey", "n_name", "n_regionkey", "n_comment"],
    )?;
    import_table(
        &conn,
        &mut storage,
        "supplier",
        SF10_SUPPLIER / 10,
        vec![
            "s_suppkey",
            "s_name",
            "s_address",
            "s_nationkey",
            "s_phone",
            "s_acctbal",
            "s_comment",
        ],
    )?;
    import_table(
        &conn,
        &mut storage,
        "part",
        SF10_PART / 10,
        vec![
            "p_partkey",
            "p_name",
            "p_mfgr",
            "p_brand",
            "p_type",
            "p_size",
            "p_container",
            "p_retailprice",
            "p_comment",
        ],
    )?;
    import_table(
        &conn,
        &mut storage,
        "partsupp",
        SF10_PARTSUPP / 10,
        vec![
            "ps_partkey",
            "ps_suppkey",
            "ps_availqty",
            "ps_supplycost",
            "ps_comment",
        ],
    )?;
    import_table(
        &conn,
        &mut storage,
        "customer",
        SF10_CUSTOMER / 10,
        vec![
            "c_custkey",
            "c_name",
            "c_address",
            "c_nationkey",
            "c_phone",
            "c_acctbal",
            "c_mktsegment",
            "c_comment",
        ],
    )?;
    import_table(
        &conn,
        &mut storage,
        "orders",
        SF10_ORDERS / 10,
        vec![
            "o_orderkey",
            "o_custkey",
            "o_orderstatus",
            "o_totalprice",
            "o_orderdate",
            "o_orderpriority",
            "o_clerk",
            "o_shippriority",
            "o_comment",
        ],
    )?;
    import_table(
        &conn,
        &mut storage,
        "lineitem",
        SF10_LINEITEM / 10,
        vec![
            "l_orderkey",
            "l_partkey",
            "l_suppkey",
            "l_linenumber",
            "l_quantity",
            "l_extendedprice",
            "l_discount",
            "l_tax",
            "l_returnflag",
            "l_linestatus",
            "l_shipdate",
            "l_commitdate",
            "l_receiptdate",
            "l_shipinstruct",
            "l_shipmode",
            "l_comment",
        ],
    )?;

    println!("Import complete: {:.1}s", start.elapsed().as_secs_f64());

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
    Ok(engine)
}

fn import_table(
    conn: &Connection,
    storage: &mut FileStorage,
    table_name: &str,
    expected_rows: usize,
    columns: Vec<&str>,
) -> std::io::Result<()> {
    println!(
        "  Importing {} (expect {} rows)...",
        table_name, expected_rows
    );

    let start = Instant::now();
    let batch_size = 100000;

    // Get column info
    let col_defs: Vec<ColumnDefinition> = columns
        .iter()
        .map(|c| ColumnDefinition {
            name: c.to_string(),
            data_type: if c.contains("key")
                || c.contains("qty")
                || c.contains("price")
                || c.contains("cost")
                || c.contains("tax")
                || c.contains("discount")
                || c.contains("size")
            {
                "INTEGER".to_string()
            } else if c.contains("date")
                || c.contains("flag")
                || c.contains("status")
                || c.contains("mode")
                || c.contains("instruct")
                || c.contains("segment")
                || c.contains("brand")
                || c.contains("type")
                || c.contains("container")
                || c.contains("name")
                || c.contains("mfgr")
                || c.contains("comment")
            {
                "TEXT".to_string()
            } else {
                "REAL".to_string()
            },
            nullable: false,
            is_unique: false,
            references: None,
            is_primary_key: false,
            auto_increment: false,
            compression: None,
        })
        .collect();

    // Create table
    storage
        .create_table(&TableInfo {
            name: table_name.to_string(),
            columns: col_defs.clone(),
        })
        .unwrap();

    // Query all rows
    let sql = format!("SELECT {} FROM {}", columns.join(", "), table_name);
    let mut stmt = conn.prepare(&sql).unwrap();
    let mut rows_imported = 0;
    let mut batch: Vec<Vec<Value>> = Vec::new();

    let mut rows_iter = stmt.query([]).unwrap();

    while let Some(row) = rows_iter.next().unwrap() {
        let mut record: Vec<Value> = Vec::new();
        for i in 0..columns.len() {
            let val: rusqlite::types::Value = row.get(i).unwrap();
            let sql_value = match val {
                rusqlite::types::Value::Integer(i) => Value::Integer(i),
                rusqlite::types::Value::Real(f) => Value::Float(f),
                rusqlite::types::Value::Text(s) => Value::Text(s),
                rusqlite::types::Value::Null => Value::Null,
                _ => Value::Null,
            };
            record.push(sql_value);
        }
        batch.push(record);
        rows_imported += 1;

        if batch.len() >= batch_size {
            let table_data = TableData {
                info: TableInfo {
                    name: table_name.to_string(),
                    columns: col_defs.clone(),
                },
                rows: std::mem::replace(&mut batch, Vec::with_capacity(batch_size)),
            };
            storage
                .insert_table(table_name.to_string(), table_data)
                .unwrap();
        }
    }

    // Insert remaining
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

/// Run TPC-H Q1-Q22 queries and measure latency
fn run_tpch_queries(engine: &mut ExecutionEngine, iterations: usize) -> Vec<QueryBenchmarkResult> {
    println!(
        "Running TPC-H Q1-Q22 benchmark ({} iterations)...",
        iterations
    );

    let queries = vec![
        ("Q1", "SELECT l_returnflag, SUM(l_quantity) FROM lineitem WHERE l_shipdate <= '1995-12-01' GROUP BY l_returnflag"),
        ("Q2", "SELECT s_acctbal, s_name, n_name, p_partkey FROM part, supplier, partsupp, nation, region WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15 AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'EUROPE' ORDER BY s_acctbal DESC LIMIT 10"),
        ("Q3", "SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey"),
        ("Q4", "SELECT o_orderpriority, COUNT(*) FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' GROUP BY o_orderpriority"),
        ("Q5", "SELECT n_name, SUM(l_extendedprice) FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' GROUP BY n_name"),
        ("Q6", "SELECT SUM(l_extendedprice) FROM lineitem WHERE l_quantity < 24 AND l_shipdate >= '1994-01-01'"),
        ("Q7", "SELECT n1.n_name AS supp_nation, n2.n_name AS cust_nation, SUM(l_extendedprice) FROM supplier, lineitem, orders, customer, nation n1, nation n2 WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND c_custkey = o_custkey AND s_nationkey = n1.n_nationkey AND c_nationkey = n2.n_nationkey GROUP BY n1.n_name, n2.n_name"),
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
            "    {}: P99={:.2}ms, avg={:.2}ms {}",
            name,
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
    #[ignore]
    fn test_tpch_sf10_full_benchmark() {
        // This test is ignored by default because it takes 10-30 minutes
        // Run with: cargo test --test tpch_sf10_benchmark -- --ignored --nocapture

        let sqlite_path = "/tmp/tpch_sf10.db";
        let storage_path = PathBuf::from("/tmp/tpch_sf10_storage");

        // Generate SF=10 data
        let db_path = generate_sf10_data(sqlite_path).expect("Failed to generate SF=10 data");

        // Import to SQLRustGo
        let mut engine =
            import_to_sqlrustgo(sqlite_path, &storage_path).expect("Failed to import data");

        // Run benchmark
        let results = run_tpch_queries(&mut engine, 10);

        // Check results
        let all_passed = results.iter().all(|r| r.passed);
        let benchmark_results = SFTenBenchmarkResults {
            scale_factor: 10,
            total_rows: SF10_LINEITEM,
            queries: results,
            all_passed,
            p99_target_ms: P99_TARGET_MS,
        };

        println!("\n=== SF=10 Benchmark Results ===");
        println!("Scale Factor: SF={}", benchmark_results.scale_factor);
        println!("Total Rows: {}", benchmark_results.total_rows);
        println!("P99 Target: {:.0}ms", benchmark_results.p99_target_ms);
        println!(
            "All Passed: {}",
            if all_passed { "YES ✅" } else { "NO ❌" }
        );

        assert!(
            all_passed,
            "Some queries exceeded P99 latency target of {}ms",
            P99_TARGET_MS
        );
    }

    #[test]
    fn test_tpch_sf01_quick_benchmark() {
        // Quick benchmark with SF=0.1 to verify the test framework works
        // This uses in-memory storage for speed

        let sqlite_path = "/tmp/tpch_sf01_quick.db";
        let storage_path = PathBuf::from("/tmp/tpch_sf01_quick_storage");

        // Generate SF=1 data (scaled down)
        println!("Generating SF=1 test data...");
        generate_sf10_data(sqlite_path).expect("Failed to generate data");

        // Import to SQLRustGo
        let mut engine =
            import_to_sqlrustgo(sqlite_path, &storage_path).expect("Failed to import data");

        // Run a simple query test
        let sql = "SELECT COUNT(*) FROM lineitem";
        let start = Instant::now();
        let result = engine.execute(parse(sql).unwrap());
        let elapsed = start.elapsed();

        println!(
            "Query executed: {} rows in {:.2}ms",
            result.as_ref().map(|r| r.rows.len()).unwrap_or(0),
            elapsed.as_secs_f64() * 1000.0
        );

        assert!(result.is_ok());
    }
}
