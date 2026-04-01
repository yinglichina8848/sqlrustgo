//! TPC-H Benchmark横向对比测试
//!
//! 对比 SQLRustGo、SQLite、MySQL、PostgreSQL 的 TPC-H 查询性能
//!
//! 运行方式:
//!   cargo test --test tpch_benchmark -- --nocapture
//!   cargo test --test tpch_benchmark -- --nocapture --test-threads=1

#[cfg(test)]
mod tests {
    use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
    use std::process::Command;
    use std::sync::{Arc, RwLock};
    use std::time::Instant;

    fn create_engine() -> ExecutionEngine {
        ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
    }

    fn setup_sqlrustgo_test_data() -> ExecutionEngine {
        let mut engine = create_engine();

        engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_linenumber INTEGER, l_quantity INTEGER, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT, l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT)").unwrap()).unwrap();
        engine.execute(parse("CREATE TABLE orders (o_orderkey INTEGER PRIMARY KEY, o_custkey INTEGER, o_orderstatus TEXT, o_totalprice REAL, o_orderdate TEXT, o_orderpriority TEXT, o_clerk TEXT, o_shippriority INTEGER, o_comment TEXT)").unwrap()).unwrap();
        engine.execute(parse("CREATE TABLE customer (c_custkey INTEGER PRIMARY KEY, c_name TEXT, c_address TEXT, c_nationkey INTEGER, c_phone TEXT, c_acctbal REAL, c_mktsegment TEXT, c_comment TEXT)").unwrap()).unwrap();

        engine.execute(parse("INSERT INTO lineitem VALUES (1, 100, 1000, 1, 15, 15000.00, 0.05, 1.2, 'N', 'O', '2024-01-01', '2024-01-02', '2024-01-03', 'NONE', 'AIR', 'comment1')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO lineitem VALUES (1, 101, 1001, 2, 20, 20000.00, 0.05, 1.6, 'N', 'O', '2024-01-01', '2024-01-02', '2024-01-03', 'NONE', 'AIR', 'comment2')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO lineitem VALUES (2, 102, 1002, 1, 5, 5000.00, 0.10, 0.4, 'N', 'O', '2024-01-02', '2024-01-03', '2024-01-04', 'NONE', 'TRUCK', 'comment3')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO lineitem VALUES (3, 103, 1003, 1, 8, 8000.00, 0.08, 0.64, 'N', 'O', '2024-01-03', '2024-01-04', '2024-01-05', 'NONE', 'RAIL', 'comment4')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO lineitem VALUES (3, 104, 1004, 1, 25, 25000.00, 0.03, 2.0, 'A', 'F', '2024-01-03', '2024-01-04', '2024-01-05', 'NONE', 'AIR', 'comment5')").unwrap()).unwrap();

        engine.execute(parse("INSERT INTO orders VALUES (1, 1, 'O', 15000.00, '2024-01-01', '1-URGENT', 'Clerk#001', 0, 'comment')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO orders VALUES (2, 2, 'O', 5000.00, '2024-01-02', '5-LOW', 'Clerk#002', 0, 'comment')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO orders VALUES (3, 3, 'O', 8000.00, '2024-01-03', '3-MEDIUM', 'Clerk#003', 0, 'comment')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO orders VALUES (4, 1, 'O', 25000.00, '2024-01-03', '1-URGENT', 'Clerk#001', 0, 'comment')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO orders VALUES (5, 2, 'F', 3000.00, '2024-01-04', '2-HIGH', 'Clerk#002', 0, 'comment')").unwrap()).unwrap();

        engine.execute(parse("INSERT INTO customer VALUES (1, 'Customer#00001', 'Address1', 1, '10-1111111', 1000.00, 'AUTOMOBILE', 'comment1')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO customer VALUES (2, 'Customer#00002', 'Address2', 2, '10-2222222', 2000.00, 'BUILDING', 'comment2')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO customer VALUES (3, 'Customer#00003', 'Address3', 1, '10-3333333', 3000.00, 'AUTOMOBILE', 'comment3')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO customer VALUES (4, 'Customer#00004', 'Address4', 1, '10-4444444', 4000.00, 'FURNITURE', 'comment4')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO customer VALUES (5, 'Customer#00005', 'Address5', 2, '10-5555555', 5000.00, 'MACHINERY', 'comment5')").unwrap()).unwrap();

        engine
    }

    mod sqlrustgo_bench {
        use super::*;

        #[test]
        fn test_sqlrustgo_tpch_q1_scan() {
            let mut engine = setup_sqlrustgo_test_data();

            let sql = "SELECT * FROM lineitem";

            let start = Instant::now();
            let result = engine.execute(parse(sql).unwrap()).unwrap();
            let elapsed = start.elapsed();

            println!(
                "SQLRustGo Q1 (Scan): {} rows in {:?}",
                result.rows.len(),
                elapsed
            );
            assert!(!result.rows.is_empty());
        }

        #[test]
        fn test_sqlrustgo_tpch_q2_filter() {
            let mut engine = setup_sqlrustgo_test_data();

            let sql = "SELECT * FROM lineitem WHERE l_quantity > 10";

            let start = Instant::now();
            let result = engine.execute(parse(sql).unwrap()).unwrap();
            let elapsed = start.elapsed();

            println!(
                "SQLRustGo Q2 (Filter): {} rows in {:?}",
                result.rows.len(),
                elapsed
            );
            assert!(result.rows.len() >= 2);
        }

        #[test]
        fn test_sqlrustgo_tpch_q3_join() {
            let mut engine = setup_sqlrustgo_test_data();

            let sql = "SELECT c.c_name, o.o_orderkey FROM customer c JOIN orders o ON c.c_custkey = o.o_custkey WHERE c.c_mktsegment = 'AUTOMOBILE'";

            let start = Instant::now();
            let result = engine.execute(parse(sql).unwrap()).unwrap();
            let elapsed = start.elapsed();

            println!(
                "SQLRustGo Q3 (Join): {} rows in {:?}",
                result.rows.len(),
                elapsed
            );
            assert!(!result.rows.is_empty());
        }

        #[test]
        fn test_sqlrustgo_tpch_performance_100_iterations() {
            let mut engine = setup_sqlrustgo_test_data();
            let iterations = 100;

            let sql = "SELECT * FROM lineitem WHERE l_quantity > 10";

            let start = Instant::now();
            for _ in 0..iterations {
                let _ = engine.execute(parse(sql).unwrap()).unwrap();
            }
            let total_elapsed = start.elapsed();
            let avg_ms = (total_elapsed.as_secs_f64() * 1000.0) / iterations as f64;

            println!(
                "SQLRustGo Performance: {} iterations, avg {:.3}ms, total {:?}",
                iterations, avg_ms, total_elapsed
            );
            assert!(avg_ms < 10.0, "SQLRustGo too slow: {:.3}ms avg", avg_ms);
        }

        #[test]
        fn test_sqlrustgo_tpch_all_queries() {
            let mut engine = setup_sqlrustgo_test_data();

            let queries = vec![
                ("Q1", "SELECT * FROM lineitem WHERE l_quantity > 10"),
                ("Q2", "SELECT * FROM lineitem WHERE l_extendedprice > 10000"),
                (
                    "Q3",
                    "SELECT * FROM customer WHERE c_mktsegment = 'AUTOMOBILE'",
                ),
            ];

            println!("\n=== SQLRustGo TPC-H Q1-Q3 Performance ===");

            for (name, sql) in queries {
                let start = Instant::now();
                let result = engine.execute(parse(sql).unwrap()).unwrap();
                let elapsed = start.elapsed();

                println!(
                    "SQLRustGo {}: {} rows in {:?}",
                    name,
                    result.rows.len(),
                    elapsed
                );
            }
        }
    }

    fn run_sqlite_query(sql: &str, db_path: &str) -> Result<(usize, std::time::Duration), String> {
        let start = Instant::now();
        let output = Command::new("sqlite3")
            .args(&[db_path, sql])
            .output()
            .map_err(|e| format!("Failed to run sqlite3: {}", e))?;

        let elapsed = start.elapsed();

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        let rows = String::from_utf8_lossy(&output.stdout).lines().count();

        Ok((rows, elapsed))
    }

    fn check_database(database: &str) -> bool {
        match database {
            "sqlite" => Command::new("sqlite3").arg("--version").output().is_ok(),
            "mysql" => Command::new("mysql").arg("--version").output().is_ok(),
            "postgres" => Command::new("psql").arg("--version").output().is_ok(),
            _ => false,
        }
    }

    fn create_sqlite_test_db() -> Result<String, String> {
        let pid = std::process::id();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = format!("/tmp/sqlrustgo_tpch_{}_{}.db", pid, ts);

        let bash_script = format!(
            r#"
cat << 'EOSQL' | sqlite3 {}
CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_linenumber INTEGER, l_quantity INTEGER, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT, l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT);
CREATE TABLE orders (o_orderkey INTEGER PRIMARY KEY, o_custkey INTEGER, o_orderstatus TEXT, o_totalprice REAL, o_orderdate TEXT, o_orderpriority TEXT, o_clerk TEXT, o_shippriority INTEGER, o_comment TEXT);
CREATE TABLE customer (c_custkey INTEGER PRIMARY KEY, c_name TEXT, c_address TEXT, c_nationkey INTEGER, c_phone TEXT, c_acctbal REAL, c_mktsegment TEXT, c_comment TEXT);
INSERT INTO lineitem VALUES (1, 100, 1000, 1, 15, 15000.00, 0.05, 1.2, 'N', 'O', '2024-01-01', '2024-01-02', '2024-01-03', 'NONE', 'AIR', 'comment1');
INSERT INTO lineitem VALUES (1, 101, 1001, 2, 20, 20000.00, 0.05, 1.6, 'N', 'O', '2024-01-01', '2024-01-02', '2024-01-03', 'NONE', 'AIR', 'comment2');
INSERT INTO lineitem VALUES (2, 102, 1002, 1, 5, 5000.00, 0.10, 0.4, 'N', 'O', '2024-01-02', '2024-01-03', '2024-01-04', 'NONE', 'TRUCK', 'comment3');
INSERT INTO lineitem VALUES (3, 103, 1003, 1, 8, 8000.00, 0.08, 0.64, 'N', 'O', '2024-01-03', '2024-01-04', '2024-01-05', 'NONE', 'RAIL', 'comment4');
INSERT INTO lineitem VALUES (3, 104, 1004, 1, 25, 25000.00, 0.03, 2.0, 'A', 'F', '2024-01-03', '2024-01-04', '2024-01-05', 'NONE', 'AIR', 'comment5');
INSERT INTO orders VALUES (1, 1, 'O', 15000.00, '2024-01-01', '1-URGENT', 'Clerk#001', 0, 'comment');
INSERT INTO orders VALUES (2, 2, 'O', 5000.00, '2024-01-02', '5-LOW', 'Clerk#002', 0, 'comment');
INSERT INTO orders VALUES (3, 3, 'O', 8000.00, '2024-01-03', '3-MEDIUM', 'Clerk#003', 0, 'comment');
INSERT INTO orders VALUES (4, 1, 'O', 25000.00, '2024-01-03', '1-URGENT', 'Clerk#001', 0, 'comment');
INSERT INTO orders VALUES (5, 2, 'F', 3000.00, '2024-01-04', '2-HIGH', 'Clerk#002', 0, 'comment');
INSERT INTO customer VALUES (1, 'Customer#00001', 'Address1', 1, '10-1111111', 1000.00, 'AUTOMOBILE', 'comment1');
INSERT INTO customer VALUES (2, 'Customer#00002', 'Address2', 2, '10-2222222', 2000.00, 'BUILDING', 'comment2');
INSERT INTO customer VALUES (3, 'Customer#00003', 'Address3', 3, '10-3333333', 3000.00, 'AUTOMOBILE', 'comment3');
INSERT INTO customer VALUES (4, 'Customer#00004', 'Address4', 1, '10-4444444', 4000.00, 'FURNITURE', 'comment4');
INSERT INTO customer VALUES (5, 'Customer#00005', 'Address5', 2, '10-5555555', 5000.00, 'MACHINERY', 'comment5');
EOSQL
"#,
            db_path
        );

        Command::new("bash")
            .args(&["-c", &bash_script])
            .output()
            .map_err(|e| format!("Failed to create database: {}", e))?;

        Ok(db_path)
    }

    #[test]
    fn test_tpch_sqlite_basic_query() {
        let db_path = create_sqlite_test_db().expect("Failed to create test DB");
        let sql = "SELECT COUNT(*) FROM lineitem WHERE l_quantity > 10";

        let result = run_sqlite_query(sql, &db_path);
        assert!(result.is_ok(), "SQLite query should succeed");

        let (count, elapsed) = result.unwrap();
        println!("SQLite Q1: {} rows in {:?}", count, elapsed);
        assert!(count > 0, "Should return at least 1 row");
    }

    #[test]
    fn test_tpch_sqlite_filter_query() {
        let db_path = create_sqlite_test_db().expect("Failed to create test DB");
        let sql = "SELECT * FROM lineitem WHERE l_quantity > 10";

        let result = run_sqlite_query(sql, &db_path);
        assert!(result.is_ok(), "SQLite query should succeed");

        let (count, elapsed) = result.unwrap();
        println!("SQLite Q2 (Filter): {} rows in {:?}", count, elapsed);
        assert!(count > 0, "Should return filtered rows");
    }

    #[test]
    fn test_tpch_sqlite_join_query() {
        let db_path = create_sqlite_test_db().expect("Failed to create test DB");
        let sql = "SELECT c.c_name, o.o_orderkey FROM customer c JOIN orders o ON c.c_custkey = o.o_custkey WHERE c.c_mktsegment = 'AUTOMOBILE'";

        let result = run_sqlite_query(sql, &db_path);
        assert!(result.is_ok(), "SQLite JOIN should succeed");

        let (count, elapsed) = result.unwrap();
        println!("SQLite Q3 (Join): {} rows in {:?}", count, elapsed);
    }

    #[test]
    fn test_tpch_sqlite_aggregate_query() {
        let db_path = create_sqlite_test_db().expect("Failed to create test DB");
        let sql = "SELECT COUNT(*) FROM lineitem";

        let result = run_sqlite_query(sql, &db_path);
        assert!(result.is_ok(), "SQLite aggregation should succeed");

        let (count, elapsed) = result.unwrap();
        println!("SQLite Q4 (Aggregate): {} rows in {:?}", count, elapsed);
    }

    #[test]
    fn test_tpch_sqlite_performance_comparison() {
        let db_path = create_sqlite_test_db().expect("Failed to create test DB");
        let iterations = 100;

        println!(
            "\n=== TPC-H SQLite Performance Benchmark ({} iterations) ===",
            iterations
        );

        let queries = vec![
            ("Q1", "SELECT * FROM lineitem WHERE l_quantity > 10"),
            ("Q2", "SELECT * FROM lineitem WHERE l_extendedprice > 10000"),
            (
                "Q3",
                "SELECT * FROM customer WHERE c_mktsegment = 'AUTOMOBILE'",
            ),
        ];

        for (name, sql) in queries {
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = run_sqlite_query(sql, &db_path);
            }
            let elapsed = start.elapsed();
            let avg_ms = (elapsed.as_secs_f64() * 1000.0) / iterations as f64;
            println!("SQLite {}: avg {:.3}ms (total {:?})", name, avg_ms, elapsed);
        }
    }

    #[test]
    fn test_database_availability() {
        println!("\n=== Database Availability ===");
        println!(
            "SQLite: {}",
            if check_database("sqlite") {
                "✓ Available"
            } else {
                "✗ Not Found"
            }
        );
        println!(
            "MySQL: {}",
            if check_database("mysql") {
                "✓ Available"
            } else {
                "✗ Not Found"
            }
        );
        println!(
            "PostgreSQL: {}",
            if check_database("postgres") {
                "✓ Available"
            } else {
                "✗ Not Found"
            }
        );
    }

    #[test]
    fn test_tpch_query_execution_summary() {
        println!("\n=== TPC-H Query Execution Summary ===");
        println!("Query | Database | Rows | Time | Status");
        println!("------|----------|------|------|-------");

        if check_database("sqlite") {
            let db_path = create_sqlite_test_db().expect("Failed to create test DB");
            let queries = vec![
                ("Q1", "SELECT COUNT(*) FROM lineitem WHERE l_quantity > 10"),
                (
                    "Q2",
                    "SELECT COUNT(*) FROM lineitem WHERE l_extendedprice > 10000",
                ),
                (
                    "Q3",
                    "SELECT COUNT(*) FROM customer WHERE c_mktsegment = 'AUTOMOBILE'",
                ),
            ];

            for (name, sql) in queries {
                match run_sqlite_query(sql, &db_path) {
                    Ok((rows, elapsed)) => {
                        println!("{} | SQLite | {} | {:?} | ✓", name, rows, elapsed);
                    }
                    Err(e) => {
                        println!("{} | SQLite | - | - | ✗: {}", name, e);
                    }
                }
            }
        } else {
            println!("Q1-Q3 | SQLite | - | - | ✗ Not Available");
        }

        if check_database("mysql") {
            println!("Q1-Q3 | MySQL | - | - | (MySQL tests not yet implemented)");
        } else {
            println!("Q1-Q3 | MySQL | - | - | ✗ Not Available");
        }

        if check_database("postgres") {
            println!("Q1-Q3 | PostgreSQL | - | - | (PostgreSQL tests not yet implemented)");
        } else {
            println!("Q1-Q3 | PostgreSQL | - | - | ✗ Not Available");
        }

        println!("\nNote: MySQL and PostgreSQL benchmarks require running database servers.");
        println!("For full cross-database comparison, ensure MySQL and PostgreSQL are running.");
    }
}
