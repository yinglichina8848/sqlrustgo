# SQLRustGo Governance v2.8.1

**Tag**: `hermes-stable-v2.8.1`  
**Status**: Stable baseline

---

## System Architecture

```
Hermes Agent
    │
    ├── wiki/           (knowledge base, R1-R7, contracts, attack model)
    ├── contract/       (machine-readable v2.8.0.json)
    ├── scripts/
    │   ├── self_audit.py    (independent audit engine)
    │   ├── audit/
    │   │   ├── runner.py    (cargo test executor)
    │   │   ├── parser.py    (independent output parser)
    │   │   └── checks.py    (R1-R6, AV2, AV6, AV9)
    │   └── verification_engine.py  (CI proof generator)
    └── verification_report.json    (proof artifact)
```

---

## Hermes Execution Rules

### RULE H1: Pre-Audit Required

BEFORE modifying any of the following, run `make audit`:

| Scope | Examples |
|-------|----------|
| CI / verification | `.github/workflows/ci.yml`, `scripts/verification_engine.py` |
| Governance artifacts | `contract/`, `wiki/` |
| Test execution logic | `scripts/audit/` |
| Release operations | `git tag`, version bumps |

### RULE H2: Audit-Driven Commit

Before `git commit` involving governance-critical files, run `make audit`.

If audit status is not `TRUSTED` or `WEAKENED`:
- **STOP** — do not commit
- Fix the violation first

---

## Commands

```bash
make audit          # Full audit (3x runs, proof vs reality)
make audit-quick    # Fast audit (1 run)
make test           # cargo test
make verify         # Check contract + proof consistency
make clean          # Remove audit artifacts
```

---

## Audit Levels

```
🟢 TRUSTED      — All checks pass. System integrity confirmed.
🟡 WEAKENED     — Critical failures empty. Warnings present (expected: AV5, AV6).
🔴 FAIL         — Critical failure detected. STOP.

Status is printed by: python3 scripts/self_audit.py --runs 3
```

---

## Dual-Path Verification (v2.8.1)

```
Path A (CI):
  cargo test → verification_engine.py → verification_report.json

Path B (Audit):
  cargo test → audit/parser.py (independent) → proof vs reality
```

AV4 (verification_engine tampering) is detected by comparing Path A and Path B.

---

## Known Limitations (Do Not Attempt to "Fix")

| ID | Limitation | Why |
|----|-----------|-----|
| L1 | Semantic correctness undetectable | Tests verify behavior, not relational algebra |
| AV5 | Semantic regression | By design — requires GBrain |
| AV1 | Macro bypass | R1 only checks `tests/` dir |
| AV7 | Baseline poisoning | Requires external artifact store |

---

## Stable Components (Do Not Restructure)

These are frozen at `hermes-stable-v2.8.1`:

- `contract/v2.8.0.json` — structure frozen
- `scripts/self_audit.py` — interface frozen
- `scripts/audit/` — module structure frozen
- `wiki/` — knowledge base structure frozen

---

## Governance Rules Reference

| Rule | Statement |
|------|-----------|
| R1 | IF PR modifies `tests/` → REJECT |
| R2 | IF PR introduces `#[ignore]` → REJECT |
| R3 | Proof must be CI-generated, not PR-submitted |
| R4 | All 226 tests must pass (R4) |
| R5 | `baseline_verified == true` |
| R6 | Test count ≥ 226 (strict: == 226) |
| R7 | CI workflow cannot be modified by PR |

---

## Next Phase: v2.9.0

- GBrain semantic inference engine
- CI-integrated self-audit (not manual)
- audit_report.json as CI artifact
- LLM-Wiki query interface
