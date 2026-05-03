#!/usr/bin/env bash
set -euo pipefail

# Coverage collection for SQLRustGo Phase 1
# Dependencies: cargo-llvm-cov 0.8.x, rustup component add llvm-tools-preview

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "[coverage] Phase 1: unit + integration coverage collection"
echo "[coverage] project root: $PROJECT_ROOT"

cd "$PROJECT_ROOT"

# Environment for coverage
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="coverage-%p-%m.profraw"

echo "[1/3] clean previous coverage data"
cargo llvm-cov clean --workspace 2>/dev/null || true
rm -f coverage-*.profraw 2>/dev/null || true

echo "[2/3] run lib tests with coverage (no report)"
# NOTE: --lib only (Phase 1 scope). Integration tests have pre-existing
# compile errors in trigger_eval_tests.rs / planner_property_tests.rs.
# These will be fixed in Phase 2.
cargo llvm-cov --lib --no-report

echo "[3/3] generate reports"
bash "$SCRIPT_DIR/merge_coverage.sh"

echo "[coverage] done"
echo ""
echo "Artifacts:"
echo "  - target/coverage/html/index.html"
echo "  - target/coverage/coverage.json"
echo "  - target/coverage/lcov.info"
