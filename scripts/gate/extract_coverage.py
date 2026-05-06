#!/usr/bin/env python3
"""Extract coverage percentage from tarpaulin JSON report."""
import json
import sys

if len(sys.argv) < 2:
    print("Usage: extract_coverage.py <tarpaulin_report.json>")
    sys.exit(1)

report_path = sys.argv[1]
try:
    with open(report_path) as f:
        report = json.load(f)
    coverage = report.get("coverage", 0)
    print(f"Coverage: {coverage:.2f}%")
except Exception as e:
    print(f"Error: {e}")
    sys.exit(1)
