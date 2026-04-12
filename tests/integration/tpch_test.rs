//! PB-05: TPC-H Query Performance Tests
//!
//! These tests verify TPC-H query execution using real SQLRustGo ExecutionEngine
//! with MemoryStorage (not MockStorage).

#[cfg(test)]
mod tests {
    use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
    use std::sync::{Arc, RwLock};
    use std::time::Instant;

    fn create_engine() -> ExecutionEngine {
        ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
    }

    fn setup_tpch_data() -> ExecutionEngine {
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

        engine
    }

    #[test]
    fn test_tpch_scan_query() {
        let mut engine = setup_tpch_data();

        let sql = "SELECT * FROM lineitem";

        let start = Instant::now();
        let result = engine.execute(parse(sql).unwrap()).unwrap();
        let elapsed = start.elapsed();

        println!(
            "TPC-H Scan Query: {:?} ({} rows)",
            elapsed,
            result.rows.len()
        );

        assert!(!result.rows.is_empty(), "Scan should return results");
        assert_eq!(result.rows.len(), 5, "Should have 5 lineitems");
    }

    #[test]
    fn test_tpch_filter_query() {
        let mut engine = setup_tpch_data();

        let sql = "SELECT * FROM lineitem WHERE l_quantity > 10";

        let start = Instant::now();
        let result = engine.execute(parse(sql).unwrap()).unwrap();
        let elapsed = start.elapsed();

        println!(
            "TPC-H Filter Query: {:?} ({} rows)",
            elapsed,
            result.rows.len()
        );

        assert!(
            result.rows.len() >= 2,
            "Filter should return at least 2 rows with qty > 10"
        );
    }

    #[test]
    fn test_tpch_join_query() {
        let mut engine = setup_tpch_data();

        let sql = "SELECT c.c_name, o.o_orderkey FROM customer c JOIN orders o ON c.c_custkey = o.o_custkey WHERE c.c_mktsegment = 'AUTOMOBILE'";

        let start = Instant::now();
        let result = engine.execute(parse(sql).unwrap()).unwrap();
        let elapsed = start.elapsed();

        println!(
            "TPC-H Join Query: {:?} ({} rows)",
            elapsed,
            result.rows.len()
        );

        assert!(!result.rows.is_empty(), "Join should return results");
    }

    #[test]
    fn test_tpch_count_aggregate() {
        let mut engine = setup_tpch_data();

        let sql = "SELECT COUNT(*) FROM lineitem";

        let start = Instant::now();
        let result = engine.execute(parse(sql).unwrap()).unwrap();
        let elapsed = start.elapsed();

        println!(
            "TPC-H Count Aggregate: {:?} ({} rows)",
            elapsed,
            result.rows.len()
        );

        assert!(!result.rows.is_empty(), "Count should return results");
    }

    #[test]
    fn test_tpch_sum_aggregate() {
        let mut engine = setup_tpch_data();

        let sql = "SELECT SUM(l_quantity) FROM lineitem";

        let start = Instant::now();
        let result = engine.execute(parse(sql).unwrap()).unwrap();
        let elapsed = start.elapsed();

        println!(
            "TPC-H Sum Aggregate: {:?} ({} rows)",
            elapsed,
            result.rows.len()
        );

        assert!(!result.rows.is_empty(), "Sum should return results");
    }

    #[test]
    fn test_tpch_q6_forecast_revenue() {
        let mut engine = setup_tpch_data();

        let sql = "SELECT SUM(l_extendedprice) FROM lineitem WHERE l_discount > 0";

        let start = Instant::now();
        let result = engine.execute(parse(sql).unwrap()).unwrap();
        let elapsed = start.elapsed();

        println!(
            "TPC-H Q6 (Forecast Revenue): {:?} ({} rows)",
            elapsed,
            result.rows.len()
        );

        assert!(
            elapsed.as_secs_f64() < 1.0,
            "Q6 should complete in under 1 second"
        );
    }

    #[test]
    fn test_tpch_all_queries_execute() {
        let mut engine = setup_tpch_data();

        let queries = vec![
            ("Q1", "SELECT COUNT(*) FROM lineitem"),
            ("Q2", "SELECT COUNT(*) FROM orders"),
            ("Q3", "SELECT COUNT(*) FROM customer"),
            ("Q4", "SELECT * FROM lineitem WHERE l_quantity > 10"),
            ("Q5", "SELECT SUM(l_extendedprice) FROM lineitem"),
            (
                "Q6",
                "SELECT SUM(l_extendedprice) FROM lineitem WHERE l_discount > 0",
            ),
        ];

        for (name, sql) in queries {
            let start = Instant::now();
            let result = engine.execute(parse(sql).unwrap());
            let elapsed = start.elapsed();

            println!(
                "SQLRustGo {}: {:?} ({} rows)",
                name,
                elapsed,
                result.as_ref().map(|r| r.rows.len()).unwrap_or(0)
            );

            assert!(
                result.is_ok(),
                "Query {} should execute successfully: {:?}",
                name,
                result.err()
            );
            assert!(
                elapsed.as_secs_f64() < 1.0,
                "{} should complete in under 1 second",
                name
            );
        }
    }

    #[test]
    fn test_tpch_performance_benchmark() {
        let mut engine = setup_tpch_data();

        let sql = "SELECT * FROM lineitem WHERE l_quantity > 10";
        let iterations = 100;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = engine.execute(parse(sql).unwrap()).unwrap();
        }
        let total_elapsed = start.elapsed();
        let avg_ms = (total_elapsed.as_secs_f64() * 1000.0) / iterations as f64;

        println!(
            "TPC-H Performance: {} iterations, avg {:.3}ms, total {:?}",
            iterations, avg_ms, total_elapsed
        );

        assert!(avg_ms < 10.0, "Queries too slow: {:.2}ms avg", avg_ms);
    }

    #[test]
    fn test_between_operator() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE t (a INT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO t VALUES (5), (10), (15)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("SELECT * FROM t WHERE a BETWEEN 1 AND 10").unwrap())
            .unwrap();
        assert_eq!(result.rows.len(), 2);

        let result = engine
            .execute(parse("SELECT * FROM t WHERE a BETWEEN 6 AND 20").unwrap())
            .unwrap();
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_in_list_operator() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE t (a TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO t VALUES ('MAIL'), ('SHIP'), ('AIR'), ('TRUCK')").unwrap())
            .unwrap();

        // Test IN with 2 values
        let result = engine
            .execute(parse("SELECT * FROM t WHERE a IN ('MAIL', 'SHIP')").unwrap())
            .unwrap();
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_in_list_operator_three_values() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE t (a TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO t VALUES ('MAIL'), ('SHIP'), ('AIR'), ('TRUCK')").unwrap())
            .unwrap();

        // Test IN with 3 values
        let result = engine
            .execute(parse("SELECT * FROM t WHERE a IN ('MAIL', 'AIR', 'TRUCK')").unwrap())
            .unwrap();
        assert_eq!(result.rows.len(), 3);
    }
}
