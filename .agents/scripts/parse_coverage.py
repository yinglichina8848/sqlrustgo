#!/usr/bin/env python3
import json
import sys

def parse_coverage_report(json_file):
    with open(json_file, 'r') as f:
        data = json.load(f)

    coverage = data.get('coverage', 0)
    line_coverage = data.get('line_coverage', 0)

    print(f"Total Coverage: {coverage}%")
    print(f"Line Coverage: {line_coverage}%")
    print()

    if 'files' in data:
        print("Per-file coverage:")
        for f in data.get('files', []):
            name = f.get('name', 'unknown')
            rate = f.get('line_rate', 0) * 100
            print(f"  {name}: {rate:.1f}%")

    return coverage, line_coverage

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: parse_coverage.py <coverage.json>")
        sys.exit(1)

    json_file = sys.argv[1]
    try:
        coverage, line_coverage = parse_coverage_report(json_file)
        threshold = 80

        if coverage < threshold:
            print(f"\n❌ Coverage {coverage}% below threshold {threshold}%")
            sys.exit(1)
        else:
            print(f"\n✅ Coverage {coverage}% meets threshold {threshold}%")
            sys.exit(0)
    except FileNotFoundError:
        print(f"Error: File {json_file} not found")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in {json_file}: {e}")
        sys.exit(1)
