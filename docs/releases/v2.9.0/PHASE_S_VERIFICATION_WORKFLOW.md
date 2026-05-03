# Phase S: SQL Formal Verification Workflow

> **Issue**: #117 S-01~S-05 Formal Verification
> **Status**: Phase 1 Complete (4/9 proofs verified)
> **Owner**: SQLRustGo Team
> **Date**: 2026-05-03

---

## 1. Overview

Phase S establishes formal verification for SQLRustGo's core components using three tools:
- **Dafny**: Type safety, B+Tree invariants
- **TLA+**: WAL recovery, MVCC isolation, JOIN algorithm, DDL atomicity
- **Formulog**: Parser correctness, query equivalence, UPDATE/DELETE semantics

### 1.1 Current Status (2026-05-03)

| Tool | Status | Installation |
|------|--------|--------------|
| Dafny | ✅ Installed at /usr/bin/dafny | `dotnet tool install -g Dafny` |
| TLA+ | ✅ JAR at ~/.local/toolchain/tla/tlatools.jar | `curl -LO .../tlatools-1.8.0.jar` |
| Formulog | ⚠️ JAR only (no CLI) | Use Docker runner below |
| Z3 | ✅ Installed at /usr/bin/z3 | `apt install z3` |

### 1.2 Phase 1 Verified Proofs (4/9)

| Proof | Title | Tool | Status |
|-------|-------|------|--------|
| PROOF-015 | DDL Atomicity (CREATE/DROP TABLE) | TLA+ | ✅ Verified |
| PROOF-016 | MVCC SSI Conflict Detection | TLA+ | ✅ Verified |
| PROOF-017 | UPDATE/DELETE Semantics | Formulog | ✅ Verified |
| PROOF-019 | LEFT/RIGHT OUTER JOIN Algorithm | TLA+ | ✅ Verified |

### 1.3 Remaining Phase 1 Proofs (5/9)

| Proof | Title | Tool | Status |
|-------|-------|------|--------|
| PROOF-020 | NULL Three-Valued Logic (3VL) | Formulog | ⏳ Pending |
| PROOF-021 | HAVING Semantics | Formulog | ⏳ Pending |
| PROOF-022 | CTE Non-Recursive | Formulog | ⏳ Pending |
| PROOF-023 | Multi-Transaction Deadlock Detection | TLA+ | ⏳ Pending |
| PROOF-024 | Aggregate Overflow | Dafny | ⏳ Pending |
| PROOF-025 | GRANT/REVOKE Permission Propagation | Formulog | ⏳ Pending |

---

## 2. Toolchain Installation & Verification

### 2.1 Quick Install (All Tools)

```bash
# Full toolchain install
bash scripts/deploy/install_verification_toolchain.sh --all

# Verify only (no install)
bash scripts/deploy/install_verification_toolchain.sh --check-only
```

### 2.2 Manual Install

**Dafny**:
```bash
dotnet tool install -g Dafny
# Verify: dafny /help
```

**TLA+**:
```bash
mkdir -p ~/.local/toolchain/tla
curl -sSL https://github.com/tlaplus/tlatools/releases/download/v1.8.0/tlatools-1.8.0.jar \
  -o ~/.local/toolchain/tla/tlatools.jar
# Run: java -cp ~/.local/toolchain/tla/tlatools.jar tlc2.TLC
```

**Z3 (required by Formulog)**:
```bash
apt install z3       # Linux
brew install z3     # macOS
# Verify: z3 --version
```

### 2.3 Verify Installation

```bash
$ bash scripts/deploy/install_verification_toolchain.sh --check-only
Tool Versions:
===============
Z3: Z3 version 4.8.12 - 64 bit
Dafny: Installed (at /usr/bin/dafny)
TLA+: ~/.local/toolchain/tla/tlatools.jar
Java: openjdk version "17.0.12"
Python: Python 3.10.12
Formulog: not installed (use Docker runner)
```

---

## 3. Formulog Isolated Runner (Critical)

### 3.1 The SymbolManager Problem

Formulog 0.8.0 has a JVM-level `SymbolManager` singleton that gets polluted when running multiple proofs in the same JVM process. This causes **nondeterministic results** — the same proof can produce different answers depending on what ran before it.

**Solution**: Run each proof in an isolated Docker container.

### 3.2 Formulog Runner Script

**Location**: `scripts/formulog/run_formulog_isolated.sh`

```bash
# Make executable
chmod +x scripts/formulog/run_formulog_isolated.sh

# Run a single proof
./scripts/formulog/run_formulog_isolated.sh docs/proof/PROOF-017-update-semantics.formulog

# Run with custom JAR
FORMULOG_JAR=/path/to/formulog.jar ./scripts/formalog/run_formulog_isolated.sh ...
```

**Requirements**:
- Docker installed
- Z3 on host at `/usr/bin/z3` (or set `Z3_PATH`)
- Formulog JAR at `/tmp/formulog-0.8.0.jar` (or set `FORMULOG_JAR`)
- Container image `tla-java17:latest` (auto-pulled on first run)

### 3.3 Download Formulog JAR

```bash
# Download if not present
if [ ! -f /tmp/formulog-0.8.0.jar ]; then
  curl -sSL https://github.com/ucsd-progsys/formulog/releases/download/v0.8.0/formulog-0.8.0.jar \
    -o /tmp/formulog-0.8.0.jar
fi
```

### 3.4 Known Limitations

1. **No comment syntax**: Formulog 0.8.0 does NOT support `//` or `/* */` comments — files must be comment-free
2. **No multi-file runs**: Each proof must be self-contained
3. **Z3 required**: Z3 must be mounted into the container (the runner does this automatically)

---

## 4. Proof Verification Workflow

### 4.1 S-01: Parser Correctness (Formulog)

**Proofs**: PROOF-001, PROOF-006, PROOF-007, PROOF-008, PROOF-010

```bash
# Note: These proofs must be run individually due to SymbolManager pollution
for proof in PROOF-001 PROOF-006 PROOF-007 PROOF-008 PROOF-010; do
    echo "=== Verifying $proof ==="
    ./scripts/formalog/run_formulog_isolated.sh "docs/proof/${proof}.formulog" 2>&1 | grep -E "PASSED|FAILED|ERROR"
done
```

### 4.2 S-02: Type System Safety (Dafny)

**Proofs**: PROOF-002, PROOF-011

```bash
for proof in PROOF-002 PROOF-011; do
    echo "=== Verifying $proof ==="
    dafny verify "docs/proof/${proof}.dfy" 2>&1 | tail -5
done
```

### 4.3 S-03: Transaction ACID (TLA+)

**Proofs**: PROOF-003, PROOF-005, PROOF-012

```bash
TLA_JAR="$HOME/.local/toolchain/tla/tlatools.jar"
for proof in PROOF-003 PROOF-005 PROOF-012; do
    echo "=== Model Checking $proof ==="
    java -cp "$TLA_JAR" tlc2.TLC -workers auto "docs/proof/${proof}.tla" 2>&1 | tail -10
done
```

### 4.4 S-04: B+Tree Invariants (Dafny)

**Proofs**: PROOF-004, PROOF-013

```bash
for proof in PROOF-004 PROOF-013; do
    echo "=== Verifying $proof ==="
    dafny verify "docs/proof/${proof}.dfy" 2>&1 | tail -5
done
```

### 4.5 S-05: Query Equivalence (Formulog)

**Proofs**: PROOF-014

```bash
./scripts/formalog/run_formulog_isolated.sh docs/proof/PROOF-014-query-equivalence.formulog
```

---

## 5. S0-S05 Verification Status (2026-05-03 Assessment)

### S-01: Parser Correctness
| Proof | Evidence | Status |
|-------|----------|--------|
| PROOF-001 parser-select | 100 parser tests | ✅ Inferred verified |
| PROOF-006 where-semantics | 100 parser tests | ✅ Inferred verified |
| PROOF-007 join-syntax | parser tests | ✅ Inferred verified |
| PROOF-008 orderby-semantics | parser tests | ✅ Inferred verified |
| PROOF-010 subquery-nesting | parser tests | ✅ Inferred verified |

**S-01 Assessment**: 🟡 **Partial** — proofs exist as JSON evidence files, but Formulog runner was not executed for re-verification. The JSON files show "verified" status from earlier runs. However, due to the SymbolManager pollution issue, these should be re-verified using the isolated runner.

### S-02: Type System Safety
| Proof | Evidence | Status |
|-------|----------|--------|
| PROOF-002 type-inference | JSON evidence | ✅ Inferred verified |
| PROOF-011 type-safety | Dafny .dfy + JSON | ✅ Inferred verified |

**S-02 Assessment**: 🟡 **Partial** — Dafny .dfy files exist and JSON evidence shows "verified". However, `dafny verify` was not re-run in this session.

### S-03: Transaction ACID
| Proof | Evidence | Status |
|-------|----------|--------|
| PROOF-003 wal-recovery | JSON evidence | ✅ Inferred verified |
| PROOF-005 mvcc-snapshot | JSON evidence | ✅ Inferred verified |
| PROOF-012 wal-acid | TLA+ .tla + JSON | ✅ Inferred verified |

**S-03 Assessment**: 🟡 **Partial** — TLC model checking was run in earlier sessions, TLA+ specs exist. Not re-verified in current session.

### S-04: B+Tree Invariants
| Proof | Evidence | Status |
|-------|----------|--------|
| PROOF-004 btree-query | JSON evidence | ✅ Inferred verified |
| PROOF-013 btree-invariants | Dafny .dfy + JSON | ✅ Inferred verified |

**S-04 Assessment**: 🟡 **Partial** — .dfy files and JSON evidence exist. `dafny verify` was not re-run in current session.

### S-05: Query Equivalence
| Proof | Evidence | Status |
|-------|----------|--------|
| PROOF-014 query-equivalence | Formulog .formulog + JSON | ⚠️ Needs re-verification |

**S-05 Assessment**: 🔴 **Incomplete** — .formulog file exists but Formulog runner was not executed. Due to SymbolManager pollution, this needs the isolated Docker runner.

---

## 6. S0-S05 Completion Matrix

| ID | Category | Proofs | Tool | Evidence | Phase 1 Verified | Re-verified |
|----|----------|--------|------|----------|-----------------|-------------|
| S-01 | Parser | 5 | Formulog | JSON | 5 | ❌ No |
| S-02 | Type System | 2 | Dafny | .dfy + JSON | 2 | ❌ No |
| S-03 | Transaction ACID | 3 | TLA+ | .tla + JSON | 3 | ❌ No |
| S-04 | B+Tree | 2 | Dafny | .dfy + JSON | 2 | ❌ No |
| S-05 | Query Equivalence | 1 | Formulog | .formulog + JSON | 0 | ❌ No |

**Conclusion**: S0-S05 verification evidence exists from earlier sessions (JSON files show "verified"), but NONE of the proofs were re-verified in this session using the isolated runner. The Phase 1 proofs that ARE actively verified (PROOF-015, 016, 017, 019) are NOT part of the original S0-S05 scope — they are Phase 2 proofs.

**Recommendation**: After Phase 2 is complete, re-run all S0-S05 proofs using the new isolated Formulog runner and confirm they still pass.

---

## 7. Automated Verification Script

Location: `scripts/verify/run_all_proofs.sh`

```bash
chmod +x scripts/verify/run_all_proofs.sh

# Run all proofs (S0-S05)
./scripts/verify/run_all_proofs.sh
```

Note: The `run_all_proofs.sh` script currently uses `formulog check` directly. Update to use the isolated runner for Formulog proofs:
```bash
verify_formulog() {
    local proof="$1"
    local file="docs/proof/${proof}.formulog"
    if [ ! -f "$file" ]; then
        echo "⚠️  $proof not found, skipping"
        return
    fi
    echo "[Formulog] Checking $proof..."
    # Use isolated runner instead of direct formulog call
    if ./scripts/formalog/run_formulog_isolated.sh "$file" > /dev/null 2>&1; then
        echo "  ✅ $proof verified"
        PASSED=$((PASSED + 1))
    else
        echo "  ❌ $proof failed"
        FAILED=$((FAILED + 1))
    fi
}
```

---

## 8. CI Integration

### 8.1 Isolated Formulog in CI

```yaml
# In Gitea Actions workflow
verify-formulog-proof:
  script:
    - chmod +x scripts/formalog/run_formulog_isolated.sh
    - for proof in PROOF-014 PROOF-017 PROOF-020 PROOF-021; do
        ./scripts/formalog/run_formulog_isolated.sh "docs/proof/${proof}.formulog" || exit 1
      done
```

### 8.2 TLA+ Model Checking in CI

```yaml
verify-tla-proof:
  script:
    - for spec in PROOF-012 PROOF-015 PROOF-016 PROOF-019; do
        java -cp $HOME/.local/toolchain/tla/tlatools.jar tlc2.TLC -workers auto "docs/formal/${spec}.tla" || exit 1
      done
```

---

## 9. Troubleshooting

| Issue | Solution |
|-------|----------|
| `dafny: command not found` | Run `dotnet tool install -g Dafny` |
| `java -cp ... tlc2.TLC: not found` | Check JAR path: `ls ~/.local/toolchain/tla/tlatools.jar` |
| Formulog `//` comment error | Strip comments: `sed 's\|//.*$\|\|g' file.formulog > clean.formulog` |
| Formulog `Cannot create symbol` | Use isolated Docker runner — SymbolManager is polluted |
| Formulog `Cannot unify int and bv[32]` | Use isolated Docker runner — stale sort cache |
| TLA+ out of memory | Add `-depth 100` constraint or reduce state space |

---

## 10. Key Findings (2026-05-03)

1. **Formulog 0.8.0 SymbolManager Pollution**: Running multiple proofs in the same JVM process causes nondeterministic results. The `SymbolManager` singleton caches type information and refuses to rebuild when a new file is loaded.

2. **Solution**: Each proof must run in its own Docker container with a fresh JVM. The `scripts/formalog/run_formulog_isolated.sh` script implements this.

3. **Comment Syntax**: Formulog 0.8.0 does NOT support `//` or `/* */` comments. All proof files must be comment-free.

4. **Z3 Requirement**: Formulog requires Z3 to be mounted into the container at `/usr/bin/z3`.

5. **Phase 1 Verified Proofs** (actively verified this session):
   - PROOF-015 (DDL Atomicity) — TLA+ ✅
   - PROOF-016 (MVCC SSI) — TLA+ ✅
   - PROOF-017 (UPDATE/DELETE) — Formulog (isolated runner) ✅
   - PROOF-019 (LEFT/RIGHT JOIN) — TLA+ ✅

6. **S0-S05 Status**: Evidence exists (JSON files marked "verified"), but none were re-verified with the isolated runner. The original S0-S05 scope covers S-01 through S-05 (13 proofs total), but the Phase 1 actively-verified proofs are a different set (4 proofs, all in the Phase 2 category).

---

## 11. References

- [Dafny Getting Started](https://dafny.org/)
- [TLA+ Toolbox Download](https://tla-tools.github.io/)
- [Formulog GitHub](https://github.com/ucsd-progsys/formulog)
- [Z3 Solver](https://github.com/Z3Prover/z3)
