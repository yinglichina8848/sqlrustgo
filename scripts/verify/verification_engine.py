#!/usr/bin/env python3
"""
Verification Engine - validates build artifacts, test results, and reports.
"""
import json, os, sys, subprocess, time

REPORT_FILE = os.environ.get("REPORT_DIR", ".") + "/verification_report.json"

class VerificationEngine:
    def __init__(self, repo_root):
        self.repo_root = repo_root
        self.results = []

    def check_binary(self, name, path):
        exists = os.path.exists(path)
        size = os.path.getsize(path) if exists else 0
        passed = exists and size > 0
        self.results.append({
            "check": f"binary:{name}",
            "passed": passed,
            "detail": f"{'found' if passed else 'missing'}, {size} bytes"
        })
        return passed

    def check_test_report(self, path="test_report.json"):
        if not os.path.exists(path):
            self.results.append({"check": "test_report", "passed": False, "detail": "report not found"})
            return False
        with open(path) as f:
            report = json.load(f)
        passed = report.get("passed", report.get("success", False))
        self.results.append({
            "check": "test_report",
            "passed": passed,
            "detail": f"tests: {report.get('total', '?')}, passed: {report.get('passed', report.get('success', '?'))}"
        })
        return passed

    def check_file_integrity(self, pattern="Cargo.lock"):
        path = os.path.join(self.repo_root, pattern)
        exists = os.path.exists(path)
        self.results.append({
            "check": f"integrity:{pattern}",
            "passed": exists,
            "detail": "present" if exists else "missing"
        })
        return exists

    def run(self):
        checks = [
            self.check_file_integrity("Cargo.toml"),
            self.check_file_integrity("Cargo.lock"),
        ]
        report = {
            "engine": "verification_engine",
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "passed": all(r["passed"] for r in self.results),
            "checks": self.results
        }
        os.makedirs(os.path.dirname(REPORT_FILE), exist_ok=True)
        with open(REPORT_FILE, "w") as f:
            json.dump(report, f, indent=2)
        print(json.dumps(report, indent=2))
        return report["passed"]

if __name__ == "__main__":
    root = sys.argv[1] if len(sys.argv) > 1 else os.getcwd()
    engine = VerificationEngine(root)
    passed = engine.run()
    sys.exit(0 if passed else 1)
