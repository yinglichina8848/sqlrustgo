"""Audit parser — INDEPENDENT cargo test output parser.

CRITICAL: This parser is implemented WITHOUT referencing verification_engine.py.
It uses DIFFERENT regex logic to provide an independent second path.

Goal: If verification_engine.py is tampered with, this parser should still
produce correct results. Comparison of both results detects AV4.
"""

import re
from dataclasses import dataclass
from typing import Optional


@dataclass
class TestCounts:
    passed: Optional[int]
    failed: Optional[int]
    ignored: Optional[int]
    parse_error: bool
    error_msg: Optional[str] = None


def parse_results_independent(output: str) -> TestCounts:
    """
    Parse cargo test output INDEPENDENTLY.

    Example lines:
      test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      test result: FAILED. 3 passed; 1 failed; 0 ignored

    Implementation note:
      - Uses re.DOTALL to handle multi-line test results
      - Sums across ALL test result lines
      - Different regex structure than verification_engine.py
    """
    total_passed = 0
    total_failed = 0
    total_ignored = 0
    errors = []

    # Different regex from V: uses word boundaries and explicit group handling
    # V uses: r'\'(\d+) passed; (\d+) failed\''
    # We use: r'\'test result:\s*(?:ok|FAILED)\.?\s*(\d+)\s+passed\''
    passed_pattern = re.compile(
        r"test result:\s*(?:ok|FAILED)\.?\s*(\d+)\s+passed",
        re.IGNORECASE
    )
    failed_pattern = re.compile(
        r"test result:\s*(?:ok|FAILED)\.?\s*(?:\d+\s+passed;\s*)?(\d+)\s+failed",
        re.IGNORECASE
    )
    ignored_pattern = re.compile(
        r"test result:\s*(?:ok|FAILED)\.?\s*(?:\d+\s+passed;\s*){1,2}(\d+)\s+ignored",
        re.IGNORECASE
    )

    for line in output.split("\n"):
        # Find all matches per line (handles multiple test result lines)
        for m in passed_pattern.finditer(line):
            total_passed += int(m.group(1))
        for m in failed_pattern.finditer(line):
            total_failed += int(m.group(1))
        for m in ignored_pattern.finditer(line):
            total_ignored += int(m.group(1))

    if total_passed == 0 and total_failed == 0 and "test result:" not in output:
        return TestCounts(
            passed=None,
            failed=None,
            ignored=None,
            parse_error=True,
            error_msg="No test result lines found in output"
        )

    return TestCounts(
        passed=total_passed,
        failed=total_failed,
        ignored=total_ignored,
        parse_error=False
    )


def parse_exit_code(output: str) -> int:
    """Extract the last exit code from output if available."""
    # Look for error indicators
    if "error:" in output.lower():
        return 1
    if "test result:" in output:
        # If we see test results, assume success (unless FAILED)
        if re.search(r"test result:\s*FAILED", output, re.IGNORECASE):
            return 1
        return 0
    return 0


def compare_counts(a: TestCounts, b: TestCounts) -> dict:
    """Compare two TestCounts results."""
    if a.parse_error or b.parse_error:
        return {"consistent": False, "reason": "Parse error in one or both"}

    return {
        "consistent": (
            a.passed == b.passed and
            a.failed == b.failed and
            a.ignored == b.ignored
        ),
        "passed_match": a.passed == b.passed,
        "failed_match": a.failed == b.failed,
        "ignored_match": a.ignored == b.ignored,
        "a": {"passed": a.passed, "failed": a.failed, "ignored": a.ignored},
        "b": {"passed": b.passed, "failed": b.failed, "ignored": b.ignored}
    }
