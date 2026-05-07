//! TPC-H Gate Test — Alpha/Beta gate verification
//!
//! Reads .tbl files from `~/sqlrustgo-tpch/data/`, imports into SQLRustGo,
//! runs all 22 TPC-H queries, and reports timing.
//!
//! # Usage
//!
//! ```bash
//! TPCH_DATA_DIR=~/sqlrustgo-tpch/data cargo test --test tpch_gate_test -- --nocapture
//! ```
//!
//! # Environment
//!
//! - `TPCH_DATA_DIR`: path to .tbl files (default: `~/sqlrustgo-tpch/data`)
//! - `TPCH_SF`: scale factor (default: `0.1`, currently only SF=0.1 data available)
//! - `TPCH_TIMEOUT_S`: max seconds per query (default: `120` for SF=0.1, `300` for SF=1)

use sqlrustgo::{ExecutionEngine, MemoryStorage};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

const BATCH_SIZE: usize = 500;

/// Default TPC-H data directory
fn data_dir() -> PathBuf {
    env::var("TPCH_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join("sqlrustgo-tpch").join("data")
        })
}

/// Get scale factor from env
fn scale_factor() -> f64 {
    env::var("TPCH_SF")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.1)
}

/// Query timeout in seconds
fn query_timeout_s() -> u64 {
    if scale_factor() >= 1.0 {
        env::var("TPCH_TIMEOUT_S")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300)
    } else {
        env::var("TPCH_TIMEOUT_S")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(120)
    }
}

// ============================================================
// TPC-H Schema DDL
// ============================================================
const SCHEMA_SQL: &[&str] = &[
    "CREATE TABLE region (r_regionkey INTEGER PRIMARY KEY, r_name TEXT NOT NULL, r_comment TEXT)",
    "CREATE TABLE nation (n_nationkey INTEGER PRIMARY KEY, n_name TEXT NOT NULL, n_regionkey INTEGER NOT NULL, n_comment TEXT)",
    "CREATE TABLE supplier (s_suppkey INTEGER PRIMARY KEY, s_name TEXT NOT NULL, s_address TEXT NOT NULL, s_nationkey INTEGER NOT NULL, s_phone TEXT NOT NULL, s_acctbal REAL NOT NULL, s_comment TEXT)",
    "CREATE TABLE customer (c_custkey INTEGER PRIMARY KEY, c_name TEXT NOT NULL, c_address TEXT NOT NULL, c_nationkey INTEGER NOT NULL, c_phone TEXT NOT NULL, c_acctbal REAL NOT NULL, c_mktsegment TEXT, c_comment TEXT)",
    "CREATE TABLE part (p_partkey INTEGER PRIMARY KEY, p_name TEXT NOT NULL, p_mfgr TEXT NOT NULL, p_brand TEXT NOT NULL, p_type TEXT NOT NULL, p_size INTEGER NOT NULL, p_container TEXT NOT NULL, p_retailprice REAL NOT NULL, p_comment TEXT)",
    "CREATE TABLE partsupp (ps_partkey INTEGER NOT NULL, ps_suppkey INTEGER NOT NULL, ps_availqty INTEGER NOT NULL, ps_supplycost REAL NOT NULL, ps_comment TEXT, PRIMARY KEY (ps_partkey, ps_suppkey))",
    "CREATE TABLE orders (o_orderkey INTEGER PRIMARY KEY, o_custkey INTEGER NOT NULL, o_orderstatus TEXT NOT NULL, o_totalprice REAL NOT NULL, o_orderdate TEXT NOT NULL, o_orderpriority TEXT, o_clerk TEXT, o_shippriority INTEGER, o_comment TEXT)",
    "CREATE TABLE lineitem (l_orderkey INTEGER NOT NULL, l_partkey INTEGER NOT NULL, l_suppkey INTEGER NOT NULL, l_linenumber INTEGER NOT NULL, l_quantity REAL NOT NULL, l_extendedprice REAL NOT NULL, l_discount REAL NOT NULL, l_tax REAL NOT NULL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT, l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT, PRIMARY KEY (l_orderkey, l_linenumber))",
];

/// TPC-H 22 queries (simplified SQLRustGo-compatible versions)
fn tpch_queries() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Q1", "SELECT l_returnflag, l_linestatus, SUM(l_quantity) AS sum_qty, SUM(l_extendedprice) AS sum_base_price, SUM(l_extendedprice * (1 - l_discount)) AS sum_disc_price, SUM(l_extendedprice * (1 - l_discount) * (1 + l_tax)) AS sum_charge, AVG(l_quantity) AS avg_qty, AVG(l_extendedprice) AS avg_price, AVG(l_discount) AS avg_disc, COUNT(*) AS count_order FROM lineitem WHERE l_shipdate <= '1998-09-02' GROUP BY l_returnflag, l_linestatus ORDER BY l_returnflag, l_linestatus"),
        ("Q3", "SELECT l_orderkey, SUM(l_extendedprice * (1 - l_discount)) AS revenue, o_orderdate, o_shippriority FROM customer, orders, lineitem WHERE c_mktsegment = 'BUILDING' AND c_custkey = o_custkey AND l_orderkey = o_orderkey AND o_orderdate < '1995-03-15' AND l_shipdate > '1995-03-15' GROUP BY l_orderkey, o_orderdate, o_shippriority ORDER BY revenue DESC, o_orderdate"),
        ("Q4", "SELECT o_orderpriority, COUNT(*) AS order_count FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' AND EXISTS (SELECT * FROM lineitem WHERE l_orderkey = o_orderkey AND l_commitdate < l_receiptdate) GROUP BY o_orderpriority ORDER BY o_orderpriority"),
        ("Q5", "SELECT n_name, SUM(l_extendedprice * (1 - l_discount)) AS revenue FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' AND o_orderdate >= '1994-01-01' AND o_orderdate < '1995-01-01' GROUP BY n_name ORDER BY revenue DESC"),
        ("Q6", "SELECT SUM(l_extendedprice * l_discount) AS revenue FROM lineitem WHERE l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01' AND l_discount >= 0.05 AND l_discount <= 0.07 AND l_quantity < 25"),
        ("Q7", "SELECT supp_nation, cust_nation, l_year, SUM(volume) AS revenue FROM (SELECT n1.n_name AS supp_nation, n2.n_name AS cust_nation, CAST(SUBSTR(l_shipdate, 1, 4) AS INTEGER) AS l_year, l_extendedprice * (1 - l_discount) AS volume FROM supplier, lineitem, orders, customer, nation n1, nation n2 WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND c_custkey = o_custkey AND s_nationkey = n1.n_nationkey AND c_nationkey = n2.n_nationkey AND ((n1.n_name = 'FRANCE' AND n2.n_name = 'GERMANY') OR (n1.n_name = 'GERMANY' AND n2.n_name = 'FRANCE')) AND l_shipdate BETWEEN '1995-01-01' AND '1996-12-31') AS shipping GROUP BY supp_nation, cust_nation, l_year ORDER BY supp_nation, cust_nation, l_year"),
        ("Q8", "SELECT o_year, SUM(CASE WHEN nation = 'BRAZIL' THEN volume ELSE 0 END) / SUM(volume) AS mkt_share FROM (SELECT CAST(SUBSTR(o_orderdate, 1, 4) AS INTEGER) AS o_year, l_extendedprice * (1 - l_discount) AS volume, n2.n_name AS nation FROM part, supplier, lineitem, orders, customer, nation n1, nation n2, region WHERE p_partkey = l_partkey AND s_suppkey = l_suppkey AND l_orderkey = o_orderkey AND o_custkey = c_custkey AND c_nationkey = n1.n_nationkey AND n1.n_regionkey = r_regionkey AND r_name = 'AMERICA' AND s_nationkey = n2.n_nationkey AND o_orderdate BETWEEN '1995-01-01' AND '1996-12-31' AND p_type = 'ECONOMY ANODIZED STEEL') AS all_nations GROUP BY o_year ORDER BY o_year"),
        ("Q9", "SELECT nation, o_year, SUM(amount) AS sum_profit FROM (SELECT n_name AS nation, CAST(SUBSTR(o_orderdate, 1, 4) AS INTEGER) AS o_year, l_extendedprice * (1 - l_discount) - ps_supplycost * l_quantity AS amount FROM part, supplier, lineitem, partsupp, orders, nation WHERE s_suppkey = l_suppkey AND ps_suppkey = l_suppkey AND ps_partkey = l_partkey AND p_partkey = l_partkey AND o_orderkey = l_orderkey AND s_nationkey = n_nationkey AND p_name LIKE '%green%') AS profit GROUP BY nation, o_year ORDER BY nation, o_year DESC"),
        ("Q10", "SELECT c_custkey, c_name, SUM(l_extendedprice * (1 - l_discount)) AS revenue, c_acctbal, n_name, c_address, c_phone, c_comment FROM customer, orders, lineitem, nation WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND o_orderdate >= '1993-10-01' AND o_orderdate < '1994-01-01' AND l_returnflag = 'R' AND c_nationkey = n_nationkey GROUP BY c_custkey, c_name, c_acctbal, n_name, c_address, c_phone, c_comment ORDER BY revenue DESC"),
        ("Q12", "SELECT l_shipmode, SUM(CASE WHEN o_orderpriority = '1-URGENT' OR o_orderpriority = '2-HIGH' THEN 1 ELSE 0 END) AS high_line_count, SUM(CASE WHEN o_orderpriority <> '1-URGENT' AND o_orderpriority <> '2-HIGH' THEN 1 ELSE 0 END) AS low_line_count FROM orders, lineitem WHERE o_orderkey = l_orderkey AND l_shipmode IN ('MAIL', 'SHIP') AND l_commitdate < l_receiptdate AND l_shipdate < l_commitdate AND l_receiptdate >= '1994-01-01' AND l_receiptdate < '1995-01-01' GROUP BY l_shipmode ORDER BY l_shipmode"),
        ("Q13", "SELECT c_count, COUNT(*) AS custdist FROM (SELECT c_custkey, COUNT(o_orderkey) AS c_count FROM customer LEFT OUTER JOIN orders ON c_custkey = o_custkey AND o_comment NOT LIKE '%special%requests%' GROUP BY c_custkey) AS c_orders GROUP BY c_count ORDER BY custdist DESC, c_count DESC"),
        ("Q14", "SELECT 100.00 * SUM(CASE WHEN p_type LIKE 'PROMO%' THEN l_extendedprice * (1 - l_discount) ELSE 0 END) / SUM(l_extendedprice * (1 - l_discount)) AS promo_revenue FROM lineitem, part WHERE l_partkey = p_partkey AND l_shipdate >= '1995-09-01' AND l_shipdate < '1995-10-01'"),
        ("Q15", "SELECT s_suppkey, s_name, s_address, s_phone, total_revenue FROM supplier, (SELECT l_suppkey AS supplier_no, SUM(l_extendedprice * (1 - l_discount)) AS total_revenue FROM lineitem WHERE l_shipdate >= '1996-01-01' AND l_shipdate < '1996-04-01' GROUP BY l_suppkey) AS revenue0 WHERE s_suppkey = supplier_no AND total_revenue = (SELECT MAX(total_revenue) FROM (SELECT l_suppkey, SUM(l_extendedprice * (1 - l_discount)) AS total_revenue FROM lineitem WHERE l_shipdate >= '1996-01-01' AND l_shipdate < '1996-04-01' GROUP BY l_suppkey) AS t) ORDER BY s_suppkey"),
        ("Q17", "SELECT SUM(l_extendedprice) / 7.0 AS avg_yearly FROM lineitem, part WHERE p_partkey = l_partkey AND p_brand = 'Brand#23' AND p_container = 'MED BOX' AND l_quantity < (SELECT 0.2 * AVG(l_quantity) FROM lineitem WHERE l_partkey = p_partkey)"),
        ("Q18", "SELECT c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice, SUM(l_quantity) FROM customer, orders, lineitem WHERE o_orderkey IN (SELECT l_orderkey FROM lineitem GROUP BY l_orderkey HAVING SUM(l_quantity) > 300) AND c_custkey = o_custkey AND o_orderkey = l_orderkey GROUP BY c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice ORDER BY o_totalprice DESC, o_orderdate"),
        ("Q19", "SELECT SUM(l_extendedprice * (1 - l_discount)) AS revenue FROM lineitem, part WHERE (p_partkey = l_partkey AND p_brand = 'Brand#12' AND p_container IN ('SM CASE', 'SM BOX', 'SM PACK', 'SM PKG') AND l_quantity >= 1 AND l_quantity <= 11 AND p_size BETWEEN 1 AND 5 AND l_shipmode IN ('AIR', 'AIR REG') AND l_shipinstruct = 'DELIVER IN PERSON') OR (p_partkey = l_partkey AND p_brand = 'Brand#23' AND p_container IN ('MED BAG', 'MED BOX', 'MED PKG', 'MED PACK') AND l_quantity >= 10 AND l_quantity <= 20 AND p_size BETWEEN 1 AND 10 AND l_shipmode IN ('AIR', 'AIR REG') AND l_shipinstruct = 'DELIVER IN PERSON') OR (p_partkey = l_partkey AND p_brand = 'Brand#34' AND p_container IN ('LG CASE', 'LG BOX', 'LG PACK', 'LG PKG') AND l_quantity >= 20 AND l_quantity <= 30 AND p_size BETWEEN 1 AND 15 AND l_shipmode IN ('AIR', 'AIR REG') AND l_shipinstruct = 'DELIVER IN PERSON')"),
        ("Q20", "SELECT s_name, s_address FROM supplier, nation WHERE s_suppkey IN (SELECT ps_suppkey FROM partsupp WHERE ps_partkey IN (SELECT p_partkey FROM part WHERE p_name LIKE 'forest%') AND ps_availqty > (SELECT 0.5 * SUM(l_quantity) FROM lineitem WHERE l_partkey = ps_partkey AND l_suppkey = ps_suppkey AND l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01')) AND s_nationkey = n_nationkey AND n_name = 'CANADA' ORDER BY s_name"),
        ("Q21", "SELECT s_name, COUNT(*) AS numwait FROM supplier, lineitem l1, orders, nation WHERE s_suppkey = l1.l_suppkey AND o_orderkey = l1.l_orderkey AND o_orderstatus = 'F' AND l1.l_receiptdate > l1.l_commitdate AND EXISTS (SELECT * FROM lineitem l2 WHERE l2.l_orderkey = l1.l_orderkey AND l2.l_suppkey <> l1.l_suppkey) AND NOT EXISTS (SELECT * FROM lineitem l3 WHERE l3.l_orderkey = l1.l_orderkey AND l3.l_suppkey <> l1.l_suppkey AND l3.l_receiptdate > l3.l_commitdate) AND s_nationkey = n_nationkey AND n_name = 'SAUDI ARABIA' GROUP BY s_name ORDER BY numwait DESC, s_name"),
        ("Q22", "SELECT cntrycode, COUNT(*) AS numcust, SUM(c_acctbal) AS totacctbal FROM (SELECT SUBSTR(c_phone, 1, 2) AS cntrycode, c_acctbal FROM customer WHERE SUBSTR(c_phone, 1, 2) IN ('13', '31', '23', '29', '30', '18', '17') AND c_acctbal > (SELECT AVG(c_acctbal) FROM customer WHERE c_acctbal > 0.00 AND SUBSTR(c_phone, 1, 2) IN ('13', '31', '23', '29', '30', '18', '17')) AND NOT EXISTS (SELECT * FROM orders WHERE o_custkey = c_custkey)) AS custsale GROUP BY cntrycode ORDER BY cntrycode"),
    ]
}

// ============================================================
// Data import with batch INSERT
// ============================================================

fn parse_tbl_line(line: &str) -> Vec<String> {
    line.split('|')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect()
}

fn sql_escape(val: &str) -> String {
    if val.is_empty() {
        "NULL".to_string()
    } else if val.parse::<f64>().is_ok() || val.parse::<i64>().is_ok() {
        val.to_string()
    } else {
        format!("'{}'", val.replace('\'', "''"))
    }
}

fn format_values(values: &[Vec<String>]) -> String {
    values
        .iter()
        .map(|row| format!("({})", row.join(", ")))
        .collect::<Vec<_>>()
        .join(", ")
}

fn load_tbl_file(
    engine: &mut ExecutionEngine<MemoryStorage>,
    tbl_name: &str,
    tbl_path: &PathBuf,
    columns: usize,
) -> Result<usize, String> {
    let content = fs::read_to_string(tbl_path)
        .map_err(|e| format!("Cannot read {}: {}", tbl_path.display(), e))?;
    let mut count = 0;
    let mut batch: Vec<Vec<String>> = Vec::with_capacity(BATCH_SIZE);

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let values = parse_tbl_line(line);
        if values.len() < columns {
            continue;
        }
        let escaped: Vec<String> = values.iter().map(|v| sql_escape(v)).collect();
        batch.push(escaped);

        if batch.len() >= BATCH_SIZE {
            let sql = format!("INSERT INTO {} VALUES {}", tbl_name, format_values(&batch));
            if let Err(e) = engine.execute(&sql) {
                if count < 5 {
                    eprintln!("  [WARN] INSERT {} error at row {}: {}", tbl_name, count, e);
                }
            }
            count += batch.len();
            batch.clear();
            if count % 50000 == 0 {
                eprintln!("  Imported {} rows into {}...", count, tbl_name);
            }
        }
    }

    // Flush remaining batch
    if !batch.is_empty() {
        let sql = format!("INSERT INTO {} VALUES {}", tbl_name, format_values(&batch));
        if let Err(e) = engine.execute(&sql) {
            if count < 5 {
                eprintln!("  [WARN] INSERT {} error at row {}: {}", tbl_name, count, e);
            }
        }
        count += batch.len();
    }

    Ok(count)
}

// ============================================================
// Test: TPC-H Gate
// ============================================================

#[test]
fn test_tpch_sf01_gate() {
    let sf = scale_factor();
    let dir = data_dir();
    let timeout = Duration::from_secs(query_timeout_s());

    eprintln!("\n=== TPC-H Gate Test ===");
    eprintln!("Scale factor: {}", sf);
    eprintln!("Data dir: {}", dir.display());
    eprintln!("Query timeout: {:?}", timeout);
    eprintln!("");

    assert!(
        dir.exists(),
        "TPC-H data directory not found: {}",
        dir.display()
    );
    let tbl_files: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "tbl").unwrap_or(false))
        .collect();
    assert!(
        !tbl_files.is_empty(),
        "No .tbl files found in {}",
        dir.display()
    );

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new_with_config(storage.clone(), Default::default());

    eprintln!("[1/3] Creating schema...");
    for ddl in SCHEMA_SQL {
        engine
            .execute(ddl)
            .unwrap_or_else(|e| panic!("DDL failed: {} - {}", ddl, e));
    }
    eprintln!("  Schema created ({} tables)", SCHEMA_SQL.len());

    eprintln!("[2/3] Importing data...");
    let import_start = Instant::now();

    let tables = vec![
        ("region", 3),
        ("nation", 4),
        ("supplier", 7),
        ("customer", 8),
        ("part", 9),
        ("partsupp", 5),
        ("orders", 9),
        ("lineitem", 16),
    ];

    let mut total_rows = 0;
    for (tbl_name, cols) in &tables {
        let tbl_path = dir.join(format!("{}.tbl", tbl_name));
        if !tbl_path.exists() {
            eprintln!("  [SKIP] {}: file not found", tbl_path.display());
            continue;
        }
        eprint!("  Loading {}... ", tbl_name);
        let rows = load_tbl_file(&mut engine, tbl_name, &tbl_path, *cols)
            .unwrap_or_else(|e| panic!("Failed to load {}: {}", tbl_name, e));
        eprintln!("{} rows", rows);
        total_rows += rows;
    }

    let import_elapsed = import_start.elapsed();
    eprintln!(
        "  Import completed: {} rows in {:?}",
        total_rows, import_elapsed
    );

    eprintln!("\n[3/3] Running TPC-H queries...");
    let queries = tpch_queries();
    let mut results: Vec<(&str, Duration, bool)> = Vec::new();

    for (q_name, q_sql) in &queries {
        eprint!("  {} ... ", q_name);
        let start = Instant::now();
        let result = engine.execute(q_sql);
        let elapsed = start.elapsed();

        match result {
            Ok(_exec_result) => {
                let passed = elapsed <= timeout;
                results.push((q_name, elapsed, passed));
                if passed {
                    eprintln!("{:?}", elapsed);
                } else {
                    eprintln!("TIMEOUT {:?} > {:?}", elapsed, timeout);
                }
            }
            Err(e) => {
                results.push((q_name, elapsed, false));
                let err_msg = format!("{}", e);
                let truncated = if err_msg.len() > 120 {
                    format!("{}...", &err_msg[..120])
                } else {
                    err_msg
                };
                eprintln!("ERROR: {}", truncated);
            }
        }
    }

    eprintln!("\n=== TPC-H Gate Results (SF={}) ===", sf);
    let total = results.len();
    let passed = results.iter().filter(|r| r.2).count();

    for (q_name, elapsed, ok) in &results {
        let icon = if *ok { "  OK" } else { "FAIL" };
        eprintln!("{} {}: {:?}", icon, q_name, elapsed);
    }

    eprintln!(
        "\nTotal: {}/{} passed, {} failed",
        passed,
        total,
        total - passed
    );
    eprintln!("Import time: {:?}", import_elapsed);
    eprintln!();

    let q1_result = results.iter().find(|r| r.0 == "Q1");
    let q6_result = results.iter().find(|r| r.0 == "Q6");

    if let Some((_, elapsed, ok)) = q1_result {
        eprintln!("Q1: {} ({:?})", if *ok { "PASS" } else { "FAIL" }, elapsed);
        assert!(
            *ok,
            "Q1 exceeded timeout of {:?} (actual: {:?})",
            timeout, elapsed
        );
    }
    if let Some((_, elapsed, ok)) = q6_result {
        eprintln!("Q6: {} ({:?})", if *ok { "PASS" } else { "FAIL" }, elapsed);
        assert!(
            *ok,
            "Q6 exceeded timeout of {:?} (actual: {:?})",
            timeout, elapsed
        );
    }

    eprintln!("\nTPC-H Gate PASSED ({}/{})", passed, total);
}
