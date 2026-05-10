# Coverage Report (v3.0.0)

> **Date**: 2026-05-10
> **Branch**: `develop/v3.0.0`
> **Gate Target**: R5 ≥85% (RC Gate)
> **Mode**: Debug mode (--all-features, llvm-cov)

---

## Executive Summary

| Phase | Date | Coverage | Target | Status |
|-------|------|----------|--------|--------|
| Beta | 2026-05-08 | ~76% (release) | ≥75% | ✅ PASS |
| RC | 2026-05-10 | ~76% (release, est.) | ≥85% | ⚠️ TBD |

> **Note**: Full release-mode coverage requires ~10GB disk and is run on CI runner.
> Debug-mode coverage (~42%) is not gate-comparable.
> See [BETA_GATE_REPORT.md](./BETA_GATE_REPORT.md) for last verified release data.

---

## RC Gate R5 Requirements

Per [gate_spec_v300.md](../../governance/gate_spec_v300.md):

| Stage | Gate | Coverage Target |
|-------|------|----------------|
| Alpha | A-Gate | ≥50% |
| Beta | B-Gate | ≥75% |
| RC | **R-Gate** | **≥85%** |
| GA | G-Gate | ≥85% |

---

## Historical Coverage

| Date | Branch | Mode | Coverage | Gate |
|------|--------|------|----------|------|
| 2026-05-08 | beta/v3.0.0 | Release | ~76% | B-Gate ✅ |
| 2026-05-06 | develop/v3.0.0 | Release | ~76% | B-Gate ✅ |
| 2026-04-28 | develop/v2.9.0 | Release | ~72% | B-Gate ✅ |

---

## R5 Coverage Analysis (from RC_GATE_REPORT)

Per [RC_GATE_REPORT.md](./RC_GATE_REPORT.md):

| Crate | Line Coverage | Target | Gap |
|-------|-------------|--------|-----|
| sqlrustgo-parser | 63.01% | 85% | **-21.99%** ⚠️ |
| sqlrustgo-planner | 73.83% | 85% | -11.17% |
| sqlrustgo-storage | 76.49% | 85% | -8.51% |
| sqlrustgo-optimizer | 84.12% | 85% | -0.88% ✅ |
| sqlrustgo-executor | 84.38% | 85% | +0.62% ✅ |
| sqlrustgo-types | ~90% | 85% | +5% ✅ |
| sqlrustgo-transaction | ~90% | 85% | +5% ✅ |

---

## Coverage Improvement Actions (R5)

| ID | Action | Est. Impact | Priority |
|----|--------|------------|----------|
| C-01 | Un-ignore 101 parser tests (Issue #526) | ~+5% parser | P0 |
| C-02 | Add DDL coverage tests (CREATE/DROP/ALTER TABLE) | ~+3% parser | P0 |
| C-03 | Add DCL coverage tests (GRANT/REVOKE/CREATE ROLE) | ~+1% parser | P1 |
| C-04 | Add transaction coverage tests | ~+2% executor | P1 |

---

## RC Gate Evidence

- [BETA_GATE_REPORT.md](./BETA_GATE_REPORT.md) — B-Gate verified coverage ~76%
- [RC_GATE_REPORT.md](./RC_GATE_REPORT.md) — RC R5 gap analysis

---

*Generated: 2026-05-10 by Hermes Agent*
