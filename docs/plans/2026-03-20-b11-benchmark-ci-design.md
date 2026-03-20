# B-11 Benchmark CI Design

## Overview

Implement GitHub Actions CI for the new TPC-H/OLTP/Custom benchmark system created in EPIC-01.

## Goals

1. **Catch regressions early** - Run quick benchmarks on every PR
2. **Track trends** - Weekly full suite to detect gradual performance degradation
3. **No external dependencies** - Use SQLite for reproducible CI results

## Architecture

### Two Workflows

| Workflow | Trigger | Scale Factors | Time |
|----------|---------|---------------|------|
| `bench-pr.yml` | PR/push to `develop/**` | SF=0.1 | ~2-3 min |
| `bench-schedule.yml` | Weekly (Monday 00:00 UTC) | SF=0.1, SF=1, SF=10 | ~10-15 min |

### Benchmark Types

All three benchmark types run in both workflows:
- `tpch` - TPC-H Q1/Q3/Q6/Q10
- `oltp` - OLTP YCSB-like workload
- `custom` - Custom query workload

### Baseline Comparison

- Baselines stored in `benchmarks/baselines/` directory
- Compare current vs baseline metrics:
  - **QPS regression** > 20% → Fail
  - **P50 latency increase** > 20% → Fail
- PRs get comments with comparison results

## File Structure

```
.github/workflows/
  bench-pr.yml          # Quick CI (SF=0.1 only)
  bench-schedule.yml    # Weekly full suite

benchmarks/
  baselines/
    sf01/
      tpch.json
      oltp.json
      custom.json
    sf1/
      ...
    sf10/
      ...
```

## Implementation Details

### bench-pr.yml

```yaml
on:
  pull_request:
    branches: [develop/**]
  push:
    branches: [develop/**]

jobs:
  quick-benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build bench-cli
        run: cargo build --release -p sqlrustgo-bench-cli
      - name: Run SF=0.1 benchmarks
        run: |
          ./target/release/benchmark tpch --sf 0.1 --output benchmarks/results/sf01/tpch.json
          ./target/release/benchmark oltp --sf 0.1 --output benchmarks/results/sf01/oltp.json
          ./target/release/benchmark custom --sf 0.1 --output benchmarks/results/sf01/custom.json
      - name: Compare with baseline
        run: python scripts/compare_benchmarks.py benchmarks/baselines/sf01 benchmarks/results/sf01
      - name: Upload results
        uses: actions/upload-artifact@v4
```

### bench-schedule.yml

```yaml
on:
  schedule:
    - cron: '0 0 * * 1'  # Monday at midnight UTC

jobs:
  full-benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build bench-cli
        run: cargo build --release -p sqlrustgo-bench-cli
      - name: Run all benchmarks
        run: |
          for sf in 0.1 1 10; do
            ./target/release/benchmark tpch --sf $sf --output benchmarks/results/sf${sf}/tpch.json
            ./target/release/benchmark oltp --sf $sf --output benchmarks/results/sf${sf}/oltp.json
            ./target/release/benchmark custom --sf $sf --output benchmarks/results/sf${sf}/custom.json
          done
      - name: Update baselines (if needed)
        if: github.event_name == 'schedule'
        run: cp -r benchmarks/results/* benchmarks/baselines/
      - name: Upload results
        uses: actions/upload-artifact@v4
```

### Comparison Script

`scripts/compare_benchmarks.py`:
- Load baseline JSON
- Load current results JSON
- Calculate % change for QPS and P50
- Output pass/fail with details
- Exit 0 if all metrics within 20% of baseline

## Success Criteria

1. ✅ PRs to `develop/**` trigger quick benchmark (SF=0.1)
2. ✅ Full benchmark suite runs weekly (SF=0.1, 1, 10)
3. ✅ QPS regression > 20% causes PR failure
4. ✅ P50 latency increase > 20% causes PR failure
5. ✅ Results uploaded as artifacts
6. ✅ Weekly runs update baselines

## Rejected Alternatives

1. **Single workflow with conditionals** - Harder to debug, less clear
2. **PostgreSQL in CI** - External dependency slows CI, less reproducible
3. **No regression thresholds** - Defeats purpose of CI