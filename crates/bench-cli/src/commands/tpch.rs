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
            "Q1" => {
                "SELECT l_returnflag, SUM(l_quantity) FROM lineitem GROUP BY l_returnflag"
            }
            "Q3" => "SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey",
            "Q6" => {
                "SELECT SUM(l_extendedprice) FROM lineitem WHERE l_quantity < 24 AND l_shipdate >= '1994-01-01'"
            }
            "Q10" => "SELECT c_custkey, SUM(l_extendedprice) FROM customer JOIN orders ON c_custkey = o_custkey JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate >= '1993-10-01' GROUP BY c_custkey",

            // Q19 - Discounted revenue query (simplified)
            "Q19" => "SELECT SUM(l_extendedprice * (1 - l_discount)) AS revenue FROM lineitem, part WHERE p_partkey = l_partkey AND p_brand = 'Brand#12' AND p_container IN ('SM CASE', 'SM BOX', 'SM PACK', 'SM PKG') AND l_quantity >= 1 AND l_quantity <= 11 AND p_size BETWEEN 1 AND 5 AND l_shipmode IN ('AIR', 'AIR REG') AND l_shipinstruct = 'DELIVER IN PERSON'",

            // Q20 - Potential promotion supplier (simplified)
            "Q20" => "SELECT s_name, s_address FROM supplier, nation WHERE s_nationkey = n_nationkey AND n_name = 'CANADA' ORDER BY s_name LIMIT 100",

            // Q21 - Suppliers with late orders (simplified)
            "Q21" => "SELECT s_name, COUNT(*) AS numwait FROM supplier, lineitem l1, orders, nation WHERE s_suppkey = l1.l_suppkey AND o_orderkey = l1.l_orderkey AND o_orderstatus = 'F' AND l1.l_receiptdate > l1.l_commitdate AND s_nationkey = n_nationkey AND n_name = 'SAUDI ARABIA' GROUP BY s_name ORDER BY numwait DESC, s_name LIMIT 100",

            // Q22 - Customer geography query (simplified)
            "Q22" => "SELECT SUBSTRING(c_phone FROM 1 FOR 2) AS cntrycode, COUNT(*) AS numcust, SUM(c_acctbal) AS totacctbal FROM customer WHERE SUBSTRING(c_phone FROM 1 FOR 2) IN ('13', '31', '23', '29', '30', '18', '17') GROUP BY cntrycode ORDER BY cntrycode",

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
