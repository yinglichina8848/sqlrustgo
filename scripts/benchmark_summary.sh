#!/bin/bash
# Simple benchmark runner

echo "=== SQLRustGo v1.6.1 TPC-H Benchmark ==="
echo ""

# Run TPC-H Q1
echo "Running TPC-H Q1..."
cargo bench --bench tpch_bench -- tpch_q1/pricing_summary 2>&1 | grep "time:" | head -1

echo "Running TPC-H Q3..."
cargo bench --bench tpch_bench -- tpch_q3/shipping_priority 2>&1 | grep "time:" | head -1

echo "Running TPC-H Q6..."
cargo bench --bench tpch_bench -- tpch_q6/revenue_query 2>&1 | grep "time:" | head -1

echo "Running TPC-H Q10..."
cargo bench --bench tpch_bench -- tpch_q10/customer_revenue 2>&1 | grep "time:" | head -1

echo ""
echo "=== Aggregation Benchmarks ==="
echo "Running SUM..."
cargo bench --bench tpch_bench -- aggregation/sum_amount 2>&1 | grep "time:" | head -1

echo "Running AVG..."
cargo bench --bench tpch_bench -- aggregation/avg_amount 2>&1 | grep "time:" | head -1

echo "Running COUNT..."
cargo bench --bench tpch_bench -- aggregation/count_all 2>&1 | grep "time:" | head -1
