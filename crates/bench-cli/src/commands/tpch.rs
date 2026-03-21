use crate::cli::TpchArgs;
use crate::metrics::LatencyCollector;
use crate::reporter::{BenchmarkResult, QueryResult};
use sqlrustgo::{parse, ExecutionEngine};
use sqlrustgo_storage::MemoryStorage;
use std::sync::Arc;
use std::time::Instant;

pub fn run(args: TpchArgs) -> BenchmarkResult {
    let mut result = BenchmarkResult::new(
        "tpch".to_string(),
        serde_json::json!({
            "scale": args.scale,
            "iterations": args.iterations,
            "queries": args.queries,
        }),
    );

    let queries: Vec<&str> = args.queries.split(',').collect();

    for query_name in queries {
        let sql = match query_name.trim() {
            // Existing queries
            "Q1" => {
                "SELECT l_returnflag, SUM(l_quantity) FROM lineitem GROUP BY l_returnflag"
            }
            "Q3" => "SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey",
            "Q4" => "SELECT o_orderpriority, COUNT(*) FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' GROUP BY o_orderpriority",
            "Q6" => {
                "SELECT SUM(l_extendedprice) FROM lineitem WHERE l_quantity < 24 AND l_shipdate >= '1994-01-01'"
            }
            "Q10" => "SELECT c_custkey, SUM(l_extendedprice) FROM customer JOIN orders ON c_custkey = o_custkey JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate >= '1993-10-01' GROUP BY c_custkey",
            "Q13" => "SELECT c_custkey, COUNT(*) FROM customer GROUP BY c_custkey",
            
            // New queries - Q2
            "Q2" => "SELECT s_acctbal, s_name, n_name, p_partkey, p_mfgr FROM part, supplier, partsupp, nation, region WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15 AND p_type LIKE '%BRASS' AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'EUROPE'",
            
            // Q5 - Regional revenue
            "Q5" => "SELECT n_name, SUM(l_extendedprice * (1 - l_discount)) AS revenue FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' AND o_orderdate >= '1994-01-01' AND o_orderdate < '1995-01-01' GROUP BY n_name",
            
            // Q7 - Shipping volume between nations
            "Q7" => "SELECT n1.n_name AS supp_nation, n2.n_name AS cust_nation, l_shipdate, SUM(l_extendedprice * (1 - l_discount)) AS volume FROM supplier, lineitem, orders, customer, nation n1, nation n2 WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND c_custkey = o_custkey AND s_nationkey = n1.n_nationkey AND c_nationkey = n2.n_nationkey AND l_shipdate >= '1995-01-01' AND l_shipdate <= '1996-12-31' GROUP BY n1.n_name, n2.n_name, l_shipdate",
            
            // Q8 - Market share in Americas
            "Q8" => "SELECT extract(year FROM o_orderdate) AS o_year, SUM(l_extendedprice * (1 - l_discount)) AS volume FROM part, supplier, lineitem, orders, customer, nation n1, nation n2, region WHERE p_partkey = l_partkey AND s_suppkey = l_suppkey AND l_orderkey = o_orderkey AND o_custkey = c_custkey AND c_nationkey = n1.n_nationkey AND n1.n_regionkey = r_regionkey AND r_name = 'AMERICA' AND s_nationkey = n2.n_nationkey AND o_orderdate >= '1995-01-01' AND o_orderdate <= '1996-12-31' AND p_type = 'ECONOMY ANODIZED STEEL' GROUP BY o_year",
            
            // Q9 - Profit by nation and year
            "Q9" => "SELECT n_name AS nation, extract(year FROM o_orderdate) AS o_year, SUM(l_extendedprice * (1 - l_discount) - ps_supplycost * l_quantity) AS amount FROM part, supplier, lineitem, partsupp, orders, nation WHERE s_suppkey = l_suppkey AND ps_suppkey = l_suppkey AND ps_partkey = l_partkey AND p_partkey = l_partkey AND o_orderkey = l_orderkey AND s_nationkey = n_nationkey AND p_name LIKE '%green%' GROUP BY n_name, o_year",
            
            // Q11 - German partsupplier value
            "Q11" => "SELECT ps_partkey, SUM(ps_supplycost * ps_availqty) AS value FROM partsupp, supplier, nation WHERE ps_suppkey = s_suppkey AND s_nationkey = n_nationkey AND n_name = 'GERMANY' GROUP BY ps_partkey",
            
            // Q12 - Shipping mode order priority
            "Q12" => "SELECT l_shipmode, COUNT(*) FROM orders, lineitem WHERE l_orderkey = o_orderkey AND l_shipmode IN ('MAIL', 'SHIP') AND l_commitdate < l_receiptdate AND l_shipdate < l_commitdate AND o_orderdate >= '1993-01-01' AND o_orderdate < '1994-01-01' GROUP BY l_shipmode",
            
            // Q14 - Promotion revenue
            "Q14" => "SELECT SUM(CASE WHEN p_type LIKE 'PROMO%' THEN l_extendedprice * (1 - l_discount) ELSE 0 END) AS promo_revenue FROM lineitem, part WHERE l_partkey = p_partkey AND l_shipdate >= '1995-09-01' AND l_shipdate < '1995-10-01'",
            
            // Q15 - Top supplier by revenue
            "Q15" => "SELECT s_suppkey, s_name, s_address, s_phone, SUM(l_extendedprice * (1 - l_discount)) AS total_revenue FROM supplier, lineitem WHERE l_suppkey = s_suppkey AND l_shipdate >= '1996-01-01' AND l_shipdate < '1996-04-01' GROUP BY s_suppkey, s_name, s_address, s_phone",
            
            // Q16 - Parts supplier count
            "Q16" => "SELECT p_brand, p_type, p_size, COUNT(DISTINCT ps_suppkey) AS supplier_cnt FROM partsupp, part WHERE p_partkey = ps_partkey AND p_brand <> 'Brand#45' AND p_type NOT LIKE 'MEDIUM POLISHED%' AND p_size IN (49, 14, 23, 45, 19, 3, 36, 9) GROUP BY p_brand, p_type, p_size",
            
            // Q17 - Average yearly revenue
            "Q17" => "SELECT SUM(l_extendedprice) / 7.0 AS avg_yearly FROM lineitem, part WHERE p_partkey = l_partkey AND p_brand = 'Brand#23' AND p_container = 'MED BOX'",
            
            // Q18 - Large order customers
            "Q18" => "SELECT c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice, SUM(l_quantity) FROM customer, orders, lineitem WHERE o_orderkey = l_orderkey AND c_custkey = o_custkey GROUP BY c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice",
            
            // Q19 - Discounted revenue
            "Q19" => "SELECT SUM(l_extendedprice * (1 - l_discount)) AS revenue FROM lineitem, part WHERE p_partkey = l_partkey AND p_brand = 'Brand#12' AND p_container IN ('SM CASE', 'SM BOX', 'SM PACK', 'SM PKG') AND l_quantity >= 1 AND l_quantity <= 11 AND p_size BETWEEN 1 AND 5",
            
            // Q20 - Canadian suppliers
            "Q20" => "SELECT s_name, s_address FROM supplier, nation WHERE s_nationkey = n_nationkey AND n_name = 'CANADA'",
            
            // Q21 - Suppliers with waiting orders
            "Q21" => "SELECT s_name, COUNT(*) AS numwait FROM supplier, lineitem, orders, nation WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND o_orderstatus = 'F' AND s_nationkey = n_nationkey AND n_name = 'SAUDI ARABIA' GROUP BY s_name",
            
            // Q22 - Customer demographics
            "Q22" => "SELECT SUBSTRING(c_phone FROM 1 FOR 2) AS cntrycode, COUNT(*) AS numcust, SUM(c_acctbal) AS totacctbal FROM customer WHERE SUBSTRING(c_phone FROM 1 FOR 2) IN ('13', '31', '23', '29', '30', '18', '17') AND c_acctbal > 0.00 GROUP BY cntrycode",
            
            _ => continue,
        };

        let mut collector = LatencyCollector::new();

        for _ in 0..args.iterations {
            let iteration_start = Instant::now();
            let storage = Arc::new(MemoryStorage::new());
            let mut engine = ExecutionEngine::new(storage);
            let _ = engine.execute(parse(sql).unwrap());
            collector.record(iteration_start.elapsed().as_nanos() as u64);
        }

        let metrics = collector.into_metrics(args.iterations);

        result.queries.push(QueryResult {
            name: query_name.to_string(),
            avg_latency_ms: metrics.avg_latency_ms,
            min_latency_ms: metrics.min_ms,
            max_latency_ms: metrics.max_ms,
            iterations: args.iterations,
        });
    }

    let total_latencies: Vec<u64> = result
        .queries
        .iter()
        .map(|q| (q.avg_latency_ms as u64) * 1000)
        .collect();
    result.metrics = crate::metrics::BenchmarkMetrics::calculate(&total_latencies, args.iterations);

    result
}
