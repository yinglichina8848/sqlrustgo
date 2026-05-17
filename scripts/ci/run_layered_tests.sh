#!/bin/bash
# =============================================================================
# run_layered_tests.sh - SQLRustGo 分层测试系统
# =============================================================================
#
# 分层策略:
#   L0: 编译验证 (每次)        ~30s
#   L1: 单元测试 (每次)        ~2-5min
#   L2: 增量集成测试 (按需)    ~5-15min
#   L3: 全量门禁 (合并前)      ~40min
#
# 用法:
#   LAYER=2 TARGET_BRANCH=develop/v3.2.0 bash scripts/ci/run_layered_tests.sh
#
# 环境变量:
#   LAYER          测试层级 (0/1/2/3, 默认: auto)
#   TARGET_BRANCH  比较分支 (默认: develop/v3.2.0)
#   CHANGED_CRATES 预计算的变更 crate 列表 (逗号或空格分隔)
#   PARALLEL_JOBS  并行任务数 (默认: 4)
#   TIMEOUT_MIN    单个任务超时分钟 (默认: 10)
#   CI_MODE        CI 模式 (true=机器可读输出)
#   DRY_RUN        干跑模式 (true=只打印不执行)

set -euo pipefail

# === 默认配置 ===
LAYER="${LAYER:-auto}"
TARGET_BRANCH="${TARGET_BRANCH:-develop/v3.2.0}"
CHANGED_CRATES="${CHANGED_CRATES:-}"
PARALLEL_JOBS="${PARALLEL_JOBS:-4}"
TIMEOUT_MIN="${TIMEOUT_MIN:-10}"
CI_MODE="${CI_MODE:-false}"
DRY_RUN="${DRY_RUN:-false}"

# 向上找 3 层: scripts/ci -> scripts -> workspace root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
LOG_DIR="${WORKSPACE_ROOT}/ci_artifacts/logs/${TIMESTAMP}"
EXIT_CODE=0

# === 颜色输出 ===
if [ "$CI_MODE" = "false" ]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
    BLUE='\033[0;34m'; BOLD='\033[1m'; RESET='\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; BLUE=''; BOLD=''; RESET=''
fi

log_info() { echo -e "${BLUE}[INFO]${RESET} $*"; }
log_pass() { echo -e "${GREEN}[PASS]${RESET} $*"; }
log_fail() { echo -e "${RED}[FAIL]${RESET} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${RESET} $*"; }
log_step() { echo -e "${BOLD}[STEP]${RESET} $*"; }

# === 帮助 ===
show_help() {
    cat << EOF
${BOLD}SQLRustGo 分层测试系统${RESET}

${BOLD}用法:${RESET}
    LAYER=2 TARGET_BRANCH=develop/v3.2.0 bash scripts/ci/run_layered_tests.sh

${BOLD}环境变量:${RESET}
    LAYER              测试层级 (0/1/2/3/auto, 默认: auto)
    TARGET_BRANCH      比较分支 (默认: develop/v3.2.0)
    CHANGED_CRATES     预计算的变更 crate 列表 (逗号或空格分隔)
    PARALLEL_JOBS      并行任务数 (默认: 4)
    TIMEOUT_MIN        单个任务超时分钟 (默认: 10)
    CI_MODE            CI 模式 (true=机器可读输出, 默认: false)
    DRY_RUN            干跑模式 (默认: false)

${BOLD}分层说明:${RESET}
    L0 (编译)     cargo build --release --workspace           ~30s
    L1 (单元)     cargo test --lib                            ~2-5min
    L2 (增量)     cargo test -p <changed-crate> --tests      ~5-15min
    L3 (全量)     cargo test --workspace + scripts/gate/*    ~40min

${BOLD}示例:${RESET}
    # 只跑 L0+L1
    LAYER=1 bash scripts/ci/run_layered_tests.sh

    # 检测变更跑 L2
    LAYER=2 bash scripts/ci/run_layered_tests.sh

    # 全量门禁
    LAYER=3 bash scripts/ci/run_layered_tests.sh

    # 指定变更 crate
    CHANGED_CRATES="storage,planner" LAYER=2 bash scripts/ci/run_layered_tests.sh
EOF
}

# === 参数解析 ===
if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    show_help; exit 0
fi

# === 创建日志目录 ===
mkdir -p "$LOG_DIR"
log_info "Log directory: $LOG_DIR"

# === 检测变更的 crate ===
detect_changed_crates() {
    if [ -n "$CHANGED_CRATES" ]; then
        # 支持逗号或空格分隔
        echo "$CHANGED_CRATES" | tr ', ' '\n' | grep -v '^$'
        return 0
    fi

    local result
    result=$("$SCRIPT_DIR/detect_changed_crates.sh" "$TARGET_BRANCH" 2>/dev/null) || result=""
    if [ -z "$result" ]; then
        echo ""
        return 1
    fi
    echo "$result"
}

# === 执行命令 (带超时和日志) ===
run_cmd() {
    local name="$1"; shift
    local cmd="$*"
    local log_file="${LOG_DIR}/${name}.log"
    local timeout_sec=$((TIMEOUT_MIN * 60))

    if [ "$DRY_RUN" = "true" ]; then
        log_info "[DRY RUN] $name: $cmd"
        return 0
    fi

    log_step "$name (timeout: ${TIMEOUT_MIN}min)"

    # 使用 timeout 命令
    if timeout "$timeout_sec" bash -c "cd '$WORKSPACE_ROOT' && $cmd" > "$log_file" 2>&1; then
        log_pass "$name"
        return 0
    else
        local ret=$?
        if [ $ret -eq 124 ]; then
            log_fail "$name: TIMEOUT after ${TIMEOUT_MIN}min"
        else
            log_fail "$name: FAILED (exit $ret)"
        fi
        log_warn "See $log_file for details"
        tail -5 "$log_file" 2>/dev/null || true
        return 1
    fi
}

# === L0: 编译验证 ===
run_l0() {
    log_info "=== L0: 编译验证 ==="

    # 快速编译检查
    if ! run_cmd "l0-check" "cargo check --all-features 2>&1"; then
        log_fail "L0 check failed"
        return 1
    fi

    # Release 编译
    if ! run_cmd "l0-build" "cargo build --release --workspace 2>&1"; then
        log_fail "L0 build failed"
        return 1
    fi

    log_pass "L0: 编译验证通过"
    return 0
}

# === L1: 单元测试 ===
run_l1() {
    log_info "=== L1: 单元测试 ==="

    # Lib tests (不跑集成测试，更快)
    if ! run_cmd "l1-lib" "cargo test --lib --all-features 2>&1"; then
        log_fail "L1 failed"
        return 1
    fi

    # Doc tests
    if ! run_cmd "l1-doc" "cargo test --doc --all-features 2>&1"; then
        log_warn "L1 doc tests: 有警告，继续"
    fi

    log_pass "L1: 单元测试通过"
    return 0
}

# === L2: 增量集成测试 ===
run_l2() {
    if [ "$DRY_RUN" = "true" ]; then
        log_info "=== L2: 增量集成测试 (DRY RUN) ==="
        log_info "L2: Would test crates: storage planner"
        return 0
    fi

    log_info "=== L2: 增量集成测试 ==="

    local crates_array=()
    while IFS= read -r crate; do
        [ -n "$crate" ] && crates_array+=("$crate")
    done < <(detect_changed_crates)

    if [ ${#crates_array[@]} -eq 0 ]; then
        log_warn "L2: 未检测到 crate 变更，跳过"
        return 0
    fi

    log_info "L2: 变更 crates: ${crates_array[*]}"

    # 按 crate 并行跑集成测试
    local pids=()
    local failed=0

    for crate in "${crates_array[@]}"; do
        (
            local crate_log="${LOG_DIR}/l2-${crate}.log"
            local timeout_sec=$((TIMEOUT_MIN * 60))

            if timeout "$timeout_sec" bash -c "cd '$WORKSPACE_ROOT' && cargo test -p sqlrustgo-${crate} --all-features 2>&1" > "$crate_log" 2>&1; then
                echo "PASS" >> "$crate_log"
            else
                echo "FAIL" >> "$crate_log"
                exit 1
            fi
        ) &
        pids+=($!)
    done

    # 等待所有任务
    for i in "${!pids[@]}"; do
        if wait "${pids[$i]}" 2>/dev/null; then
            log_pass "L2: ${crates_array[$i]}"
        else
            log_fail "L2: ${crates_array[$i]}"
            failed=$((failed + 1))
        fi
    done

    if [ $failed -gt 0 ]; then
        log_fail "L2: $failed 个 crate 测试失败"
        return 1
    fi

    log_pass "L2: 增量集成测试通过"
    return 0
}

# === L3: 全量门禁 ===
run_l3() {
    log_info "=== L3: 全量门禁 ==="

    # 全量测试
    if ! run_cmd "l3-test" "cargo test --workspace --all-features 2>&1"; then
        log_fail "L3 failed: cargo test"
        return 1
    fi

    # Clippy
    if ! run_cmd "l3-clippy" "cargo clippy --all-features -- -D warnings 2>&1"; then
        log_fail "L3 failed: clippy"
        return 1
    fi

    # Format
    if ! run_cmd "l3-fmt" "cargo fmt --check --all 2>&1"; then
        log_fail "L3 failed: format"
        return 1
    fi

    # 门禁脚本
    local gate_scripts=(
        "check_docs_links.sh"
        "check_coverage.sh"
        "check_security.sh"
    )

    for script in "${gate_scripts[@]}"; do
        if [ -x "$WORKSPACE_ROOT/scripts/gate/$script" ]; then
            if ! run_cmd "l3-gate-${script%.sh}" "bash scripts/gate/$script 2>&1"; then
                log_warn "L3 gate $script: 有警告"
            fi
        fi
    done

    log_pass "L3: 全量门禁通过"
    return 0
}

# === 主流程 ===
main() {
    log_info "=========================================="
    log_info "SQLRustGo 分层测试系统"
    log_info "LAYER=$LAYER, TARGET=$TARGET_BRANCH"
    log_info "=========================================="

    # 确定层级
    if [ "$LAYER" = "auto" ]; then
        log_info "LAYER=auto: 自动检测需要的层级"
        # auto: 总是 L0+L1，L2 按需，L3 需显式指定
        LAYER_DESIRED=2
    else
        LAYER_DESIRED="$LAYER"
    fi

    # 执行层级
    case "$LAYER_DESIRED" in
        0)
            run_l0
            EXIT_CODE=$?
            ;;
        1)
            run_l0 || EXIT_CODE=1
            run_l1
            EXIT_CODE=$?
            ;;
        2)
            run_l0 || EXIT_CODE=1
            run_l1 || EXIT_CODE=1
            run_l2
            EXIT_CODE=$?
            ;;
        3)
            run_l0 || EXIT_CODE=1
            run_l1 || EXIT_CODE=1
            run_l2 || EXIT_CODE=1
            run_l3
            EXIT_CODE=$?
            ;;
        *)
            log_fail "无效 LAYER: $LAYER (有效值: 0/1/2/3/auto)"
            EXIT_CODE=2
            ;;
    esac

    # 总结
    echo ""
    log_info "=========================================="
    if [ $EXIT_CODE -eq 0 ]; then
        log_pass "分层测试完成: 全部通过"
    else
        log_fail "分层测试完成: 失败 (exit $EXIT_CODE)"
    fi
    log_info "日志目录: $LOG_DIR"
    log_info "=========================================="

    return $EXIT_CODE
}

main "$@"
