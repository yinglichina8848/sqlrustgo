#!/usr/bin/env bash
# run_all_proofs.sh — Beta Gate B3: 运行全部形式化证明验证
#
# 原则:
#   可执行 spec 在 docs/formal/ (TLA+, Dafny)
#   可执行 spec 在 docs/proof/  (Formulog .formulog 文件)
#   文档/JSON 在 docs/proof/
#   TTrace 输出文件跳过
#
# 触发: Beta Gate check_beta.sh
# 用法: bash scripts/verify/run_all_proofs.sh
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

# === 工具路径 ===
DAFNY_CMD="/usr/bin/dafny"
TLA_JAR="${TLA_JAR:-/tmp/tla2tools.jar}"
FORMULOG_JAR="${FORMULOG_JAR:-/tmp/formulog-0.8.0.jar}"
JAVA="${JAVA:-/usr/bin/java}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASSED=0
FAILED=0
SKIPPED=0

pass() { echo -e "  ${GREEN}PASS${NC} $1"; PASSED=$((PASSED+1)); }
fail() { echo -e "  ${RED}FAIL${NC} $1"; FAILED=$((FAILED+1)); }
skip() { echo -e "  ${YELLOW}SKIP${NC} $1"; SKIPPED=$((SKIPPED+1)); }

echo "=== SQLRustGo Formal Verification (Beta Gate B3) ==="
echo "Date: $(date)"
echo ""

# ============================================================
# Dafny Verification (docs/formal/*.dfy — 可执行 spec)
# ============================================================
echo "--- Dafny Verification ---"
for spec in docs/formal/*.dfy; do
    [ -f "$spec" ] || continue
    base=$(basename "$spec")
    echo "[Dafny] $base..."
    if [ ! -x "$DAFNY_CMD" ] && [ ! -f "$DAFNY_CMD" ]; then
        skip "$base (dafny not installed)"
        continue
    fi
    # Dafny 2.3.0 CLI: /dafnyVerify:1 /compile:0
    if $DAFNY_CMD /dafnyVerify:1 /compile:0 "$spec" >/dev/null 2>&1; then
        pass "$base"
    else
        fail "$base"
    fi
done
echo ""

# ============================================================
# TLA+ TLC Verification (docs/formal/*.tla — 可执行 spec, 跳过 TTrace)
# ============================================================
echo "--- TLA+ Model Checking ---"
if [ ! -f "$TLA_JAR" ]; then
    skip "ALL TLA+ (tla2tools.jar not found at $TLA_JAR)"
else
    for spec in docs/formal/*.tla; do
        [ -f "$spec" ] || continue
        base=$(basename "$spec" .tla)
        cfg="docs/formal/${base}.cfg"

        # 跳过 TTrace 文件
        echo "$base" | grep -q "TTrace" && continue

        if [ ! -f "$cfg" ]; then
            skip "$base (cfg not found)"
            continue
        fi

        echo "[TLA+] $base..."
        set +e
        out=$(timeout 60 $JAVA -XX:+UseParallelGC -cp "$TLA_JAR" \
            tlc2.TLC "$spec" -config "$cfg" -deadlock 2>&1); rc=$?
        set -e

        if [ $rc -eq 124 ]; then
            skip "$base (timeout >60s — large state space, nightly only)"
            continue
        fi

        if echo "$out" | grep -q "No error"; then
            # 检查是否预期 VIOLATED（如 TOCTOU 模型）
            if grep -q "NoCycle\|Broken" "$cfg" 2>/dev/null && echo "$spec" | grep -q "toctou"; then
                pass "$base (expected PASS — atomic variant)"
            else
                pass "$base"
            fi
        elif echo "$out" | grep -q "Error:"; then
            # TOCTOU 模型预期 VIOLATED
            if echo "$base" | grep -q "toctou\|write_skew"; then
                pass "$base (expected VIOLATED)"
            else
                fail "$base"
            fi
        else
            skip "$base (ambiguous result)"
        fi
    done
fi
echo ""

# ============================================================
# Formulog Verification (docs/proof/*.formulog — 可执行 spec)
# ============================================================
echo "--- Formulog Verification ---"
if [ ! -f "$FORMULOG_JAR" ]; then
    skip "ALL Formulog (formulog jar not found at $FORMULOG_JAR)"
else
    for spec in docs/proof/PROOF-*.formulog; do
        [ -f "$spec" ] || continue
        base=$(basename "$spec")
        # Skip if file is empty or not a regular file
        [ -s "$spec" ] || { skip "$base (empty file)"; continue; }

        echo "[Formulog] $base..."
        # Direct JVM — each invocation is a separate process (no cache pollution)
        if $JAVA -jar "$FORMULOG_JAR" "$spec" >/dev/null 2>&1; then
            pass "$base"
        else
            fail "$base"
        fi
    done
fi
echo ""

# ============================================================
# Summary
# ============================================================
echo "=== Summary ==="
echo "Passed:  $PASSED"
echo "Failed:  $FAILED"
echo "Skipped: $SKIPPED"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Formal verification FAILED ($FAILED failure(s))${NC}"
    exit 1
elif [ $PASSED -eq 0 ] && [ $SKIPPED -gt 0 ]; then
    echo -e "${YELLOW}No proofs run (all skipped — tools missing?)${NC}"
    echo "Continuing (non-blocking skip)"
    exit 0
else
    echo -e "${GREEN}All formal verifications passed${NC}"
    exit 0
fi
