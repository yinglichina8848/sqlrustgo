"""Audit runner — cargo test execution layer."""
import subprocess
import os

def run_cargo_test(extra_args=None):
    """Run cargo test and return raw stdout + return code."""
    args = ["cargo", "test"]
    if extra_args:
        args.extend(extra_args)
    result = subprocess.run(
        args,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        cwd=os.environ.get("CARGO_TESTS_DIR", ".")
    )
    return result.stdout, result.returncode

def run_multiple(n=3, extra_args=None):
    """Run cargo test N times for determinism check."""
    results = []
    for i in range(n):
        out, code = run_cargo_test(extra_args=extra_args)
        results.append((code, out))
    return results
