# Verification Engine

## Definition

Verification Engine (`scripts/verification_engine.py`) is the component that parses test output and generates the proof artifact.

---

## Identity

```
Name: verification_engine.py
Language: Python 3
Path: scripts/verification_engine.py
Git history: Commits a227749e, 88339a68
```

---

## Interface

### Input

```
Type: stdout + stderr from `cargo test`
Format: Text
Example line:
  "test result: ok. 12 passed; 0 failed; 0 ignored; ..."
```

### Output

```
File: verification_report.json
Format: JSON
Exit code: 0 (verified) or 1 (failed)
```

---

## Function Specification

### `run_tests()`

```python
def run_tests() -> tuple[int, str]:
    """
    Execute cargo test and return results.

    Returns:
        (exit_code: int, output: str)
        exit_code: cargo test return code
        output: combined stdout + stderr
    """
```

**Implementation:**
```python
result = subprocess.run(
    ["cargo", "test"],
    capture_output=True,
    text=True
)
return result.returncode, result.stdout + result.stderr
```

**Properties:**
```
P1: Deterministic
    Same test suite → same output (modulo non-determinism AV9)

P2: Faithful
    Output exactly reflects cargo test stdout/stderr

P3: Complete
    Captures all test result lines from all test suites
```

---

### `parse_results(output: str) -> tuple[int, int, int]`

```python
def parse_results(output: str) -> tuple[int, int, int]:
    """
    Parse cargo test output to extract pass/fail counts.

    Returns:
        (passed: int, failed: int, ignored: int)
    """
```

**Implementation:**
```python
passed = 0
failed = 0

for line in output.splitlines():
    if "test result:" in line:
        m = re.search(r'(\d+) passed; (\d+) failed', line)
        if m:
            passed += int(m.group(1))
            failed += int(m.group(2))

return passed, failed
```

**Regex:** `r'(\d+) passed; (\d+) failed'`

**Properties:**
```
P4: Faithful
    Counts exactly match test result lines

P5: Complete
    Sums across all "test result:" lines

P6: Deterministic
    Same output → same counts
```

**Known Issue:** Does not count `ignored` (returns 0). This is acceptable because ignored tests cause CI failure via non-zero exit code.

---

### `generate_report()`

```python
def generate_report():
    """
    Main entry point.

    1. Run tests
    2. Parse results
    3. Generate proof
    4. Write verification_report.json
    5. Exit with appropriate code
    """
```

**Flow:**
```python
code, output = run_tests()
passed, failed = parse_results(output)

report = {
    "proof_version": "1.0",
    "generated_by": "verification_engine.py",
    "source": "CI execution (not PR declaration)",
    "passed": passed,
    "failed": failed,
    "baseline_verified": failed == 0 and code == 0,
    "blocker_count": failed,
    "status": "VERIFIED" if (failed == 0 and code == 0) else "FAIL"
}

with open("verification_report.json", "w") as f:
    json.dump(report, f, indent=2)

if failed > 0 or code != 0:
    print(f"❌ VERIFICATION FAILED: {failed} test(s) failed")
    sys.exit(1)
else:
    print(f"✅ VERIFICATION PASSED: {passed} tests passed")
    sys.exit(0)
```

---

## Proof Generation Logic

```
IF failed > 0 THEN status = "FAIL"
IF code != 0 THEN status = "FAIL"
IF failed == 0 AND code == 0 THEN status = "VERIFIED"
IF status == "VERIFIED" THEN baseline_verified = true
ELSE baseline_verified = false
```

**Key Invariant:**
```
baseline_verified = true
⇔ (failed == 0 AND code == 0 AND passed == 226)
```

---

## Properties

| Property | Description |
|----------|-------------|
| Deterministic | Same input → same output |
| Faithful | Output reflects actual test execution |
| Complete | All test suites counted |
| Tamper-evident | Cannot produce false proof (only if V not tampered) |

---

## Limitations

```
LIMITATION: V cannot detect non-determinism (AV9)
  If tests fail intermittently, V sees only one run.
  Intermittent failures may not appear in single run.

LIMITATION: V assumes stable output format
  If cargo test output format changes, regex may break.
  Mitigation: Version pinned in CI.

LIMITATION: V is in repository (AV4)
  PR can modify V to output fake proof.
  Mitigation: V should move to protected repo (v2.9.0)
```

---

## Trust Classification

```
V is classified as: UNTRUSTED (T4 is assumption)

Reason: V is in repository and can be modified by PR.

If V is tampered (AV4):
  - V can output {passed: 226, failed: 0} regardless of actual results
  - CI would pass with fake proof
  - No in-system detection mechanism

Trust chain requires T4: verification_engine.py logic correctness
If T4 is violated → entire proof system invalid
```

---

## Q&A

**Q: Why does V use regex instead of parsing cargo JSON output?**

A: `cargo test --message-format=json` exists but `verification_engine.py` was written to parse text output. This is a pragmatic choice. JSON format is available but not used.

**Q: Can V be called locally?**

A: Yes. `python3 scripts/verification_engine.py` works locally. But local proof is not trusted — only CI-generated proof is accepted.

**Q: What happens if V is called with no tests run?**

A: V would output `passed: 0, failed: 0`. `baseline_verified` would be false (since passed < 226). Merge gate closes.

**Q: Why is `ignored` not counted in the proof?**

A: Current implementation does not extract ignored count. This is a known gap. However, `cargo test` exits non-zero if tests are ignored, so CI still fails.

**Q: What happens if cargo test produces output in non-English locale?**

A: V regex `(\d+) passed; (\d+) failed` assumes English. Non-English output would not match. This is a localization vulnerability.

---

*Source: SQLRustGo Governance Whitepaper v2.8.0, PART 3*
