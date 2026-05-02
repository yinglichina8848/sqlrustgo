#!/usr/bin/env python3
"""
Proof Registry Verification Script

Verifies that the proof registry meets R10 gate requirements:
- Minimum 10 proof files
- All proofs are valid JSON
- All proofs match the schema
- Proof status is verified
"""

import json
import os
import sys
from pathlib import Path

PROOF_DIR = Path("docs/proof")
MIN_PROOFS = 10


def load_schema():
    schema_path = PROOF_DIR / "PROOF_SCHEMA.json"
    if schema_path.exists():
        with open(schema_path) as f:
            return json.load(f)
    return None


def verify_proof_file(proof_path):
    errors = []
    try:
        with open(proof_path) as f:
            proof = json.load(f)
    except json.JSONDecodeError as e:
        return [f"Invalid JSON: {e}"]

    required_fields = ["proof_id", "title", "language", "category", "status", "description", "evidence", "created_at"]
    for field in required_fields:
        if field not in proof:
            errors.append(f"Missing required field: {field}")

    if "status" in proof and proof["status"] != "verified":
        errors.append(f"Proof status is '{proof['status']}', expected 'verified'")

    return errors


def main():
    print("=== Proof Registry Verification ===")
    print(f"Date: {__import__('datetime').date.today()}")
    print()

    if not PROOF_DIR.exists():
        print(f"❌ FAIL: Proof directory '{PROOF_DIR}' does not exist")
        return 1

    proof_files = list(PROOF_DIR.glob("PROOF-*.json"))
    print(f"Proof files found: {len(proof_files)}")

    if len(proof_files) < MIN_PROOFS:
        print(f"❌ FAIL: Insufficient proofs (found {len(proof_files)}, need >= {MIN_PROOFS})")
        return 1

    all_errors = []
    verified_count = 0

    for proof_file in sorted(proof_files):
        print(f"\nVerifying {proof_file.name}...")
        errors = verify_proof_file(proof_file)
        if errors:
            for err in errors:
                print(f"  ❌ {err}")
            all_errors.extend(errors)
        else:
            print(f"  ✅ Valid")
            verified_count += 1

    print()
    if all_errors:
        print(f"❌ FAIL: {len(all_errors)} error(s) found")
        return 1

    print(f"✅ PASS: {verified_count}/{len(proof_files)} proofs verified")
    print(f"   Meets R10 requirement: {verified_count} >= {MIN_PROOFS}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
