#!/usr/bin/env python3
"""Generate per-crate coverage report from cargo-llvm-cov JSON output."""

import json
import subprocess
import sys
from pathlib import Path

def get_coverage_data():
    """Run cargo llvm-cov and parse output."""
    result = subprocess.run(
        ["cargo", "llvm-cov", "report", "--json", "--output-path", "/tmp/cov.json"],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print(f"Error: {result.stderr}", file=sys.stderr)
        sys.exit(1)
    
    with open("/tmp/cov.json") as f:
        return json.load(f)

def format_pct(covered, total):
    """Format percentage with color."""
    if total == 0:
        return "N/A"
    pct = (covered / total) * 100
    if pct >= 80:
        color = "\033[92m"   # green
    elif pct >= 50:
        color = "\033[93m"   # yellow
    else:
        color = "\033[91m"    # red
    reset = "\033[0m"
    return f"{color}{pct:5.1f}%{reset}"

def main():
    print("=" * 80)
    print("SQLRustGo Coverage Report")
    print("=" * 80)
    print()
    
    data = get_coverage_data()
    
    # Group by crate
    crates = {}
    for file in data.get("files", []):
        path = file["name"]
        # Extract crate name from path like "crates/executor/src/foo.rs"
        parts = Path(path).parts
        if "crates" in parts:
            idx = parts.index("crates")
            if idx + 1 < len(parts):
                crate = parts[idx + 1]
                if crate not in crates:
                    crates[crate] = {"covered": 0, "total": 0}
                crates[crate]["covered"] += file["summary"]["lines"]["covered"]
                crates[crate]["total"] += file["summary"]["lines"]["total"]
    
    # Sort by coverage percentage
    sorted_crates = sorted(
        crates.items(),
        key=lambda x: x[1]["covered"] / max(x[1]["total"], 1)
    )
    
    print(f"{'Crate':<30} {'Lines Covered':>20} {'Coverage':>10}")
    print("-" * 80)
    
    for crate, stats in sorted_crates:
        pct_str = format_pct(stats["covered"], stats["total"])
        print(f"{crate:<30} {stats['covered']:>6}/{stats['total']:<13} {pct_str:>10}")
    
    # Overall
    total_covered = sum(s["covered"] for s in crates.values())
    total_total = sum(s["total"] for s in crates.values())
    print("-" * 80)
    overall = format_pct(total_covered, total_total)
    print(f"{'TOTAL':<30} {total_covered:>6}/{total_total:<13} {overall:>10}")
    print()

if __name__ == "__main__":
    main()
