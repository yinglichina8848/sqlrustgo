#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$PROJECT_ROOT"

mkdir -p target/coverage

# NOTE: planner_property_tests.rs has pre-existing compile errors (API incompatibility).
# It is excluded from coverage via --ignore-filename-regex.

echo "[report] HTML"
cargo llvm-cov report \
  --html \
  --output-dir target/coverage \
  --ignore-filename-regex 'planner_property_tests'

echo "[report] JSON"
cargo llvm-cov report \
  --json > target/coverage/coverage.json \
  --ignore-filename-regex 'planner_property_tests'

echo "[report] LCOV"
cargo llvm-cov report \
  --lcov > target/coverage/lcov.info \
  --ignore-filename-regex 'planner_property_tests'

echo ""
echo "coverage reports generated:"
ls -lh target/coverage/html/index.html target/coverage/coverage.json target/coverage/lcov.info 2>/dev/null
