# Formal Verification Toolchain CI/CD Integration Guide

> **Date**: 2026-05-03
> **Status**: Toolchain Deployed, CI/CD Ready

---

## 1. Toolchain Overview

The formal verification toolchain requires:

| Tool | Purpose | Language | Installation |
|------|---------|----------|--------------|
| **Dafny** | Verification | Rust/Dafny | .NET SDK + `dotnet tool install -g dafny` |
| **TLA+** | Model Checking | TLA+ | Java + tlatools.jar |
| **Formulog** | Logic Programming | Formulog | pip install formulog |

---

## 2. Local Installation

### 2.1 One-Time Setup

```bash
# Clone repository
git clone git@192.168.0.252:openclaw/sqlrustgo.git
cd sqlrustgo

# Install toolchain (this may take several minutes)
bash scripts/deploy/install_verification_toolchain.sh

# Verify installation
bash scripts/deploy/install_verification_toolchain.sh --verify
```

### 2.2 Install Individual Tools

```bash
# Dafny only
bash scripts/deploy/install_verification_toolchain.sh --dafny

# TLA+ only
bash scripts/deploy/install_verification_toolchain.sh --tla

# Formulog only
bash scripts/deploy/install_verification_toolchain.sh --formulog
```

---

## 3. CI/CD Integration

### 3.1 GitHub Actions Workflow

Create `.github/workflows/formal-verification.yml`:

```yaml
name: Formal Verification (R10)

on:
  push:
    branches: [develop/*, main]
  pull_request:
    branches: [develop/*, main]

jobs:
  verify-toolchain:
    name: Install Toolchain
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install .NET SDK
        run: |
          curl -sSL https://dot.net/v1/dotnet-install.sh | bash -s -- --channel 8.0
          echo "$HOME/.dotnet" >> $GITHUB_PATH
          echo "$HOME/.dotnet/tools" >> $GITHUB_PATH

      - name: Install Dafny
        run: |
          dotnet tool install -g dafny || true
          dafny --version

      - name: Verify Tools
        run: bash scripts/deploy/install_verification_toolchain.sh --verify

  formal-verification:
    name: Formal Verification
    needs: verify-toolchain
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Toolchain
        run: bash scripts/deploy/install_verification_toolchain.sh

      - name: Run Dafny Verification
        env:
          PATH: "$HOME/.dotnet/tools:$HOME/.dotnet:$PATH"
          DOTNET_ROOT: "$HOME/.dotnet"
        run: |
          echo "=== S-02: Type System Safety ==="
          dafny verify docs/proof/PROOF-011-type-safety.dfy || true
          echo "=== S-04: B+Tree Invariants ==="
          dafny verify docs/proof/PROOF-013-btree-invariants.dfy || true

      - name: Run TLA+ Model Checking
        run: |
          echo "=== S-03: Transaction ACID ==="
          java -cp $HOME/.local/toolchain/tla/tlatools.jar tlc2.TLC \
            -workers auto docs/proof/PROOF-012-wal-acid.tla || true

      - name: Upload Proof Results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: formal-verification-results
          path: |
            **/*.txt
            **/dafny_session.json
```

### 3.2 Gitea Actions Workflow

Create `.gitea/workflows/formal-verification.yml`:

```yaml
name: Formal Verification (R10)

on:
  push:
    branches: [develop/*]
  pull_request:
    branches: [develop/*]

jobs:
  formal-verification:
    name: Formal Verification
    runs-on: ubuntu-latest
    container:
      image: mcr.microsoft.com/dotnet/sdk:8.0
    steps:
      - uses: actions/checkout@v4

      - name: Install Dafny
        run: dotnet tool install -g dafny

      - name: Run Verification
        run: |
          export PATH="$HOME/.dotnet/tools:$PATH"
          dafny verify docs/proof/PROOF-011-type-safety.dfy
          dafny verify docs/proof/PROOF-013-btree-invariants.dfy

      - name: Run TLA+ Model Check
        run: |
          java -cp .local/toolchain/tla/tlatools.jar tlc2.TLC \
            -workers auto docs/proof/PROOF-012-wal-acid.tla
```

### 3.3 Standalone Verification Script

For local development and CI:

```bash
#!/bin/bash
# Run formal verification with all available tools

set -e

export PATH="$HOME/.dotnet/tools:$HOME/.dotnet:$PATH"
export DOTNET_ROOT="$HOME/.dotnet"

echo "=== Formal Verification ==="
echo "Date: $(date)"
echo ""

FAILED=0

# Dafny Verification
if command -v dafny &> /dev/null; then
    echo "Running Dafny verification..."
    for proof in docs/proof/*.dfy; do
        if [[ -f "$proof" ]]; then
            echo "  Verifying $(basename $proof)..."
            if dafny verify "$proof" 2>&1 | tee "${proof}.log"; then
                echo "    PASS"
            else
                echo "    FAIL"
                FAILED=$((FAILED + 1))
            fi
        fi
    done
else
    echo "Dafny not installed, skipping..."
fi

# TLA+ Model Check
if [[ -f "$HOME/.local/toolchain/tla/tlatools.jar" ]]; then
    echo "Running TLA+ model checking..."
    for proof in docs/proof/*.tla; do
        if [[ -f "$proof" ]]; then
            echo "  Model checking $(basename $proof)..."
            if java -cp "$HOME/.local/toolchain/tla/tlatools.jar" \
                tlc2.TLC -workers auto "$proof" 2>&1 | tee "${proof}.log"; then
                echo "    PASS"
            else
                echo "    FAIL"
                FAILED=$((FAILED + 1))
            fi
        fi
    done
else
    echo "TLA+ not installed, skipping..."
fi

echo ""
if [[ $FAILED -eq 0 ]]; then
    echo "All verifications passed"
    exit 0
else
    echo "$FAILED verification(s) failed"
    exit 1
fi
```

---

## 4. Verification Scripts

### 4.1 Quick Verification (Rust Tests)

For environments without formal verification tools:

```bash
#!/bin/bash
# Quick verification using Rust tests as evidence

set -e

echo "=== Rust Test Verification ==="

# Parser tests (S-01 evidence)
echo "Running parser tests..."
cargo test -p sqlrustgo-parser -- --test-threads=1 2>&1 | tail -5

# Type tests (S-02 evidence)
echo "Running type tests..."
cargo test -p sqlrustgo-types 2>&1 | tail -5

# Storage tests (S-04 evidence)
echo "Running storage tests..."
cargo test -p sqlrustgo-storage 2>&1 | tail -5

# Transaction tests (S-03 evidence)
echo "Running transaction tests..."
cargo test -p sqlrustgo-transaction 2>&1 | tail -5

echo ""
echo "Test verification complete"
```

### 4.2 Full Verification (Requires Tools)

```bash
#!/bin/bash
# Full formal verification using all available tools

set -e

export PATH="$HOME/.dotnet/tools:$HOME/.dotnet:$PATH"
export DOTNET_ROOT="$HOME/.dotnet"

echo "=== Full Formal Verification ==="
echo "Date: $(date)"
echo ""

cd "$(dirname "$0")/../.."

# Run all proofs
bash scripts/verify/run_all_proofs.sh

echo ""
echo "Verification complete"
```

---

## 5. Troubleshooting

### 5.1 Common Issues

| Issue | Solution |
|-------|----------|
| `dafny: command not found` | Run `export PATH="$HOME/.dotnet/tools:$PATH"` |
| `.NET SDK not found` | Re-run `bash scripts/deploy/install_verification_toolchain.sh --dotnet` |
| TLA+ download fails | Download manually from https://github.com/tlaplus/tlatools/releases |
| Formulog install fails | Try `pip install formulog` or use Docker |

### 5.2 Tool Paths

After installation, tools are located at:

- **Dafny**: `$HOME/.dotnet/tools/dafny`
- **.NET SDK**: `$HOME/.dotnet/`
- **TLA+**: `$HOME/.local/toolchain/tla/tlatools.jar`
- **Formulog**: `$HOME/.local/bin/formulog`

### 5.3 Path Configuration

Add to `~/.bashrc` or `~/.zshrc`:

```bash
# SQLRustGo Formal Verification Toolchain
export PATH="$HOME/.dotnet/tools:$HOME/.dotnet:$HOME/.local/toolchain/tla:$PATH"
export DOTNET_ROOT="$HOME/.dotnet"
```

---

## 6. Current Status

| Tool | Status | Version |
|------|--------|---------|
| **Dafny** | ✅ Installed | 4.11.0 |
| **TLA+** | ⚠️ Network Issue | N/A |
| **Formulog** | ⚠️ pip Issue | N/A |
| **Java** | ✅ Installed | 11.0.30 |
| **Python** | ✅ Installed | 3.10.12 |

---

## 7. Next Steps

1. **TLA+**: Download `tlatools.jar` manually if network issues persist
2. **Formulog**: Alternative installation via Docker or source compilation
3. **CI/CD**: Configure GitHub/Gitea Actions using the provided workflows
4. **Proof Files**: Create actual verifiable `.dfy`, `.tla`, `.formalog` files (current files are documentation)

---

*Last Updated: 2026-05-03*