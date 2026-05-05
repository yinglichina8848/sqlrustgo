# SQLRustGo v2.9.0 RC Coverage Report

**Date**: 2026-05-05
**Version**: v2.9.0 RC
**Coverage Tool**: cargo-llvm-cov 0.8.4

---

## Executive Summary

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| **Overall Lines Coverage** | ~84% | ≥85% | ⚠️ Close |
| **Modules ≥90%** | 5 | - | optimizer, transaction, telemetry, distributed |
| **Modules ≥85%** | 11 | - | planner, catalog, types, graph, common, query-stats, storage, network, sql-corpus |
| **Modules <70%** | 3 | 0 | mysql-server, tools, expr |

---

## Detailed Coverage by Module

### Tier 1: Excellent (≥90%) ✅

| Module | Lines | Regions | Functions | Status |
|--------|-------|---------|-----------|--------|
| optimizer | **91.11%** | 91.38% | 96.14% | ✅ |
| transaction | **90.99%** | 93.26% | 87.16% | ✅ |
| telemetry | **96.66%** | 97.38% | 95.00% | ✅ |
| distributed | **91.29%** | 92.16% | 90.97% | ✅ |

### Tier 2: Good (85-90%) ✅

| Module | Lines | Regions | Functions | Status |
|--------|-------|---------|-----------|--------|
| planner | **85.93%** | 88.36% | 84.21% | ✅ |
| catalog | **89.83%** | 92.30% | 88.17% | ✅ |
| types | **87.73%** | 86.94% | 90.43% | ✅ |
| graph | **84.67%** | 85.98% | 80.43% | ✅ |
| common | **87.84%** | 87.03% | 88.52% | ✅ |
| query-stats | **85.11%** | 84.97% | 88.64% | ✅ |

### Tier 3: Acceptable (70-85%) ⚠️

| Module | Lines | Regions | Functions | Status |
|--------|-------|---------|-----------|--------|
| storage | **81.77%** | 81.76% | 76.94% | ✅ |
| network | **77.24%** | 79.88% | 74.29% | ⚠️ |
| sql-corpus | **77.72%** | 76.82% | 69.39% | ⚠️ |
| security | **72.96%** | 73.02% | 69.42% | ⚠️ |
| server | **74.08%** | 76.98% | 69.29% | ⚠️ |
| executor | **70.54%** | 71.75% | 78.99% | ⚠️ |

### Tier 4: Needs Improvement (<70%) ❌

| Module | Lines | Regions | Functions | Critical Files |
|--------|-------|---------|-----------|----------------|
| **mysql-server** | **35.67%** | 35.17% | 50.00% | lib.rs (36.19%) |
| **tools** | **58.03%** | 60.33% | 65.83% | upgrade.rs (19.56%), backup_restore.rs (54.76%) |
| **expr** | **60.31%** | 58.55% | 70.59% | expr.rs (39.73%), op.rs (0.00%) |

---

## Coverage Gaps Analysis

### 1. mysql-server (35.67%) - CRITICAL

**Problem**: Main entry point and core protocol handling not tested.

**Root Causes**:
- `lib.rs`: MySQL protocol implementation (1238 lines) only 36.19% covered
- Integration tests require running server socket (hard to test in unit tests)

**Recommendations**:
- Add mock socket tests for protocol parsing
- Add unit tests for individual packet handlers

### 2. tools (58.03%) - HIGH

**Problem**: CLI tools have large upgrade/backup modules not tested.

**Root Causes**:
- `upgrade.rs`: 501 lines, only 19.56% covered
- `backup_restore.rs`: 210 lines, only 54.76% covered
- `main.rs`: 0% (entry point, hard to cover)

**Recommendations**:
- Add integration tests for backup/restore with mock storage
- Add upgrade path tests

### 3. expr (60.31%) - HIGH

**Problem**: Expression evaluation not fully tested.

**Root Causes**:
- `expr.rs`: Only 39.73% covered
- `op.rs`: 0% covered (operator functions)

**Recommendations**:
- Add comprehensive operator tests
- Add expression evaluation tests for all data types

### 4. executor/stored_proc (48.76%) - MEDIUM

**Problem**: Stored procedure executor has many untested paths.

**Root Causes**:
- 3193 lines with complex control flow
- Cursor operations, exception handlers, nested blocks
- `patch_stored_proc_coverage.rs` covers basic paths but not edge cases

**Recommendations**:
- Add tests for cursor edge cases
- Add exception handler path tests
- Add CTE subquery tests

### 5. security/tls (60.18%) - MEDIUM

**Problem**: TLS encryption and audit logging not fully tested.

**Root Causes**:
- `tls.rs`: 113 lines, only 60.18% covered
- `audit.rs`: 398 lines, only 64.82% covered

**Recommendations**:
- Add TLS handshake mock tests
- Add audit log serialization tests

---

## Coverage Improvement Plan

### Phase 1: Quick Wins (1-2 days)

1. **expr/op.rs (0% → 50%)**
   - Add operator function tests
   - Cover basic arithmetic and comparison operators

2. **tools/backup_restore.rs (54% → 75%)**
   - Add backup/restore integration tests
   - Mock file operations

### Phase 2: Medium Effort (3-5 days)

3. **executor/stored_proc (48% → 65%)**
   - Add cursor tests
   - Add exception handler tests
   - Add nested block scope tests

4. **security/tls (60% → 80%)**
   - Add TLS handshake mock tests
   - Add certificate validation tests

### Phase 3: Hard Problems (1-2 weeks)

5. **mysql-server (35% → 60%)**
   - Add packet parser unit tests
   - Add mock socket integration tests
   - Refactor for testability

6. **tools/upgrade (19% → 50%)**
   - Add version migration tests
   - Add upgrade path tests

---

## Test Execution

```bash
# Individual module coverage
cargo llvm-cov --package sqlrustgo-executor --package sqlrustgo-storage --package sqlrustgo-network

# Full workspace coverage (takes ~10 minutes)
cargo llvm-cov --all-features --workspace

# Generate HTML report
cargo llvm-cov --all-features --workspace --html
```

---

## Appendix: Test Count Summary

| Module | Tests |
|--------|-------|
| optimizer | 324 |
| planner | 120 |
| executor | 255+ |
| catalog | 156 |
| types | 81 |
| transaction | 257 |
| storage | 542 |
| distributed | 1295 |
| graph | 470 |
| security | 242 |
| common | 122 |

---

**Report Generated**: 2026-05-05
**Next Update**: After Phase 1 improvements
