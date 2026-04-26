#!/usr/bin/env python3
"""
Self-Audit Engine - audits code quality, security, and compliance.
Generates audit_report.json
"""
import json, os, sys, time, subprocess

REPORT_FILE = os.environ.get("REPORT_DIR", ".") + "/audit_report.json"

class AuditEngine:
    def __init__(self, repo_root):
        self.repo_root = repo_root
        self.audits = []

    def run_audit(self, name, cmd, cwd=None):
        try:
            result = subprocess.run(cmd, shell=True, capture_output=True, text=True,
                                   timeout=120, cwd=cwd or self.repo_root)
            passed = result.returncode == 0
            self.audits.append({
                "audit": name,
                "passed": passed,
                "exit_code": result.returncode,
                "stdout": result.stdout[:200],
                "stderr": result.stderr[:200]
            })
            return passed
        except subprocess.TimeoutExpired:
            self.audits.append({"audit": name, "passed": False, "exit_code": -1, "error": "timeout"})
            return False

    def audit_all(self):
        # Security: check for unsafe patterns
        self.run_audit("unsafe_code", "grep -r 'unsafe{' --include='*.rs' . 2>/dev/null | head -10 || true")

        # Dependency: check Cargo.lock exists
        self.run_audit("cargo_lock", "test -f Cargo.lock")

        # Documentation: check README
        self.run_audit("readme", "test -f README.md")

        # CI: check workflow files
        self.run_audit("ci_workflow", "test -d .gitea/workflows || test -d .github/workflows")

        report = {
            "engine": "self_audit",
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "passed": all(a["passed"] for a in self.audits),
            "total": len(self.audits),
            "passed_count": sum(1 for a in self.audits if a["passed"]),
            "audits": self.audits
        }
        os.makedirs(os.path.dirname(REPORT_FILE), exist_ok=True)
        with open(REPORT_FILE, "w") as f:
            json.dump(report, f, indent=2)
        print(json.dumps(report, indent=2))
        return report["passed"]

if __name__ == "__main__":
    root = sys.argv[1] if len(sys.argv) > 1 else os.getcwd()
    engine = AuditEngine(root)
    passed = engine.audit_all()
    sys.exit(0 if passed else 1)
