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


def load_verification_report():
    try:
        with open("verification_report.json") as f:
            return json.load(f)
    except FileNotFoundError:
        return None


def generate_audit_report(exit_code, output):
    audit_passed, audit_failed = parse_test_results(output)
    verification = load_verification_report()
    proof_match = False

    if verification and verification.get("passed") == audit_passed:
        proof_match = True

    status = "TRUSTED" if exit_code == 0 and audit_failed == 0 and proof_match else "WEAKENED"

    report = {
        "audit_version": "1.0",
        "generated_by": "self_audit.py",
        "source": "CI execution (dual-path audit)",
        "passed": audit_passed,
        "failed": audit_failed,
        "exit_code": exit_code,
        "status": status,
        "verification_passed": verification.get("passed") if verification else None,
        "proof_match": proof_match
    }

    with open("audit_report.json", "w") as f:
        json.dump(report, f, indent=2)

    print("=" * 50)
    print("SELF AUDIT REPORT")
    print("=" * 50)
    print(json.dumps(report, indent=2))
    print("=" * 50)

    if status == "WEAKENED":
        print(f"AUDIT WEAKENED: proof_match={proof_match}")
        sys.exit(1)
    else:
        print(f"AUDIT TRUSTED: {audit_passed} tests passed, proof_match={proof_match}")
        sys.exit(0)


if __name__ == "__main__":
    exit_code, output = run_cargo_test()
    generate_audit_report(exit_code, output)
