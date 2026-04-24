#!/usr/bin/env python3
"""
Self-Audit Engine v2.8.1 — Independent Verification Layer

Design principle:
  Self-Audit does NOT import or reuse verification_engine.py.
  It implements its own independent parsing and checking paths.
  This creates a SECOND TRUTH PATH to detect AV4 (V tampering).

Two-path architecture:
  Path A (CI):    cargo test → verification_engine.py → verification_report.json
  Path B (Audit): cargo test → audit.parser (independent) → audit.checks

If Path A ≠ Path B → system compromised (AV4 detected)
"""

import json
import sys
import os
import argparse
from datetime import datetime, timezone
from pathlib import Path

from audit.runner import run_cargo_test, run_multiple
from audit.parser import parse_results_independent, TestCounts
from audit.checks import (
    BASELINE_PASSED, BASELINE_FAILED, BASELINE_IGNORED,
    CheckResult,
    check_r1_test_immutability,
    check_r2_ignore_injection,
    check_r3_proof_provenance,
    check_r4_full_execution,
    check_r5_baseline_verified,
    check_r6_test_count,
    check_r6_strict,
    check_av2_partial_execution,
    check_av6_ignore_bypass,
    check_av9_determinism,
    check_proof_vs_reality,
)

CONTRACT_PATH = Path("contract/v2.8.0.json")
PROOF_PATH = Path("verification_report.json")
AUDIT_VERSION = "2.8.1"

STATUS_SYMBOLS = {
    "TRUSTED": "🟢",
    "WEAKENED": "🟡",
    "COMPROMISED": "🔴",
}


class AuditReportBuilder:
    def __init__(self):
        self.checks = []
        self.critical_failures = []
        self.warnings = []
        self.info = []
        self.audit_time = datetime.now(timezone.utc).isoformat()

    def add(self, result):
        self.checks.append(result)
        if not result.passed:
            if result.severity == "CRITICAL":
                self.critical_failures.append(f"[{result.rule}] {result.message}")
            else:
                self.warnings.append(f"[{result.rule}] {result.message}")
        else:
            self.info.append(f"[{result.rule}] {result.message}")

    def compute_status(self):
        if self.critical_failures:
            return "COMPROMISED"
        if self.warnings:
            return "WEAKENED"
        return "TRUSTED"

    def build(self):
        status = self.compute_status()
        return {
            "audit_version": AUDIT_VERSION,
            "audit_time": self.audit_time,
            "contract_version": "2.8.0",
            "system_status": status,
            "status_symbol": STATUS_SYMBOLS.get(status, "❓"),
            "critical_failures": self.critical_failures,
            "warnings": self.warnings,
            "info": self.info,
            "checks": [
                {
                    "rule": c.rule,
                    "passed": c.passed,
                    "message": c.message,
                    "severity": c.severity,
                    "evidence": c.evidence,
                }
                for c in self.checks
            ],
        }


def main():
    parser = argparse.ArgumentParser(description="SQLRustGo Self-Audit Engine v2.8.1")
    parser.add_argument("--runs", type=int, default=1,
                        help="Number of runs for determinism check (default: 1, use 3 for AV9)")
    parser.add_argument("--output", default="audit_report.json",
                        help="Output path")
    parser.add_argument("--repo", default=".",
                        help="Repository root")
    args = parser.parse_args()
    repo_root = Path(args.repo).resolve()
    os.chdir(repo_root)

    print("🔍 Self-Audit v2.8.1 — Independent Verification")
    print(f"   Contract: {CONTRACT_PATH}")
    print(f"   Proof: {PROOF_PATH}")
    print(f"   Runs: {args.runs}x")
    print()

    # STEP 1: Load contract
    print("📋 [1/7] Loading contract...")
    if not CONTRACT_PATH.exists():
        print(f"   ❌ Contract not found: {CONTRACT_PATH}")
        sys.exit(1)
    with open(CONTRACT_PATH) as f:
        contract = json.load(f)
    print(f"   ✅ Contract loaded: {contract.get('contract_version')}")

    # STEP 2: Load proof
    print("📋 [2/7] Loading proof artifact...")
    if not PROOF_PATH.exists():
        print("   ⚠️  Proof not found — running without proof comparison")
        proof = None
    else:
        with open(PROOF_PATH) as f:
            proof = json.load(f)
        print(f"   ✅ Proof: passed={proof.get('passed')}, "
              f"failed={proof.get('failed')}, "
              f"baseline_verified={proof.get('baseline_verified')}")

    # STEP 3: Run tests independently
    print(f"📋 [3/7] Running cargo test independently ({args.runs}x)...")
    runs_raw = run_multiple(n=args.runs)
    all_counts = []
    for i, (code, output) in enumerate(runs_raw):
        counts = parse_results_independent(output)
        all_counts.append(counts)
        print(f"   Run {i+1}: passed={counts.passed}, "
              f"failed={counts.failed}, "
              f"ignored={counts.ignored}, "
              f"parse_error={counts.parse_error}")

    reality = all_counts[0]

    # STEP 4: Governance checks
    print("📋 [4/7] Running governance checks (R1-R6)...")
    builder = AuditReportBuilder()

    r1 = check_r1_test_immutability(repo_root=repo_root)
    print(f"   R1: {'✅' if r1.passed else '❌'} {r1.message}")
    builder.add(r1)

    r2 = check_r2_ignore_injection(repo_root=repo_root)
    print(f"   R2: {'✅' if r2.passed else '❌'} {r2.message}")
    builder.add(r2)

    if proof:
        r3 = check_r3_proof_provenance(proof)
        print(f"   R3: {'✅' if r3.passed else '❌'} {r3.message}")
        builder.add(r3)

        r5 = check_r5_baseline_verified(proof)
        print(f"   R5: {'✅' if r5.passed else '❌'} {r5.message}")
        builder.add(r5)

    # STEP 5: Proof vs Reality (CORE — detects AV4)
    print("📋 [5/7] Comparing proof vs independent reality...")
    if proof and not reality.parse_error:
        pvr = check_proof_vs_reality(proof, reality)
        print(f"   PROOFvsREALITY: {'✅' if pvr.passed else '❌'} {pvr.message}")
        builder.add(pvr)
    else:
        print("   ⚠️  Skipped — no proof or parse error")

    # STEP 6: Attack vector detection
    print("📋 [6/7] Running attack vector detection...")

    if not reality.parse_error:
        r4 = check_r4_full_execution(reality.passed, reality.failed, reality.ignored)
        print(f"   R4: {'✅' if r4.passed else '❌'} {r4.message}")
        builder.add(r4)

        r6 = check_r6_test_count(reality.passed)
        print(f"   R6: {'✅' if r6.passed else '❌'} {r6.message}")
        builder.add(r6)

        r6s = check_r6_strict(reality.passed)
        print(f"   R6_STRICT: {'✅' if r6s.passed else '❌'} {r6s.message}")
        builder.add(r6s)

        av2 = check_av2_partial_execution(reality.passed, reality.failed, reality.ignored)
        print(f"   AV2: {'✅' if av2.passed else '❌'} {av2.message}")
        builder.add(av2)
    else:
        print("   ⚠️  Skipped — parse error")

    av6, _ = check_av6_ignore_bypass(repo_root=repo_root)
    print(f"   AV6: ℹ️  {av6.message}")
    builder.add(av6)

    av9 = check_av9_determinism(all_counts)
    print(f"   AV9: {'✅' if av9.passed else '❌'} {av9.message}")
    builder.add(av9)

    # STEP 7: Build report
    print("📋 [7/7] Building audit report...")

    report = builder.build()
    output_path = Path(args.output)
    with open(output_path, "w") as f:
        json.dump(report, f, indent=2)

    status = report["system_status"]
    symbol = report["status_symbol"]

    print()
    print(f"{symbol} SYSTEM STATUS: {status}")
    print(f"   Report: {output_path}")

    if report["critical_failures"]:
        print("   CRITICAL FAILURES:")
        for cf in report["critical_failures"]:
            print(f"     ❌ {cf}")

    if report["warnings"]:
        print("   WARNINGS:")
        for w in report["warnings"]:
            print(f"     ⚠️  {w}")

    shown = report["info"][:5]
    print(f"   PASSED ({len(report['info'])} total):")
    for i in shown:
        print(f"     ✅ {i}")
    if len(report["info"]) > 5:
        print(f"     ... +{len(report['info']) - 5} more")

    sys.exit(0 if status == "TRUSTED" else 1)


if __name__ == "__main__":
    main()
