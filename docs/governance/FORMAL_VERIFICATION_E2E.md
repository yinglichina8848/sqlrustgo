# SQLRustGo Formal Verification E2E Workflow

> **Issue**: #117 S-01~S-05 Formal Verification Framework
> **Status**: Draft
> **Owner**: SQLRustGo Team
> **Date**: 2026-05-02
> **Version**: 0.1.0

## 1. Overview

本文档定义了 SQLRustGo 形式化验证的端到端 (E2E) 流程，从形式化规约到代码实现再到测试验证的完整自动化流程。

### 1.1 Current Problem

当前验证框架的局限性：
- Dafny/TLA+/Formulog 证明文件仅作为**文档**存在
- 没有实际运行 Formal Verification Tool (dafny verify / tlc)
- test_results 引用 cargo test 而非形式化证明工具
- 缺少自动化的 E2E 验证流程

### 1.2 Goal

实现真正的 E2E 形式化验证：
```
Proof Specification (Dafny/TLA+/Formulog)
        ↓
   Formal Verification Tool (dafny verify / tlc)
        ↓
   Specification → Rust Code Mapping
        ↓
   cargo test / Integration Tests
        ↓
   Evidence Collection → Update PROOF-XXX.json
        ↓
   Gate R10 Check → CI Pass
```

## 2. Formal Verification Tools

### 2.1 Supported Tools

| Tool | Language | Use Case | Installation |
|------|----------|---------|--------------|
| Dafny | Verify | Type safety, B-Tree, Invariants | `dotnet tool install -g Dafny` |
| TLA+ | Model Check | WAL, MVCC, Concurrency | `apt install tla-tools` / docker |
| Formulog | Logic Programming | Query equivalence, Parser | `pip install formulog` |

### 2.2 Tool Installation

```bash
# Dafny (verification)
dotnet tool install -g Dafny

# TLA+ Toolbox (model checking)
# Using Docker (recommended)
docker pull tlatools/tlatools

# Formulog (logic programming)
pip install formulog
```

## 3. E2E Workflow

### 3.1 Workflow Phases

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    E2E Formal Verification Flow                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Phase 1: Specification                                              │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  1.1 Write formal specification (.dfy/.tla/.formulog)       │   │
│  │  1.2 Define invariants and theorems                         │   │
│  │  1.3 Document in PROOF-XXX.json evidence                   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                           ↓                                       │
│  Phase 2: Formal Verification                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  2.1 Run dafny verify / tlc model check                     │   │
│  │  2.2 Collect verification results                         │   │
│  │  2.3 Update proof status                                │   │
│  └────────────────────────────────────────���────────────────────┘   │
│                           ↓                                       │
│  Phase 3: Rust Implementation                                    │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  3.1 Manual translation or code generation                │   │
│  │  3.2 Implement critical functions                        │   │
│  │  3.3 Document spec-to-code mapping                      │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                           ↓                                       │
│  Phase 4: Testing & Evidence                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  4.1 Run cargo test (unit + integration)                 │   │
│  │  4.2 Property-based testing                            │   │
│  │  4.3 Collect test results                             │   │
│  │  4.4 Update PROOF-XXX.json evidence                  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                           ↓                                       │
│  Phase 5: Gate R10 Verification                                   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  5.1 Run scripts/gate/check_proof.sh                      │   │
│  │  5.2 Run scripts/verify_proof_registry.py                │   │
│  │  5.3 Collect verification report                      │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Phase Details

#### Phase 1: Specification

**Input**: Proof requirement (e.g., PROOF-011 Type Safety)

**Output**: Formal specification file + PROOF-XXX.json

```dafny
// PROOF-011-type-safety.dfy (example)
module TypeSystem {
  datatype Type = TInt | TBool | TString
  
  ghost function typeOf(e: Expr): Type
    decreases e
  {
    match e
    case LiteralBool(b) => TBool
    case LiteralInt(n) => TInt
    case BinaryOp(op, left, right) => 
      if op == Add then TInt else TBool
  }
  
  lemma TypeSafety(e: Expr)
    ensures typeOf(e) != TInt  // Safety property
  {}
}
```

#### Phase 2: Formal Verification

**Input**: .dfy/.tla/.formalog specification

**Output**: Verification report

```bash
# Dafny verification example
dafny verify PROOF-011-type-safety.dfy

# Output:
# Dafny program verifier version 4.0.0
# Prover: Z3
# Prover: CVC4
# ...
# Verification completed: 0 errors
```

#### Phase 3: Rust Implementation

**Input**: Formal specification

**Output**: Rust code in crates/

```rust
// crates/types/src/type_system.rs (example)
#[derive(Debug, Clone, PartialEq)]
pub enum SqlType {
    Int,
    Bool,
    String,
    // ...
}

impl SqlType {
    pub fn infer(&self, expr: &Expr) -> SqlType {
        match expr {
            Expr::Literal(lit) => lit.ty(),
            Expr::BinaryOp(op, lhs, rhs) => {
                if op.is_comparison() {
                    SqlType::Bool
                } else {
                    SqlType::Int
                }
            }
            // ...
        }
    }
}
```

#### Phase 4: Testing & Evidence

**Input**: Rust implementation

**Output**: Test results + Updated PROOF-XXX.json

```bash
# Run tests
cargo test -p sqlrustgo-types -- --nocapture

# Update evidence
{
  "proof_id": "PROOF-011",
  "evidence": {
    "test_results": "cargo test -p sqlrustgo-types: 42 passed",
    "verification_method": "Dafny formal verification + property-based testing",
    "dafny_report": "verification completed: 0 errors"
  }
}
```

#### Phase 5: Gate R10 Verification

**Input**: PROOF-XXX.json files + test results

**Output**: Gate pass/fail

```bash
# Gate check
bash scripts/gate/check_proof.sh

# Output:
# === R10: Formal Proof Check ===
# Proof files: 14 (>= 10 required)
# All files valid JSON
# ✅ R10: Formal Proof Check PASSED
```

## 4. Test Design

### 4.1 Test Categories

| Category | Tool | Target | Example |
|----------|------|--------|---------|
| Unit Test | cargo test | Individual functions | type_inference |
| Property Test | proptest | Invariants | btree_insert_rebalance |
| Integration Test | cargo test | Module interaction | wal_recovery |
| Model Check | tlc | Concurrent properties | mvcc_isolation |
| Formal Verify | dafny | Type safety | proof Obligations |

### 4.2 Test Evidence Mapping

Each proof requires specific test evidence:

| Proof | Test Type | cargo test | Formal Tool |
|-------|----------|-----------|-------------|
| PROOF-011 (Type Safety) | Unit + Property | cargo test -p sqlrustgo-types | dafny verify |
| PROOF-012 (WAL ACID) | Integration | cargo test -p sqlrustgo-transaction | tlc model check |
| PROOF-013 (B-Tree) | Property | cargo test -p sqlrustgo-storage | dafny verify |
| PROOF-014 (Equivalence) | Property | cargo test -p sqlrustgo-optimizer | formulog check |

## 5. Development Plan

### 5.1 Milestones

| Milestone | Description | Deliverables | Timeline |
|----------|------------|-------------|------------|
| M1 | Tool Setup | Install Dafny/TLA+/Formulog | Week 1 |
| M2 | Workflow Documentation | E2E workflow.md | Week 1 |
| M3 | PROOF-011 Verification | Type safety verified | Week 2 |
| M4 | PROOF-012 Verification | WAL ACID verified | Week 2 |
| M5 | PROOF-013 Verification | B-Tree verified | Week 3 |
| M6 | PROOF-014 Verification | Query equivalence verified | Week 3 |
| M7 | CI Integration | scripts/gate/check_proof.sh enhanced | Week 4 |
| M8 | Full Gate Pass | R10 verified | Week 4 |

### 5.2 Task Breakdown

#### M1: Tool Setup

- [ ] Install Dafny on build machine
- [ ] Install TLA+ Toolbox (docker)
- [ ] Install Formulog
- [ ] Create scripts/verify/dafny-verify.sh
- [ ] Create scripts/verify/tla-check.sh
- [ ] Create scripts/verify/formulog-check.sh

#### M2: Workflow Documentation

- [ ] Complete this E2E workflow document
- [ ] Add to docs/governance/FORMAL_VERIFICATION_E2E.md
- [ ] Update PROOF-XXX.json schema to include formal verification fields

#### M3: PROOF-011 Type Safety

- [ ] Review PROOF-011-type-safety.dfy specification
- [ ] Run `dafny verify`
- [ ] Fix any verification errors
- [ ] Update PROOF-011.json with verification results
- [ ] Run cargo test -p sqlrustgo-types

#### M4: PROOF-012 WAL ACID

- [ ] Review PROOF-012-wal-acid.tla specification
- [ ] Run TLA+ model checking
- [ ] Fix any model errors
- [ ] Update PROOF-012.json with model check results
- [ ] Run cargo test -p sqlrustgo-transaction

#### M5: PROOF-013 B-Tree

- [ ] Review PROOF-013-btree-invariants.dfy specification
- [ ] Run `dafny verify`
- [ ] Fix any verification errors
- [ ] Update PROOF-013.json with verification results
- [ ] Run cargo test -p sqlrustgo-storage

#### M6: PROOF-014 Query Equivalence

- [ ] Review PROOF-014-query-equivalence.formalog specification
- [ ] Run Formulog check
- [ ] Fix any logic errors
- [ ] Update PROOF-014.json with check results
- [ ] Run cargo test -p sqlrustgo-optimizer

#### M7: CI Integration

- [ ] Update scripts/gate/check_proof.sh to run formal tools
- [ ] Add formal verification to Gitea Actions
- [ ] Create Gate R10 report generation

#### M8: Full Gate Pass

- [ ] Run full R10 verification
- [ ] Generate verification report
- [ ] Update REGISTRY_INDEX.json
- [ ] Close Issue #117

## 6. Verification Evidence Schema

### 6.1 Enhanced PROOF-XXX.json Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Formal Proof",
  "type": "object",
  "required": [
    "proof_id",
    "title",
    "language",
    "category",
    "status",
    "description",
    "evidence",
    "formal_verification",
    "created_at"
  ],
  "properties": {
    "proof_id": {
      "type": "string",
      "pattern": "^PROOF-\\d{3}$"
    },
    "language": {
      "type": "string",
      "enum": ["Dafny", "TLA+", "Formulog", "Coq", "Lean", "Isabelle"]
    },
    "status": {
      "type": "string",
      "enum": ["draft", "verifying", "verified", "failed"]
    },
    "evidence": {
      "type": "object",
      "properties": {
        "test_results": {
          "type": "string",
          "description": "cargo test output"
        },
        "verification_method": {
          "type": "string",
          "enum": [
            "Dafny formal verification",
            "TLA+ model checking",
            "Formulog logic check",
            "形式化规约 + 测试验证"
          ]
        }
      }
    },
    "formal_verification": {
      "type": "object",
      "description": "Formal verification tool results",
      "properties": {
        "tool": {
          "type": "string",
          "enum": ["dafny", "tlc", "formulog"]
        },
        "version": {
          "type": "string"
        },
        "command": {
          "type": "string"
        },
        "output": {
          "type": "string"
        },
        "result": {
          "type": "string",
          "enum": ["passed", "failed", "errors"]
        },
        "verified_at": {
          "type": "string",
          "format": "date-time"
        }
      }
    }
  }
}
```

## 7. Scripts

### 7.1 scripts/verify/dafny-verify.sh

```bash
#!/usr/bin/env bash
set -e

DAFNY_FILE="$1"
OUTPUT_FILE="${DAFNY_FILE%.dfy}.verify Output"

echo "=== Running Dafny Verification ==="
echo "File: $DAFNY_FILE"
echo "Date: $(date)"

dafny verify "$DAFNY_FILE" > "$OUTPUT_FILE" 2>&1

if grep -q "Errors: 0" "$OUTPUT_FILE"; then
    echo "✅ Dafny verification PASSED"
    exit 0
else
    echo "❌ Dafny verification FAILED"
    cat "$OUTPUT_FILE"
    exit 1
fi
```

### 7.2 scripts/verify/tla-check.sh

```bash
#!/usr/bin/env bash
set -e

TLA_FILE="$1"
MODULE_NAME="$2"

echo "=== Running TLA+ Model Checking ==="
echo "Module: $MODULE_NAME"
echo "Date: $(date)"

docker run --rm -v "$(pwd):/workspace" tlatools/tlatools \
  tlc -workers auto -generate SPEC.tla

echo "✅ TLA+ model check PASSED"
```

### 7.3 scripts/verify/formulog-check.sh

```bash
#!/usr/bin/env bash
set -e

FORMULOG_FILE="$1"

echo "=== Running Formulog Check ==="
echo "File: $FORMULOG_FILE"
echo "Date: $(date)"

formulog check "$FORMULOG_FILE"

echo "✅ Formulog check PASSED"
```

## 8. References

- [Dafny Documentation](https://dafny.org/)
- [TLA+ Toolbox](https://tla-tools.github.io/)
- [Formulog](https://github.com/ucsd-progsys/formulog)
- [Issue #117](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/117)
- [G-02 Proof Registry](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/128)
- [R-Gate R10](docs/governance/RULES.md#R10)

## 9. Appendix

### A.1 Troubleshooting

| Error | Cause | Solution |
|-------|-------|----------|
| "dafny: command not found" | Dafny not installed | `dotnet tool install -g Dafny` |
| "tlc: Out of memory" | Model too complex | Reduce state space |
| "Formulog: Parse error" | Invalid syntax | Fix .formalog file |

### A.2 Dependencies

- dotnet 6.0+
- Docker (for TLA+)
- Python 3.8+ (for Formulog)

---

> **Note**: This is a live document. Update as the E2E workflow evolves.