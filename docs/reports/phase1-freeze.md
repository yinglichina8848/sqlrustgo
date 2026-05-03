# Phase1 Freeze Report

**Commit:** `bca79826` (`develop/v2.9.0`)
**Date:** 2026-05-03
**Status:** FROZEN — 可回滚基线

---

## Coverage

| Metric | Value |
|--------|-------|
| Function coverage | 24.0% (43/179) |
| Line coverage | 25.7% (536/2083) |
| Region coverage | 27.3% (927/3390) |
| Run1 (functions) | 24.0% |
| Run2 (functions) | 24.0% |
| **Δ stability** | **0.0%** ✅ |

---

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|
| JSON | `target/coverage/coverage.json` | ✅ 8.8MB |
| HTML | `target/coverage/html/index.html` | ✅ 1.9KB |
| LCOV | `target/coverage/lcov.info` | ✅ 70KB |

---

## Scope

**In Scope (Phase 1):**
- Unit test coverage (`--lib`)
- HTML / JSON / LCOV reports
- Gitea Actions CI workflow

**Out of Scope:**
- ❌ No server / sysbench / TPC-H
- ❌ No integration tests (compile errors pre-existing)
- ❌ No SQL corpus
- ❌ No coverage gate
- ❌ No module-level breakdown

---

## Known Limitations

| Limitation | Impact |
|------------|--------|
| Coverage only 24% | Most code paths untested |
| Integration tests excluded | `planner_property_tests.rs`, `trigger_eval_tests.rs` have pre-existing compile errors |
| HTML report 1.9KB | Few files / paths covered |
| No module breakdown | Cannot identify which crate needs coverage most |

---

## Freeze Verification

```bash
# 1. Local run
bash scripts/ci/test_all.sh
# Expected: 12 tests passed, 3 artifacts exist

# 2. Artifact check
ls -lh target/coverage/html/index.html  # 1.9KB
ls -lh target/coverage/coverage.json     # 8.8MB
ls -lh target/coverage/lcov.info        # 70KB

# 3. Stability check (2 runs)
cargo llvm-cov report --json | grep '"percent"'
# Expected: Δ 0.0% between runs
```

---

## Rollback

If future changes break coverage:

```bash
git checkout bca79826 -- scripts/ci/ .gitea/workflows/coverage.yml
```

---

## Next Phase

Phase 2: SQL corpus + server coverage + integration test compile fixes.
