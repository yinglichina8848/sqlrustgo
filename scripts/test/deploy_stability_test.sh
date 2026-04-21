#!/bin/bash

set -e

HOURS=72
BRANCH="release/v2.6.0"
TEST_TYPE="all"
OUTPUT_DIR="./test_results"
AUTO_START_MONITOR=1

show_help() {
    cat << EOF
SQLRustGo 长稳测试部署脚本

用法:
    $0 [选项]

选项:
    --hours HOURS       测试时长小时数 (默认: 72)
    --branch BRANCH     Git 分支 (默认: release/v2.6.0)
    --test-type TYPE   测试类型: long_run, qps, all (默认: all)
    --output-dir DIR   输出目录 (默认: ./test_results)
    --no-monitor       不自动启动监控
    --skip-build       跳过构建步骤
    -h, --help         显示帮助信息

示例:
    $0 --hours 72
    $0 --hours 24 --branch develop/v2.6.0
    $0 --test-type long_run --output-dir /tmp/test

EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --hours)
                HOURS="$2"
                shift 2
                ;;
            --branch)
                BRANCH="$2"
                shift 2
                ;;
            --test-type)
                TEST_TYPE="$2"
                shift 2
                ;;
            --output-dir)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            --no-monitor)
                AUTO_START_MONITOR=0
                shift
                ;;
            --skip-build)
                SKIP_BUILD=1
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                echo "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

check_env() {
    log "检查环境..."

    if ! command -v cargo &> /dev/null; then
        log "错误: Cargo 未安装"
        log "请先安装 Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    local rust_version=$(rustc --version | awk '{print $2}')
    log "Rust 版本: $rust_version"

    if ! command -v tmux &> /dev/null; then
        log "警告: tmux 未安装，测试将在前台运行"
        AUTO_START_MONITOR=0
    fi
}

setup_dir() {
    log "设置目录..."

    mkdir -p "$OUTPUT_DIR"

    export OUTPUT_DIR
    log "输出目录: $OUTPUT_DIR"
}

clone_or_update() {
    log "准备代码..."

    if [[ -d ".git" ]]; then
        log "当前目录已是 Git 仓库"

        local current_branch=$(git rev-parse --abbrev-ref HEAD)
        if [[ "$current_branch" != "$BRANCH" ]]; then
            log "切换分支: $current_branch -> $BRANCH"
            git fetch origin
            git checkout "$BRANCH"
        fi
    else
        log "克隆仓库..."
        git clone https://github.com/minzuuniversity/sqlrustgo.git .
        git checkout "$BRANCH"
    fi

    log "当前分支: $(git rev-parse --abbrev-ref HEAD)"
    log "提交: $(git rev-parse HEAD)"
}

build() {
    if [[ "$SKIP_BUILD" == "1" ]]; then
        log "跳过构建步骤"
        return
    fi

    log "构建项目 (Release 模式)..."

    cargo build --release

    log "构建完成"
}

run_test() {
    log "启动测试..."

    local test_cmd="cargo test --test regression_test -- --test-threads=1 --ignored"

    case "$TEST_TYPE" in
        long_run)
            test_cmd="cargo test --test regression_test long_run_stability_test -- --test-threads=1 --ignored"
            ;;
        qps)
            test_cmd="cargo test --test regression_test qps_benchmark_test -- --test-threads=1 --ignored"
            ;;
        all)
            test_cmd="cargo test --test regression_test -- --test-threads=1 --ignored"
            ;;
    esac

    log "测试命令: $test_cmd"

    cd "$OUTPUT_DIR"

    echo "$test_cmd" > test_command.sh

    $test_cmd 2>&1 | tee test_output.log

    local exit_code=${PIPESTATUS[0]}

    log "测试结束，退出码: $exit_code"

    return $exit_code
}

start_monitor() {
    if [[ "$AUTO_START_MONITOR" != "1" ]]; then
        return
    fi

    log "启动监控..."

    local monitor_script="$(dirname "$0")/monitor_stability_test.sh"

    if [[ ! -f "$monitor_script" ]]; then
        monitor_script="./scripts/test/monitor_stability_test.sh"
    fi

    if [[ -f "$monitor_script" ]]; then
        chmod +x "$monitor_script"

        tmux new-session -d -s monitor "bash $monitor_script --output-dir $OUTPUT_DIR --duration $HOURS"

        log "监控已启动 (tmux session: monitor)"
    else
        log "警告: 监控脚本未找到"
    fi
}

start_collector() {
    if [[ "$AUTO_START_MONITOR" != "1" ]]; then
        return
    fi

    log "结果收集器将在测试完成后运行"

    cat << 'EOF' > "$OUTPUT_DIR/run_collector.sh"
#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
bash scripts/test/collect_stability_results.sh --input-dir .
EOF

    chmod +x "$OUTPUT_DIR/run_collector.sh"
}

show_summary() {
    echo ""
    echo "================================"
    echo "  SQLRustGo 长稳测试已启动"
    echo "================================"
    echo ""
    echo "测试参数:"
    echo "  - 时长: ${HOURS} 小时"
    echo "  - 分支: $BRANCH"
    echo "  - 测试类型: $TEST_TYPE"
    echo "  - 输出目录: $OUTPUT_DIR"
    echo ""
    echo "监控命令:"
    echo "  bash scripts/test/monitor_stability_test.sh --output-dir $OUTPUT_DIR"
    echo ""
    echo "结果收集:"
    echo "  bash $OUTPUT_DIR/run_collector.sh"
    echo ""
    echo "Tmux 会话:"
    echo "  - 测试: stability_test"
    echo "  - 监控: monitor"
    echo ""
    echo "注意事项:"
    echo "  - 测试将运行 ${HOURS} 小时"
    echo "  - 监控数据每分钟记录一次"
    echo "  - 测试完成后自动收集结果"
    echo "================================"
}

main() {
    parse_args "$@"

    log "SQLRustGo 长稳测试部署"
    log "========================"

    check_env
    setup_dir
    clone_or_update
    build
    start_monitor
    start_collector

    show_summary

    log "开始测试..."

    run_test

    local exit_code=$?

    log "测试完成，退出码: $exit_code"

    if [[ -f "$OUTPUT_DIR/run_collector.sh" ]]; then
        log "运行结果收集器..."
        bash "$OUTPUT_DIR/run_collector.sh"
    fi

    exit $exit_code
}

main "$@"
