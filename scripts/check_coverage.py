#!/usr/bin/env python3
import json
import sys
from pathlib import Path
from collections import defaultdict

def extract_low_coverage(cov_file="coverage.json", threshold=80):
    with open(cov_file) as f:
        data = json.load(f)

    file_stats = defaultdict(lambda: {"low": 0, "total": 0, "zero_funcs": []})

    for entry in data.get("data", []):
        for file_data in entry.get("files", []):
            filename = file_data.get("filename", "unknown")
            if "llvm-cov-target" in filename or "rustc" in filename:
                continue

            for func in file_data.get("functions", []):
                name = func.get("name", "unnamed")
                if name.startswith("_RN") or name.startswith("_RNC") or name.startswith("_RIN"):
                    continue
                percent = func.get("percent_covered", 0.0)
                file_stats[filename]["total"] += 1

                if percent < threshold:
                    file_stats[filename]["low"] += 1
                    if percent == 0:
                        file_stats[filename]["zero_funcs"].append(name)

    return file_stats

def main():
    threshold = int(sys.argv[1]) if len(sys.argv) > 1 else 80
    cov_file = sys.argv[2] if len(sys.argv) > 2 else "coverage.json"

    if not Path(cov_file).exists():
        print(f"Error: {cov_file} not found")
        print("Run: cargo llvm-cov --workspace --json > coverage.json")
        sys.exit(1)

    stats = extract_low_coverage(cov_file, threshold)
    sorted_files = sorted(stats.items(), key=lambda x: x[1]["low"], reverse=True)

    print(f"Files with low coverage (< {threshold}%)")
    print("=" * 70)

    for filename, data in sorted_files[:25]:
        if data["low"] == 0:
            continue
        coverage = int(100 - data["low"] / data["total"] * 100) if data["total"] > 0 else 0
        short_name = "/".join(filename.split("/")[-2:])
        print(f"{short_name:<40} {data['low']:3}/{data['total']:3} low ({coverage:2}% cov)")
        if data["zero_funcs"]:
            for fn in data["zero_funcs"][:3]:
                print(f"    0% {fn}")

    print()
    print(f"Summary: {len(stats)} files analyzed, {sum(1 for f in stats.values() if f['low'] > 0)} need coverage")

if __name__ == "__main__":
    main()
