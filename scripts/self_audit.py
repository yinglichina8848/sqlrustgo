#!/usr/bin/env python3
import json
import subprocess
import re
import sys
import os


def run_cargo_test():
    result = subprocess.run(
        ["cargo", "test", "--all-features"],
        capture_output=True,
        text=True,
        cwd=os.environ.get("CARGO_TESTS_DIR", ".")
    )
    return result.returncode, result.stdout + result.stderr


def parse_test_results(output):
    passed = 0
    failed = 0
    for line in output.splitlines():
        if "test result:" in line:
            m = re.search(r'(\d+) passed; (\d+) failed', line)
            if m:
                passed += int(m.group(1))
                failed += int(m.group(2))
    return passed, failed


def generate_audit_report(exit_code, output):
    passed, failed = parse_test_results(output)
    status = "TRUSTED" if exit_code == 0 and failed == 0 else "WEAKENED"

    report = {
        "audit_version": "1.0",
        "generated_by": "self_audit.py",
        "source": "CI execution",
        "passed": passed,
        "failed": failed,
        "exit_code": exit_code,
        "status": status
    }

    with open("audit_report.json", "w") as f:
        json.dump(report, f, indent=2)

    print("=" * 50)
    print("SELF AUDIT REPORT")
    print("=" * 50)
    print(json.dumps(report, indent=2))
    print("=" * 50)

    if status == "WEAKENED":
        print(f"AUDIT WEAKENED: {failed} test(s) failed")
        sys.exit(1)
    else:
        print(f"AUDIT TRUSTED: {passed} tests passed")
        sys.exit(0)


if __name__ == "__main__":
    exit_code, output = run_cargo_test()
    generate_audit_report(exit_code, output)
