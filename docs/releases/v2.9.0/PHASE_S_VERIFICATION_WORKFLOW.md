# Phase S: SQL Formal Verification Workflow

> **Issue**: #117 S-01~S-05 Formal Verification
> **Status**: Tools Pending Installation
> **Owner**: SQLRustGo Team
> **Date**: 2026-05-03

---

## 1. Overview

Phase S establishes formal verification for SQLRustGo's core components using:
- **Dafny**: Type safety, B+Tree invariants
- **TLA+**: WAL recovery, MVCC isolation
- **Formulog**: Parser correctness, query equivalence

### 1.1 Current Status

| Tool | Status | Installation |
|------|--------|--------------|
| Dafny | Not installed | `dotnet tool install -g Dafny` |
| TLA+ | Not installed | `docker pull tlatools/tlatools` |
| Formulog | Not installed | `pip install formulog` |

### 1.2 Proof Coverage

| Category | Proofs | Language | Test Evidence |
|----------|--------|----------|---------------|
| S-01 Parser | PROOF-001,006,007,008,010 | Formulog | 100 parser tests |
| S-02 Type System | PROOF-002,011 | Dafny | 81 type tests |
| S-03 Transaction ACID | PROOF-003,005,012 | TLA+ | 14 transaction tests |
| S-04 B+Tree | PROOF-004,013 | Dafny | 180 storage tests |
| S-05 Query Equivalence | PROOF-014 | Formulog | 233 optimizer tests |

---

## 2. Tool Installation

### 2.1 Prerequisites

```bash
# Check prerequisites
dotnet --version  # Required for Dafny
docker --version   # Required for TLA+
python3 --version  # Required for Formulog
pip3 --version     # Required for Formulog
```

### 2.2 Install Commands

```bash
# Dafny (verification for Dafny proofs)
dotnet tool install -g Dafny

# TLA+ Toolbox (model checking for TLA+ specs)
docker pull tlatools/tlatools

# Formulog (logic programming for Formulog specs)
pip install formulog
```

### 2.3 Verify Installation

```bash
# Verify Dafny
dafny --version

# Verify TLA+ (Docker)
docker run --rm tlatools/tlatools tlc -version

# Verify Formulog
formulog --version
```

---

## 3. Proof Verification Workflow

### 3.1 S-01: Parser Correctness (Formulog)

**Proofs**: PROOF-001, PROOF-006, PROOF-007, PROOF-008, PROOF-010

```bash
# Install Formulog (if not installed)
pip install formulog

# Run verification for each proof
for proof in PROOF-001 PROOF-006 PROOF-007 PROOF-008 PROOF-010; do
    echo "=== Verifying $proof ==="
    formulog check "docs/proof/${proof}.formulog"
done

# Update evidence after successful verification
# Edit docs/proof/${proof}.json and set:
# "formal_verification": { "status": "verified", "verified_at": "2026-05-03T..." }
```

### 3.2 S-02: Type System Safety (Dafny)

**Proofs**: PROOF-002, PROOF-011

```bash
# Install Dafny (if not installed)
dotnet tool install -g Dafny

# Run verification for each proof
for proof in PROOF-002 PROOF-011; do
    echo "=== Verifying $proof ==="
    dafny verify "docs/proof/${proof}.dfy"
done
```

### 3.3 S-03: Transaction ACID (TLA+)

**Proofs**: PROOF-003, PROOF-005, PROOF-012

```bash
# Run model checking for each spec
for proof in PROOF-003 PROOF-005 PROOF-012; do
    echo "=== Model Checking $proof ==="
    docker run --rm \
        -v "$(pwd):/workspace" \
        -w /workspace \
        tlatools/tlatools \
        tlc -workers auto "docs/proof/${proof}.tla"
done
```

### 3.4 S-04: B+Tree Invariants (Dafny)

**Proofs**: PROOF-004, PROOF-013

```bash
# Run verification for each proof
for proof in PROOF-004 PROOF-013; do
    echo "=== Verifying $proof ==="
    dafny verify "docs/proof/${proof}.dfy"
done
```

### 3.5 S-05: Query Equivalence (Formulog)

**Proofs**: PROOF-014

```bash
# Verify query equivalence framework
echo "=== Verifying PROOF-014 ==="
formulog check "docs/proof/PROOF-014.formulog"
```

---

## 4. Automated Verification Script

Create `scripts/verify/run_all_proofs.sh`:

```bash
#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== SQLRustGo Formal Verification ==="
echo "Date: $(date)"
echo ""

cd "$PROJECT_ROOT"

FAILED=0
PASSED=0

verify_dafny() {
    local proof="$1"
    local file="docs/proof/${proof}.dfy"
    if [ ! -f "$file" ]; then
        echo "⚠️  $proof not found, skipping"
        return
    fi
    echo "[Dafny] Verifying $proof..."
    if dafny verify "$file" > /dev/null 2>&1; then
        echo "  ✅ $proof verified"
        PASSED=$((PASSED + 1))
    else
        echo "  ❌ $proof failed"
        FAILED=$((FAILED + 1))
    fi
}

verify_tla() {
    local proof="$1"
    local file="docs/proof/${proof}.tla"
    if [ ! -f "$file" ]; then
        echo "⚠️  $proof not found, skipping"
        return
    fi
    echo "[TLA+] Model checking $proof..."
    if docker run --rm -v "$(pwd):/workspace" -w /workspace tlatools/tlatools tlc -workers auto "$file" > /dev/null 2>&1; then
        echo "  ✅ $proof verified"
        PASSED=$((PASSED + 1))
    else
        echo "  ❌ $proof failed"
        FAILED=$((FAILED + 1))
    fi
}

verify_formulog() {
    local proof="$1"
    local file="docs/proof/${proof}.formalog"
    if [ ! -f "$file" ]; then
        echo "⚠️  $proof not found, skipping"
        return
    fi
    echo "[Formulog] Checking $proof..."
    if formulog check "$file" > /dev/null 2>&1; then
        echo "  ✅ $proof verified"
        PASSED=$((PASSED + 1))
    else
        echo "  ❌ $proof failed"
        FAILED=$((FAILED + 1))
    fi
}

# S-01: Parser (Formulog)
echo "=== S-01: Parser Correctness ==="
for proof in PROOF-001 PROOF-006 PROOF-007 PROOF-008 PROOF-010; do
    verify_formulog "$proof"
done

# S-02: Type System (Dafny)
echo "=== S-02: Type System Safety ==="
for proof in PROOF-002 PROOF-011; do
    verify_dafny "$proof"
done

# S-03: Transaction ACID (TLA+)
echo "=== S-03: Transaction ACID ==="
for proof in PROOF-003 PROOF-005 PROOF-012; do
    verify_tla "$proof"
done

# S-04: B+Tree (Dafny)
echo "=== S-04: B+Tree Invariants ==="
for proof in PROOF-004 PROOF-013; do
    verify_dafny "$proof"
done

# S-05: Query Equivalence (Formulog)
echo "=== S-05: Query Equivalence ==="
verify_formulog "PROOF-014"

echo ""
echo "=== Summary ==="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✅ All formal verifications passed!"
    exit 0
else
    echo "❌ $FAILED verification(s) failed"
    exit 1
fi
```

Make executable:
```bash
chmod +x scripts/verify/run_all_proofs.sh
```

---

## 5. CI Integration

### 5.1 Gate R10 Enhancement

Update `scripts/gate/check_proof.sh` to optionally run formal verification:

```bash
# Add to check_proof.sh after JSON validation
if [ "${RUN_FORMAL_VERIFICATION:-false}" = "true" ]; then
    echo "Running formal verification..."
    bash scripts/verify/run_all_proofs.sh
fi
```

### 5.2 Gitea Actions Workflow

Add to `.github/workflows/r-gate.yml`:

```yaml
  gate-r10-formal-verification:
    name: R10: Formal Verification (Optional)
    runs-on: ubuntu-latest
    if: env.RUN_FORMAL_VERIFICATION == 'true'
    steps:
      - uses: actions/checkout@v4
      - name: Install Dafny
        run: dotnet tool install -g Dafny
      - name: Run Formal Verification
        env:
          RUN_FORMAL_VERIFICATION: true
        run: bash scripts/verify/run_all_proofs.sh
```

---

## 6. Troubleshooting

| Issue | Solution |
|-------|----------|
| "dafny: command not found" | Run `dotnet tool install -g Dafny` |
| "docker: command not found" | Install Docker or use TLA+ Toolbox directly |
| "formulog: command not found" | Run `pip install formulog` |
| TLA+ out of memory | Reduce state space or use TLC model checker with constraints |

---

## 7. References

- [Dafny Getting Started](https://dafny.org/)
- [TLA+ Toolbox Download](https://tla-tools.github.io/)
- [Formulog GitHub](https://github.com/ucsd-progsys/formulog)
- [Formal Verification E2E](../governance/FORMAL_VERIFICATION_E2E.md)
- [Issue #117](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/117)