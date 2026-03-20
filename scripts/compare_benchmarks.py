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


def get_metric(metrics, key):
    """Get metric value, treating null as 0."""
    val = metrics.get(key)
    return 0 if val is None else val


def compare_metric(name, baseline_val, current_val):
    # Handle None/null values - treat null as 0
    if baseline_val is None:
        baseline_val = 0
    if current_val is None:
        current_val = 0

    if baseline_val == 0:
        if current_val == 0:
            return True, 0.0
        return False, float("inf")

    change = (current_val - baseline_val) / baseline_val

    # For QPS, higher is better - fail if decreased
    if "qps" in name.lower():
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
            get_metric(baseline.get("metrics", {}), "qps"),
            get_metric(current.get("metrics", {}), "qps"),
        )
        status = "PASS" if passed else "FAIL"
        print(f"{result_file.name} QPS: {change:+.1f}% [{status}]")
        if not passed:
            all_passed = False
            failures.append(f"{result_file.name} QPS: {change:+.1f}%")

        # Compare P50
        passed, change = compare_metric(
            "P50",
            get_metric(baseline.get("metrics", {}), "p50_ms"),
            get_metric(current.get("metrics", {}), "p50_ms"),
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
