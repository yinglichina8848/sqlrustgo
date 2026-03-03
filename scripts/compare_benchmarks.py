#!/usr/bin/env python3
"""
Performance Regression Detection Script

Compares benchmark results between two baselines and detects performance regressions.
Uses criterion's JSON output format.

Usage:
    python3 scripts/compare_benchmarks.py --baseline1 <path> --baseline2 <path> --threshold <percentage>

Example:
    python3 scripts/compare_benchmarks.py --baseline1 baseline_2024_01_01.json --baseline2 current.json --threshold 10
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple


def load_benchmark_results(path: str) -> Dict:
    """Load benchmark results from JSON file."""
    with open(path, 'r') as f:
        return json.load(f)


def extract_metrics(results: Dict) -> Dict[str, float]:
    """Extract mean time in nanoseconds for each benchmark."""
    metrics = {}
    for benchmark in results.get('benchmarks', []):
        name = benchmark['full_name']
        # Get mean time in nanoseconds
        mean = benchmark['mean']['value']
        metrics[name] = mean
    return metrics


def compare_metrics(
    baseline1: Dict[str, float],
    baseline2: Dict[str, float],
    threshold_percent: float
) -> Tuple[List[Dict], List[Dict], List[Dict]]:
    """
    Compare two sets of benchmark metrics.

    Returns:
        - improved: benchmarks that improved (negative change)
        - regressed: benchmarks that regressed (positive change > threshold)
        - unchanged: benchmarks within threshold
    """
    improved = []
    regressed = []
    unchanged = []

    # Get union of all benchmark names
    all_names = set(baseline1.keys()) | set(baseline2.keys())

    for name in sorted(all_names):
        if name not in baseline1:
            print(f"Warning: Benchmark '{name}' not in first baseline, skipping")
            continue
        if name not in baseline2:
            print(f"Warning: Benchmark '{name}' not in second baseline, skipping")
            continue

        old_time = baseline1[name]
        new_time = baseline2[name]

        # Calculate percentage change
        if old_time > 0:
            change_percent = ((new_time - old_time) / old_time) * 100
        else:
            change_percent = 0

        entry = {
            'name': name,
            'old_time_ns': old_time,
            'new_time_ns': new_time,
            'change_percent': change_percent
        }

        if change_percent < -threshold_percent:
            improved.append(entry)
        elif change_percent > threshold_percent:
            regressed.append(entry)
        else:
            unchanged.append(entry)

    return improved, regressed, unchanged


def format_time(ns: float) -> str:
    """Format nanoseconds to human readable format."""
    if ns < 1000:
        return f"{ns:.2f}ns"
    elif ns < 1_000_000:
        return f"{ns/1000:.2f}µs"
    else:
        return f"{ns/1_000_000:.2f}ms"


def print_report(
    improved: List[Dict],
    regressed: List[Dict],
    unchanged: List[Dict],
    threshold: float
):
    """Print a formatted comparison report."""
    print("=" * 80)
    print("PERFORMANCE REGRESSION REPORT")
    print("=" * 80)
    print()

    print(f"Threshold: {threshold}%")
    print(f"Total benchmarks: {len(improved) + len(regressed) + len(unchanged)}")
    print()

    if regressed:
        print("-" * 80)
        print(f"REGRESSED ({len(regressed)} benchmarks)")
        print("-" * 80)
        for entry in regressed:
            print(f"  ❌ {entry['name']}")
            print(f"     {format_time(entry['old_time_ns'])} → {format_time(entry['new_time_ns'])}")
            print(f"     +{entry['change_percent']:.2f}%")
        print()

    if improved:
        print("-" * 80)
        print(f"IMPROVED ({len(improved)} benchmarks)")
        print("-" * 80)
        for entry in improved:
            print(f"  ✓ {entry['name']}")
            print(f"     {format_time(entry['old_time_ns'])} → {format_time(entry['new_time_ns'])}")
            print(f"     {entry['change_percent']:.2f}%")
        print()

    if unchanged:
        print("-" * 80)
        print(f"UNCHANGED ({len(unchanged)} benchmarks within {threshold}%)")
        print("-" * 80)
        for entry in unchanged:
            print(f"  • {entry['name']}")
            print(f"     {format_time(entry['old_time_ns'])} → {format_time(entry['new_time_ns'])}")
            print(f"     {entry['change_percent']:+.2f}%")
        print()

    # Summary
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print(f"  Regressed: {len(regressed)}")
    print(f"  Improved:  {len(improved)}")
    print(f"  Unchanged: {len(unchanged)}")
    print()

    if regressed:
        print("⚠️  WARNING: Performance regressions detected!")
        return 1
    else:
        print("✅ No performance regressions detected.")
        return 0


def main():
    parser = argparse.ArgumentParser(
        description='Compare benchmark results and detect performance regressions'
    )
    parser.add_argument(
        '--baseline1',
        required=True,
        help='Path to first baseline JSON (usually older)'
    )
    parser.add_argument(
        '--baseline2',
        required=True,
        help='Path to second baseline JSON (usually newer)'
    )
    parser.add_argument(
        '--threshold',
        type=float,
        default=10.0,
        help='Percentage threshold for regression detection (default: 10)'
    )
    parser.add_argument(
        '--output',
        help='Output file for the report (optional)'
    )

    args = parser.parse_args()

    try:
        results1 = load_benchmark_results(args.baseline1)
        results2 = load_benchmark_results(args.baseline2)
    except FileNotFoundError as e:
        print(f"Error: {e}")
        return 1
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in benchmark file: {e}")
        return 1

    metrics1 = extract_metrics(results1)
    metrics2 = extract_metrics(results2)

    improved, regressed, unchanged = compare_metrics(
        metrics1, metrics2, args.threshold
    )

    exit_code = print_report(improved, regressed, unchanged, args.threshold)

    # Optionally write to file
    if args.output:
        with open(args.output, 'w') as f:
            # Redirect stdout to file
            import io
            from contextlib import redirect_stdout

            with redirect_stdout(f):
                exit_code = print_report(
                    improved, regressed, unchanged, args.threshold
                )

    return exit_code


if __name__ == '__main__':
    sys.exit(main())
