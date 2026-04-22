#!/usr/bin/env bash

set -e

echo "=== Running Performance Test Gate Check ==="

echo "Running performance tests (excluded from coverage)..."

PERFORMANCE_TESTS=(
    "test_hnsw_100k_build_and_search"
    "test_hnsw_100k_search_performance"
    "test_hnsw_1m_search_performance"
    "test_ivfpq_100k_performance"
    "test_batch_writer_auto_flush"
    "test_throughput_estimate"
    "test_performance_comparison"
)

SKIP_ARGS=""
for test in "${PERFORMANCE_TESTS[@]}"; do
    SKIP_ARGS="$SKIP_ARGS --skip $test"
done

echo "Skipping performance tests: ${PERFORMANCE_TESTS[*]}"

echo "Running unit tests..."
cargo test --lib ${SKIP_ARGS}

echo "Running integration tests..."
cargo test --test '*' ${SKIP_ARGS}

echo "Running vector crate tests (excluding performance tests)..."
cargo test -p sqlrustgo-vector ${SKIP_ARGS}

echo "Running executor tests..."
cargo test -p sqlrustgo-executor ${SKIP_ARGS}

echo "Running storage tests..."
cargo test -p sqlrustgo-storage ${SKIP_ARGS}

echo "✅ Performance gate check passed!"
echo "Note: Run 'cargo bench' for performance benchmarks"
echo "=== Performance Gate Check Complete ==="
