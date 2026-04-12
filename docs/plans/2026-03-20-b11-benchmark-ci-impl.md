# B-11 Benchmark CI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement GitHub Actions CI workflows for TPC-H/OLTP/Custom benchmarks with baseline comparison

**Architecture:** Two workflows - quick benchmark on PR (SF=0.1) and full suite weekly (SF=0.1/1/10)

**Tech Stack:** GitHub Actions, Rust, Python (comparison script)

---

## Task 1: Create bench-pr.yml Workflow

**Files:**
- Create: `.github/workflows/bench-pr.yml`

**Step 1: Create bench-pr.yml workflow**

```yaml
name: Benchmark PR

on:
  pull_request:
    branches: [develop/**]
  push:
    branches: [develop/**]

jobs:
  quick-benchmark:
    name: Quick Benchmark (SF=0.1)
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Build bench-cli
        run: cargo build --release -p sqlrustgo-bench-cli

      - name: Run TPC-H SF=0.1
        run: |
          mkdir -p benchmarks/results/sf01
          ./target/release/benchmark tpch --sf 0.1 --output benchmarks/results/sf01/tpch.json

      - name: Run OLTP SF=0.1
        run: |
          ./target/release/benchmark oltp --sf 0.1 --output benchmarks/results/sf01/oltp.json

      - name: Run Custom SF=0.1
        run: |
          ./target/release/benchmark custom --sf 0.1 --output benchmarks/results/sf01/custom.json

      - name: Compare with baseline
        run: python scripts/compare_benchmarks.py benchmarks/baselines/sf01 benchmarks/results/sf01

      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results-sf01
          path: benchmarks/results/sf01/
```

**Step 2: Commit**

```bash
git add .github/workflows/bench-pr.yml
git commit -m "feat(ci): add quick benchmark workflow for PRs"
```

---

## Task 2: Create bench-schedule.yml Workflow

**Files:**
- Create: `.github/workflows/bench-schedule.yml`

**Step 1: Create bench-schedule.yml workflow**

```yaml
name: Benchmark Schedule

on:
  schedule:
    - cron: '0 0 * * 1'  # Monday at midnight UTC
  workflow_dispatch:  # Manual trigger

jobs:
  full-benchmark:
    name: Full Benchmark Suite
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Build bench-cli
        run: cargo build --release -p sqlrustgo-bench-cli

      - name: Run SF=0.1 benchmarks
        run: |
          mkdir -p benchmarks/results/sf01
          ./target/release/benchmark tpch --sf 0.1 --output benchmarks/results/sf01/tpch.json
          ./target/release/benchmark oltp --sf 0.1 --output benchmarks/results/sf01/oltp.json
          ./target/release/benchmark custom --sf 0.1 --output benchmarks/results/sf01/custom.json

      - name: Run SF=1 benchmarks
        run: |
          mkdir -p benchmarks/results/sf1
          ./target/release/benchmark tpch --sf 1 --output benchmarks/results/sf1/tpch.json
          ./target/release/benchmark oltp --sf 1 --output benchmarks/results/sf1/oltp.json
          ./target/release/benchmark custom --sf 1 --output benchmarks/results/sf1/custom.json

      - name: Run SF=10 benchmarks
        run: |
          mkdir -p benchmarks/results/sf10
          ./target/release/benchmark tpch --sf 10 --output benchmarks/results/sf10/tpch.json
          ./target/release/benchmark oltp --sf 10 --output benchmarks/results/sf10/oltp.json
          ./target/release/benchmark custom --sf 10 --output benchmarks/results/sf10/custom.json

      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results-full
          path: benchmarks/results/
```

**Step 2: Commit**

```bash
git add .github/workflows/bench-schedule.yml
git commit -m "feat(ci): add weekly full benchmark workflow"
```

---

## Task 3: Create Comparison Script

**Files:**
- Create: `scripts/compare_benchmarks.py`
- Create: `benchmarks/baselines/sf01/.gitkeep`

**Step 1: Create comparison script**

```python
#!/usr/bin/env python3
"""
Compare benchmark results with baselines.
Exit 0 if all metrics within threshold, exit 1 if regression detected.
"""

import json
import sys
from pathlib import Path

THRESHOLD = 0.20  # 20% tolerance

def load_json(path):
    with open(path) as f:
        return json.load(f)

def compare_metric(name, baseline_val, current_val):
    if baseline_val == 0:
        if current_val == 0:
            return True, 0.0
        return False, float('inf')
    
    change = (current_val - baseline_val) / baseline_val
    
    # For QPS, higher is better - fail if decreased
    if 'qps' in name.lower():
        if change < -THRESHOLD:
            return False, change * 100
        return True, change * 100
    
    # For latency, lower is better - fail if increased
    if change > THRESHOLD:
        return False, change * 100
    
    return True, change * 100

def main():
    if len(sys.argv) != 3:
        print("Usage: compare_benchmarks.py <baseline_dir> <results_dir>")
        sys.exit(1)
    
    baseline_dir = Path(sys.argv[1])
    results_dir = Path(sys.argv[2])
    
    all_passed = True
    failures = []
    
    for result_file in results_dir.glob("*.json"):
        baseline_file = baseline_dir / result_file.name
        
        if not baseline_file.exists():
            print(f"WARNING: No baseline for {result_file.name}, skipping")
            continue
        
        baseline = load_json(baseline_file)
        current = load_json(result_file)
        
        # Compare QPS
        passed, change = compare_metric(
            "QPS",
            baseline.get("metrics", {}).get("qps", 0),
            current.get("metrics", {}).get("qps", 0)
        )
        status = "PASS" if passed else "FAIL"
        print(f"{result_file.name} QPS: {change:+.1f}% [{status}]")
        if not passed:
            all_passed = False
            failures.append(f"{result_file.name} QPS: {change:+.1f}%")
        
        # Compare P50
        passed, change = compare_metric(
            "P50",
            baseline.get("metrics", {}).get("p50_ms", 0),
            current.get("metrics", {}).get("p50_ms", 0)
        )
        status = "PASS" if passed else "FAIL"
        print(f"{result_file.name} P50: {change:+.1f}% [{status}]")
        if not passed:
            all_passed = False
            failures.append(f"{result_file.name} P50: {change:+.1f}%")
    
    print()
    if all_passed:
        print("All comparisons passed (within 20% threshold)")
        sys.exit(0)
    else:
        print(f"FAILURES detected: {', '.join(failures)}")
        sys.exit(1)

if __name__ == "__main__":
    main()
```

**Step 2: Make script executable**

```bash
chmod +x scripts/compare_benchmarks.py
```

**Step 3: Create baseline placeholder**

```bash
mkdir -p benchmarks/baselines/sf01
touch benchmarks/baselines/sf01/.gitkeep
```

**Step 4: Commit**

```bash
git add scripts/compare_benchmarks.py benchmarks/baselines/
git commit -m "feat(ci): add benchmark comparison script"
```

---

## Task 4: Create Initial Baseline Data

**Files:**
- Create: `benchmarks/baselines/sf01/tpch.json`
- Create: `benchmarks/baselines/sf01/oltp.json`
- Create: `benchmarks/baselines/sf01/custom.json`

**Step 1: Build and run benchmarks to generate initial baselines**

```bash
cargo build --release -p sqlrustgo-bench-cli

mkdir -p benchmarks/baselines/sf01

./target/release/benchmark tpch --sf 0.1 --output benchmarks/baselines/sf01/tpch.json
./target/release/benchmark oltp --sf 0.1 --output benchmarks/baselines/sf01/oltp.json
./target/release/benchmark custom --sf 0.1 --output benchmarks/baselines/sf01/custom.json
```

**Step 2: Commit baselines**

```bash
git add benchmarks/baselines/
git commit -m "feat(bench): add initial SF=0.1 baseline data"
```

---

## Task 5: Update ci.yml to Include bench-cli

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Add bench-cli to build and test**

The ci.yml should already build all packages via `cargo build --all`, but verify it includes `sqlrustgo-bench-cli`.

---

## Task 6: Test Locally

**Step 1: Run the comparison script locally**

```bash
# First ensure we have baseline and results
ls benchmarks/baselines/sf01/
ls benchmarks/results/sf01/

# Run comparison
python scripts/compare_benchmarks.py benchmarks/baselines/sf01 benchmarks/results/sf01
```

**Step 2: Verify workflow syntax**

```bash
# Install act to test locally (optional)
# Or just verify YAML syntax
python -c "import yaml; yaml.safe_load(open('.github/workflows/bench-pr.yml'))"
```

---

## Verification Commands

```bash
# Format check
cargo fmt --check

# Clippy
cargo clippy --all-targets -- -D warnings

# Test
cargo test --all

# Build
cargo build --release -p sqlrustgo-bench-cli

# Run benchmarks
./target/release/benchmark tpch --sf 0.1 --output /tmp/tpch.json
./target/release/benchmark oltp --sf 0.1 --output /tmp/oltp.json
./target/release/benchmark custom --sf 0.1 --output /tmp/custom.json

# Compare results
python scripts/compare_benchmarks.py benchmarks/baselines/sf01 /tmp
```

---

## Files Summary

| File | Action |
|------|--------|
| `.github/workflows/bench-pr.yml` | Create |
| `.github/workflows/bench-schedule.yml` | Create |
| `scripts/compare_benchmarks.py` | Create |
| `benchmarks/baselines/sf01/tpch.json` | Create |
| `benchmarks/baselines/sf01/oltp.json` | Create |
| `benchmarks/baselines/sf01/custom.json` | Create |
| `benchmarks/baselines/sf01/.gitkeep` | Create |

---

## Estimated Time

- Task 1: 5 min (workflow creation)
- Task 2: 5 min (workflow creation)
- Task 3: 10 min (comparison script)
- Task 4: 5 min (baseline data)
- Task 5: 2 min (CI update)
- Task 6: 5 min (verification)

**Total: ~32 minutes**

---

> **Plan complete.** Two execution options:
> 1. **Subagent-Driven (this session)** - I dispatch fresh subagent per task
> 2. **Parallel Session** - Open new session with executing-plans