# Attack Model

## Definition

Attack Surface is the set of all vectors by which a malicious or negligent PR author could bypass governance rules or subvert the proof system.

---

## Attack Vector Inventory

### AV1: Test Immutability Bypass via Macro

```
VECTOR: Test modification through procedural macros

METHOD:
  Define #[test] in src/ procedural macro.
  Macro expands at compile time to generate test code.
  PR modifies src/ (not tests/).
  R1 check passes (checks only tests/ directory).
  Generated code modifies test behavior.

EXAMPLE:
  // In src/my_macro.rs
  #[test_macro]
  fn critical_test() { ... }
  // Expands to tests/ at compile time

CURRENT DEFENSE: None

SEVERITY: HIGH
LIKELIHOOD: LOW (requires intentional macro abuse)

REMEDIATION:
  R1 must check generated code, not just source.
  v2.9.0: Add build artifact diff check
```

---

### AV2: Partial Test Execution

```
VECTOR: Running fewer tests than the full suite

METHOD:
  Modify CI to run: cargo test --skip some_failing_test
  Result: fewer tests run, fewer failures visible
  Proof shows passed=226 but some were skipped

EXAMPLE:
  # In .github/workflows/ci.yml
  - run: cargo test --skip flaky_test

CURRENT DEFENSE: None

SEVERITY: HIGH
LIKELIHOOD: MEDIUM (easy to do subtly)

REMEDIATION:
  V must verify test count matches expected (226)
  v2.9.0: Add --all-features flag enforcement
```

---

### AV3: CI Workflow Disablement

```
VECTOR: Delete CI to eliminate enforcement

METHOD:
  PR deletes .github/workflows/ci.yml
  No CI runs
  No enforcement
  PR merges directly

EXAMPLE:
  git rm .github/workflows/ci.yml
  # PR description: "clean up unused CI"

CURRENT DEFENSE: Branch protection (T5)

SEVERITY: CRITICAL
LIKELIHOOD: LOW (caught by branch protection)

REMEDIATION:
  Branch protection required (not optional)
  .github/ directory protected from deletion
```

---

### AV4: Verification Engine Tampering

```
VECTOR: Modify verification_engine.py to output fake proof

METHOD:
  PR modifies V to always output:
  {passed: 226, failed: 0, baseline_verified: true}
  regardless of actual test results
  CI runs with tampered V
  Fake proof passes validation

EXAMPLE:
  # verification_engine.py (tampered)
  def parse_results(output):
      return (226, 0, 0)  # Always return perfect results

CURRENT DEFENSE: R3 (V is regenerated from source)

NOTE:
  V is in repository, not immutable.
  Attack succeeds if reviewer doesn't notice V modification.

SEVERITY: CRITICAL
LIKELIHOOD: MEDIUM

REMEDIATION:
  V should be in separate, protected repository
  v2.9.0: V moves to dedicated infrastructure repo
```

---

### AV5: Semantic Regression

```
VECTOR: Change code behavior without changing test assertions

METHOD:
  Change query processing logic (JOIN ordering, GROUP BY evaluation)
  Test assertions match new (wrong) behavior
  Tests pass
  But query results are semantically incorrect

EXAMPLE:
  Original: SELECT a, b FROM t1 JOIN t2 ON t1.id = t2.id
  Changed:  Results same but order different (SQL permits this)
  Test:     assert_eq!(results, expected) — passes
  Reality:  ORDER BY missing — semantics wrong

CURRENT DEFENSE: None (by design)

NOTE:
  This is the FUNDAMENTAL LIMITATION of behavior-based testing.
  Tests verify WHAT code does, not IF code is correct.

SEVERITY: CRITICAL (by design)
LIKELIHOOD: HIGH

REMEDIATION:
  Requires formal verification (Phase 3)
  v2.9.0: GBrain semantic engine
```

---

### AV6: Ignore Semantic Bypass

```
VECTOR: Use different #[ignore] syntax to evade pattern match

METHOD:
  R2 regex: "^+.*#\[ignore\]"
  Replace:
    #[ignore]  →  #[ignore = "temporary fix pending"]
  R2 pattern does not match
  Test is ignored
  R2 check passes

EXAMPLE:
  // Before (R2 detects):
  #[ignore]

  // After (R2 misses):
  #[ignore = "known issue #1234"]

CURRENT DEFENSE: R2 regex may not catch all variants

SEVERITY: MEDIUM
LIKELIHOOD: MEDIUM

REMEDIATION:
  Use AST parsing to detect any #[ignore] attribute
  v2.9.0: Implement AST-level ignore detection
```

---

### AV7: Baseline Poisoning

```
VECTOR: Modify baseline to lower standards

METHOD:
  Use admin privileges to modify v2.8.0 baseline commit
  Change: baseline.passed = 226 → baseline.passed = 100
  Future PRs only need to pass 100 tests
  Coverage dramatically reduced

EXAMPLE:
  # Force push to modify baseline
  git push --force origin v2.8.0:refs/tags/v2.8.0

CURRENT DEFENSE: None (relies on T5, T6)

SEVERITY: HIGH
LIKELIHOOD: LOW (requires admin access)

REMEDIATION:
  Baseline must be in separate, protected artifact store
  v2.9.0: Baseline stored in immutable external store
```

---

### AV8: Proof Replay Attack

```
VECTOR: Copy proof from successful CI run

METHOD:
  Copy verification_report.json from a prior successful CI run
  Include in PR
  CI sees proof
  Reality: CI regenerates proof, overwrites PR's proof

CURRENT DEFENSE: R3 — CI always regenerates P

NOTE:
  Attack fails because CI overwrites, not accepts.
  This attack vector is MITIGATED.

SEVERITY: LOW (if CI correctly implemented)
LIKELIHOOD: LOW

STATUS: FULLY MITIGATED
```

---

### AV9: Non-Deterministic Test Masking

```
VECTOR: Exploit intermittent test failures

METHOD:
  Test has race condition
  Fails 10% of the time
  Developer runs locally 5 times — all pass
  Submits PR
  CI runs once — passes
  PR merges
  Production: Test fails 10% of the time

EXAMPLE:
  #[test]
  fn test_concurrent_insert() {
      // Race: depends on thread scheduling
      let handle = thread::spawn(...);
      let result = handle.join().unwrap();
      assert_eq!(result, expected);  // Sometimes fails
  }

CURRENT DEFENSE: None

SEVERITY: MEDIUM
LIKELIHOOD: MEDIUM

REMEDIATION:
  CI must run tests N times (N-runs strategy)
  v2.9.0: Add --test-threads=1 for deterministic execution
```

---

## Attack Surface Summary

```
┌──────────────────────────────────────────────────────────────┐
│ TOTAL: 9 Attack Vectors                                      │
├─────────┬──────────────────┬─────────┬────────────────────┤
│ AV      │ Name             │ Severity│ Status              │
├─────────┼──────────────────┼─────────┼────────────────────┤
│ AV1     │ Macro Bypass     │ HIGH    │ Not Mitigated       │
│ AV2     │ Partial Exec     │ HIGH    │ Not Mitigated       │
│ AV3     │ CI Disable       │ CRITICAL│ Mitigated (T5)      │
│ AV4     │ V Tampering      │ CRITICAL│ Partially Mitigated │
│ AV5     │ Semantic Regress │ CRITICAL│ Not Mitigated (by design) │
│ AV6     │ Ignore Bypass    │ MEDIUM  │ Partially Mitigated │
│ AV7     │ Baseline Poison  │ HIGH    │ Not Mitigated       │
│ AV8     │ Proof Replay     │ LOW     │ Fully Mitigated     │
│ AV9     │ Non-Det Masking  │ MEDIUM  │ Not Mitigated       │
└─────────┴──────────────────┴─────────┴────────────────────┘
```

---

## Q&A

**Q: Which attack is the most dangerous?**

A: AV5 (Semantic Regression). It cannot be detected by behavior-based testing. The code can be semantically wrong while all tests pass. This is a fundamental limitation.

**Q: Which attack is most likely to occur?**

A: AV5 and AV2. AV2 (partial execution) is easy to do subtly. AV5 (semantic regression) is often unintentional — developer changes logic, tests pass, but semantics change in subtle ways.

**Q: Why is AV3 (CI Disable) considered mitigated when defense is just branch protection?**

A: Because CI disable requires deleting `.github/workflows/ci.yml`. Branch protection can be configured to block deletion of this file. However, this is a GitHub-level control, not a governance system control.

**Q: Can AV4 (Verification Engine Tampering) be detected by CI?**

A: No. CI runs with the V that is in the repository. If V is tampered, CI runs tampered V and produces fake proof. Detection requires V to be in a separate, integrity-checked location.

**Q: What is the relationship between AV5 and the Trust Boundary?**

A: AV5 exploits the fact that tests only verify behavior, not semantics. This is explicitly outside the trust boundary — the system trusts that tests are correct, but does not verify that tests cover semantic correctness.

**Q: Why is AV8 (Proof Replay) considered fully mitigated?**

A: Because R3 ensures CI always regenerates `verification_report.json`. PR cannot submit proof. The attack requires CI to accept PR-submitted proof, but CI overwrites it.

---

*Source: SQLRustGo Governance Whitepaper v2.8.0, PART 5*
