#!/bin/bash
# =============================================================================
# replay.sh - CI Replay 机制 (V3 Enhancement #4)
# =============================================================================
# 用法: ./scripts/ci/replay.sh ci_artifacts/
# 本地一键复现 CI job 环境

set -euo pipefail

ARTIFACTS_DIR="${1:-ci_artifacts}"

if [ ! -d "$ARTIFACTS_DIR" ]; then
    echo "Usage: $0 <ci_artifacts_dir>"
    echo "Example: $0 /path/to/pr-123-artifacts/"
    exit 1
fi

echo "[replay] ======================================"
echo "[replay] SQLRustGo CI Replay"
echo "[replay] ======================================"

REQUIRED="commit_sha affected_crates.txt"
for f in $REQUIRED; do
    if [ ! -f "$ARTIFACTS_DIR/$f" ]; then
        echo "[replay] ERROR: Missing required file: $f"
        exit 1
    fi
done

COMMIT=$(cat "$ARTIFACTS_DIR/commit_sha")
AFFECTED_CRATES=$(cat "$ARTIFACTS_DIR/affected_crates.txt")

echo "[replay] Commit:    $COMMIT"
echo "[replay] Crates:    $AFFECTED_CRATES"

if [ -f "$ARTIFACTS_DIR/env.txt" ]; then
    echo "[replay] Restoring environment..."
    set -a
    source "$ARTIFACTS_DIR/env.txt"
    set +a
fi

echo ""
echo "[replay] === Step 1: Checkout ==="
if [ -d ".git" ]; then
    git fetch origin "$COMMIT"
    git checkout "$COMMIT"
else
    echo "[replay] WARNING: Not a git repo, skipping checkout"
fi

echo ""
echo "[replay] === Step 2: Setup Rust ==="
if command -v rustup &>/dev/null; then
    rustup default stable
fi

echo ""
echo "[replay] === Step 3: Build ==="
cargo build --workspace 2>&1 | tail -5

echo ""
echo "[replay] === Step 4: Run affected crate tests ==="
if [ -n "$AFFECTED_CRATES" ]; then
    for crate in $AFFECTED_CRATES; do
        if [ "$crate" = "(workspace)" ] || [ -z "$crate" ]; then
            echo "[replay] Full workspace test"
            cargo test --workspace 2>&1 | tail -10
        else
            echo "[replay] Testing: $crate"
            cargo test -p "$crate" 2>&1 | tail -3 || true
        fi
    done
else
    echo "[replay] No affected crates - running quick check"
    cargo check --workspace
fi

echo ""
echo "[replay] === Replay complete ==="
echo "[replay] Full test: cargo test --workspace"
echo "[replay] Coverage: cargo llvm-cov --html"
