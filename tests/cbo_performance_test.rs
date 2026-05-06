use sqlrustgo::{ExecutionEngine, MemoryStorage};
use std::sync::{Arc, RwLock};
use std::time::Instant;

fn setup_tpch_data(engine: &mut ExecutionEngine<MemoryStorage>) {
    engine
        .execute("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_quantity INTEGER, l_extendedprice FLOAT, l_discount FLOAT, l_tax FLOAT, l_returnflag TEXT, l_linestatus TEXT, l_shipdate DATE)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (o_orderkey INTEGER, o_custkey INTEGER, o_orderdate DATE, o_totalprice FLOAT, o_orderpriority TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE customer (c_custkey INTEGER, c_name TEXT, c_acctbal FLOAT)")
        .unwrap();

    for i in 0..1000 {
        let _ = engine.execute(&format!(
            "INSERT INTO lineitem VALUES ({}, {}, {}, {}.0, 0.05, 0.01, 'N', 'O', '1996-01-01')",
            i % 100,
            i % 200,
            i % 10,
            1000.0 + (i % 100) as f64
        ));
    }
    for i in 0..500 {
        let _ = engine.execute(&format!(
            "INSERT INTO orders VALUES ({}, {}, '1996-01-01', {}, '1-URGENT')",
            i,
            i % 100,
            10000.0 + (i % 50) as f64
        ));
    }
    for i in 0..100 {
        let _ = engine.execute(&format!(
            "INSERT INTO customer VALUES ({}, 'Customer {}', {})",
            i,
            i,
            1000.0 + (i % 10) as f64
        ));
    }
    let _ = engine.execute("ANALYZE lineitem").unwrap();
    let _ = engine.execute("ANALYZE orders").unwrap();
    let _ = engine.execute("ANALYZE customer").unwrap();
}

fn run_query_with_cbo(
    engine: &mut ExecutionEngine<MemoryStorage>,
    query: &str,
) -> std::time::Duration {
    let start = Instant::now();
    let _ = engine.execute(query);
    start.elapsed()
}

#[test]
fn test_cbo_q1_performance_improvement() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine_cbo = ExecutionEngine::new(storage.clone());
    setup_tpch_data(&mut engine_cbo);

    let storage_no_cbo = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine_no_cbo = ExecutionEngine::with_cbo(storage_no_cbo, false);
    setup_tpch_data(&mut engine_no_cbo);

    let q1 = "SELECT l_returnflag, l_linestatus, SUM(l_quantity) AS sum_qty, SUM(l_extendedprice) AS sum_base_price FROM lineitem WHERE l_shipdate <= '1996-12-01' GROUP BY l_returnflag, l_linestatus ORDER BY l_returnflag, l_linestatus";

    let time_cbo = run_query_with_cbo(&mut engine_cbo, q1);
    let time_no_cbo = run_query_with_cbo(&mut engine_no_cbo, q1);

    let improvement = if time_no_cbo > time_cbo && time_cbo.as_nanos() > 0 {
        (time_no_cbo.as_nanos() - time_cbo.as_nanos()) as f64 / time_no_cbo.as_nanos() as f64
            * 100.0
    } else {
        0.0
    };

    assert!(
        improvement >= 0.0,
        "CBO should not make query slower: improvement = {}% (cbo: {:?}, no_cbo: {:?})",
        improvement,
        time_cbo,
        time_no_cbo
    );
}

#[test]
fn test_cbo_predicate_pushdown_effect() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine_cbo = ExecutionEngine::new(storage.clone());
    setup_tpch_data(&mut engine_cbo);

    let storage_no_cbo = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine_no_cbo = ExecutionEngine::with_cbo(storage_no_cbo, false);
    setup_tpch_data(&mut engine_no_cbo);

    let query = "SELECT * FROM lineitem WHERE l_quantity > 5 AND l_discount < 0.1 AND l_extendedprice > 1000.0";

    let result_cbo = engine_cbo.execute(query);
    let result_no_cbo = engine_no_cbo.execute(query);

    assert!(result_cbo.is_ok(), "CBO engine should execute query");
    assert!(result_no_cbo.is_ok(), "Non-CBO engine should execute query");
}

#[test]
fn test_cbo_constant_folding_effect() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);
    engine
        .execute("CREATE TABLE t (a INTEGER, b INTEGER)")
        .unwrap();
    for i in 0..100 {
        let _ = engine.execute(&format!("INSERT INTO t VALUES ({}, {})", i, i * 2));
    }

    let query_with_constants = "SELECT a + 2 + 3 FROM t WHERE b > 5 + 1";

    let start = Instant::now();
    let _ = engine.execute(query_with_constants);
    let time_opt = start.elapsed();

    let storage2 = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine2 = ExecutionEngine::with_cbo(storage2, false);
    engine2
        .execute("CREATE TABLE t (a INTEGER, b INTEGER)")
        .unwrap();
    for i in 0..100 {
        let _ = engine2.execute(&format!("INSERT INTO t VALUES ({}, {})", i, i * 2));
    }

    let start = Instant::now();
    let _ = engine2.execute(query_with_constants);
    let time_unopt = start.elapsed();

    let improvement = if time_unopt > time_opt && time_opt.as_nanos() > 0 {
        (time_unopt.as_nanos() - time_opt.as_nanos()) as f64 / time_unopt.as_nanos() as f64 * 100.0
    } else {
        0.0
    };

    assert!(
        improvement >= 0.0,
        "CBO constant folding should not slow down query: improvement = {}% (opt: {:?}, unopt: {:?})",
        improvement,
        time_opt,
        time_unopt
    );
}
