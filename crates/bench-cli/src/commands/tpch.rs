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
            "Q2" => {
                "SELECT s_acctbal, s_name, n_name, p_partkey FROM part, supplier, partsupp, nation, region WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15 AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'EUROPE' ORDER BY s_acctbal DESC LIMIT 10"
            }
            "Q3" => "SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey",
            "Q5" => "SELECT n_name, SUM(l_extendedprice) FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' GROUP BY n_name",
            "Q6" => {
                "SELECT SUM(l_extendedprice) FROM lineitem WHERE l_quantity < 24 AND l_shipdate >= '1994-01-01'"
            }
            "Q7" => "SELECT n1.n_name AS supp_nation, n2.n_name AS cust_nation, SUM(l_extendedprice) FROM supplier, lineitem, orders, customer, nation n1, nation n2 WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND c_custkey = o_custkey AND s_nationkey = n1.n_nationkey AND c_nationkey = n2.n_nationkey GROUP BY n1.n_name, n2.n_name",
            "Q10" => "SELECT c_custkey, SUM(l_extendedprice) FROM customer JOIN orders ON c_custkey = o_custkey JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate >= '1993-10-01' GROUP BY c_custkey",
            "Q12" => "SELECT l_shipmode, COUNT(*) FROM orders, lineitem WHERE l_orderkey = o_orderkey AND l_shipmode IN ('MAIL', 'SHIP') GROUP BY l_shipmode",
            "Q14" => "SELECT SUM(l_extendedprice) FROM lineitem, part WHERE l_partkey = p_partkey AND l_shipdate >= '1995-09-01'",
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
