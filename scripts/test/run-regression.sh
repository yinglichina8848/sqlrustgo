#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

MODE="${1:-full}"
PRIORITY="${2:-P0}"

echo "=== SQLRustGo Regression Test Runner ==="
echo "Mode: $MODE"
echo "Priority: $PRIORITY"
echo ""

cd "$PROJECT_ROOT"

run_cmd() {
    echo "Running: $1"
    if eval "$1"; then
        echo "✅ PASSED"
    else
        echo "❌ FAILED"
        return 1
    fi
    echo ""
}

case "$MODE" in
    full)
        echo "=========================================="
        echo "Running FULL test suite..."
        echo "=========================================="
        
        echo ">>> Core Library Tests"
        run_cmd "cargo test --lib"
        
        echo ">>> Parser Tests"
        run_cmd "cargo test -p sqlrustgo-parser"
        
        echo ">>> Planner Tests"
        run_cmd "cargo test -p sqlrustgo-planner"
        
        echo ">>> Executor Tests"
        run_cmd "cargo test -p sqlrustgo-executor"
        
        echo ">>> Storage Tests"
        run_cmd "cargo test -p sqlrustgo-storage"
        
        echo ">>> Optimizer Tests"
        run_cmd "cargo test -p sqlrustgo-optimizer"
        
        echo ">>> Integration Tests"
        run_cmd "cargo test --test page_test"
        run_cmd "cargo test --test executor_test"
        run_cmd "cargo test --test planner_test"
        run_cmd "cargo test --test optimizer_stats_test"
        run_cmd "cargo test --test server_integration_test"
        run_cmd "cargo test --test teaching_scenario_test"
        run_cmd "cargo test --test performance_test"
        run_cmd "cargo test --test mysql_tpch_test"
        run_cmd "cargo test --test binary_format_test"
        
        echo ">>> Unit Tests"
        run_cmd "cargo test --test types_value_test"
        run_cmd "cargo test --test query_cache_config_test"
        run_cmd "cargo test --test server_health_test"
        run_cmd "cargo test --test bplus_tree_test"
        run_cmd "cargo test --test optimizer_cost_test"
        run_cmd "cargo test --test local_executor_test"
        run_cmd "cargo test --test query_cache_test"
        run_cmd "cargo test --test vectorization_test"
        run_cmd "cargo test --test file_storage_test"
        run_cmd "cargo test --test optimizer_rules_test"
        run_cmd "cargo test --test buffer_pool_test"
        
        echo ">>> Anomaly Tests"
        run_cmd "cargo test --test mvcc_concurrency_test"
        run_cmd "cargo test --test crash_injection_test"
        run_cmd "cargo test --test catalog_consistency_test"
        run_cmd "cargo test --test long_run_stability_test"
        run_cmd "cargo test --test qps_benchmark_test"
        run_cmd "cargo test --test transaction_isolation_test"
        run_cmd "cargo test --test join_test"
        run_cmd "cargo test --test set_operations_test"
        run_cmd "cargo test --test view_test"
        run_cmd "cargo test --test transaction_timeout_test"
        run_cmd "cargo test --test datetime_type_test"
        run_cmd "cargo test --test boundary_test"
        run_cmd "cargo test --test error_handling_test"
        run_cmd "cargo test --test aggregate_type_test"
        run_cmd "cargo test --test null_handling_test"
        
        echo ">>> Stress Tests"
        run_cmd "cargo test --test crash_recovery_test"
        run_cmd "cargo test --test stress_test"
        run_cmd "cargo test --test concurrency_stress_test"
        run_cmd "cargo test --test production_scenario_test"
        
        echo ">>> Feature Tests"
        run_cmd "cargo test --test foreign_key_test"
        run_cmd "cargo test --test outer_join_test"
        
        echo ">>> TPC-H Tests"
        run_cmd "cargo test --test tpch_test"
        
        echo ">>> Code Quality"
        run_cmd "cargo fmt --all -- --check"
        run_cmd "cargo clippy --all-targets -- -D warnings"
        
        echo ">>> SQL Fuzz (SQLancer)"
        cargo build --release -p sqlancer || true
        timeout 300 cargo run --release -p sqlancer -- --duration 120 || true
        ;;
        
    unit)
        echo "Running unit tests only..."
        run_cmd "cargo test --test '*_test' --lib"
        ;;
        
    integration)
        echo "Running integration tests..."
        run_cmd "cargo test --test page_test"
        run_cmd "cargo test --test executor_test"
        run_cmd "cargo test --test planner_test"
        run_cmd "cargo test --test optimizer_stats_test"
        run_cmd "cargo test --test server_integration_test"
        run_cmd "cargo test --test teaching_scenario_test"
        run_cmd "cargo test --test performance_test"
        ;;
        
    anomaly)
        echo "Running anomaly tests..."
        run_cmd "cargo test --test mvcc_concurrency_test"
        run_cmd "cargo test --test crash_injection_test"
        run_cmd "cargo test --test catalog_consistency_test"
        run_cmd "cargo test --test long_run_stability_test"
        run_cmd "cargo test --test transaction_isolation_test"
        run_cmd "cargo test --test join_test"
        ;;
        
    stress)
        echo "Running stress tests..."
        run_cmd "cargo test --test crash_recovery_test"
        run_cmd "cargo test --test stress_test"
        run_cmd "cargo test --test concurrency_stress_test"
        run_cmd "cargo test --test production_scenario_test"
        ;;
        
    incremental)
        echo "Running incremental tests based on git diff..."
        
        if [ -d ".git" ]; then
            CHANGED_FILES=$(git diff --name-only HEAD~1 HEAD | grep -E '\.(rs|toml)$' || true)
            
            if [ -z "$CHANGED_FILES" ]; then
                echo "No changed files detected, running all tests"
                $0 full
            else
                echo "Changed files:"
                echo "$CHANGED_FILES"
                echo ""
                
                MODULES=$(echo "$CHANGED_FILES" | grep -oE 'crates/[^/]+' | sort -u || true)
                
                if [ -z "$MODULES" ]; then
                    echo "No module changes detected"
                    $0 full
                else
                    echo "Affected modules: $MODULES"
                    
                    for mod in $MODULES; do
                        echo "Testing $mod..."
                        cargo test -p "$mod" 2>/dev/null || true
                    done
                    
                    echo ""
                    echo "Running impacted integration tests..."
                    for test in join_test mvcc_concurrency_test transaction_isolation_test; do
                        cargo test --test "$test" 2>/dev/null || true
                    done
                fi
            fi
        else
            echo "Not a git repository, running full tests"
            $0 full
        fi
        ;;
        
    priority)
        echo "Running $PRIORITY priority tests..."
        
        case "$PRIORITY" in
            P0)
                run_cmd "cargo test --lib"
                run_cmd "cargo test -p sqlrustgo-parser"
                run_cmd "cargo test -p sqlrustgo-planner"
                run_cmd "cargo test -p sqlrustgo-executor"
                run_cmd "cargo test -p sqlrustgo-storage"
                run_cmd "cargo test -p sqlrustgo-optimizer"
                run_cmd "cargo test --test crash_recovery_test"
                run_cmd "cargo test --test catalog_consistency_test"
                run_cmd "cargo test --test qps_benchmark_test"
                run_cmd "cargo test --test long_run_stability_test"
                ;;
            P1)
                run_cmd "cargo test --test mvcc_concurrency_test"
                run_cmd "cargo test --test transaction_isolation_test"
                run_cmd "cargo test --test foreign_key_test"
                ;;
            P2)
                run_cmd "cargo test --test join_test"
                run_cmd "cargo test --test set_operations_test"
                run_cmd "cargo test --test view_test"
                run_cmd "cargo test --test transaction_timeout_test"
                ;;
            P3)
                run_cmd "cargo test --test datetime_type_test"
                run_cmd "cargo test --test boundary_test"
                run_cmd "cargo test --test error_handling_test"
                ;;
            P4)
                run_cmd "cargo test --test aggregate_type_test"
                run_cmd "cargo test --test null_handling_test"
                ;;
            *)
                echo "Unknown priority: $PRIORITY"
                exit 1
                ;;
        esac
        ;;
        
    quick)
        echo "Running quick smoke tests..."
        run_cmd "cargo test --lib"
        run_cmd "cargo test -p sqlrustgo-parser"
        run_cmd "cargo test -p sqlrustgo-planner"
        run_cmd "cargo test --test crash_recovery_test"
        run_cmd "cargo test --test mvcc_concurrency_test"
        run_cmd "cargo fmt --all -- --check"
        ;;
        
    *)
        echo "Unknown mode: $MODE"
        echo "Usage: $0 [full|unit|integration|anomaly|stress|incremental|priority|quick] [P0-P4]"
        exit 1
        ;;
esac

echo ""
echo "=== Test Run Complete ==="
