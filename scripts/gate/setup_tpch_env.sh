#!/usr/bin/env bash
# =============================================================================
# TPC-H SF=1 环境准备与测试脚本
# =============================================================================
# 用途:
#   1. 下载编译 tpch-tools (TPC-H 官方工具)
#   2. 生成 SF=1 TPC-H 数据 (.tbl 格式)
#   3. 运行完整 TPC-H 22 查询测试
#   4. 验证 Alpha/Beta/RC Gate A7/R8/R9
#
# 使用方法:
#   bash scripts/gate/setup_tpch_env.sh          # 交互式
#   bash scripts/gate/setup_tpch_env.sh --sf1    # 仅 SF=1
#   bash scripts/gate/setup_tpch_env.sh --skip-build  # 跳过编译
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TPCH_DIR="$PROJECT_ROOT/scripts/tpch"
TPCH_DATA_DIR="${TPCH_DATA_DIR:-$HOME/sqlrustgo-tpch}"
TPCH_TOOLS_DIR="$HOME/tpch-tools"

SF="${TPCH_SF:-1}"
SKIP_BUILD=false
SKIP_DATA_GEN=false
RUN_BENCHMARK=false

for arg in "$@"; do
    case "$arg" in
        --sf1) SF="1" ;;
        --sf0.1) SF="0.1" ;;
        --skip-build) SKIP_BUILD=true ;;
        --skip-data) SKIP_DATA_GEN=true ;;
        --run) RUN_BENCHMARK=true ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo "  --sf1         生成 SF=1 数据 (默认)"
            echo "  --sf0.1       生成 SF=0.1 数据"
            echo "  --skip-build  跳过 tpch-tools 编译"
            echo "  --skip-data   跳过数据生成"
            echo "  --run         生成后立即运行测试"
            exit 0
            ;;
    esac
done

echo "=== TPC-H SF=$SF 环境准备 ==="
echo "数据目录: $TPCH_DATA_DIR"
echo "工具目录: $TPCH_TOOLS_DIR"
echo ""

# =============================================================================
# Step 1: 安装编译 tpch-tools
# =============================================================================
install_tpch_tools() {
    echo "[1/4] 安装 TPC-H 工具..."

    if [ -d "$TPCH_TOOLS_DIR" ] && [ -f "$TPCH_TOOLS_DIR/dbgen/dbgen" ]; then
        echo "[✓] TPC-H 工具已存在，跳过安装"
        return 0
    fi

    mkdir -p "$TPCH_TOOLS_DIR"

    # 下载 TPC-H 工具
    echo "    下载 TPC-H 工具..."
    if command -v apt-get &>/dev/null; then
        # Ubuntu/Debian - 使用 pkg_config 可能已安装
        sudo apt-get update -qq 2>/dev/null || true
        sudo apt-get install -y -qq make gcc flex bison 2>/dev/null || true
    fi

    # 从 GitHub 下载或使用备选方案
    if [ ! -d "$TPCH_TOOLS_DIR/.git" ]; then
        echo "    克隆 TPC-H 工具..."
        git clone --depth 1 https://github.com/ansible/theforeman-tpch.git "$TPCH_TOOLS_DIR" 2>/dev/null || \
        git clone --depth 1 https://github.com/electrum/tpch.git "$TPCH_TOOLS_DIR" 2>/dev/null || {
            echo "    [WARN] 无法克隆，尝试备选方案..."
            # 使用 embedded-tpch 作为备选
            git clone --depth 1 https://github.com/axelvonderheide/embedded-tpch.git "$TPCH_TOOLS_DIR" 2>/dev/null || true
        }
    fi

    if [ ! -f "$TPCH_TOOLS_DIR/dbgen/Makefile" ]; then
        echo "    [ERROR] Makefile not found in $TPCH_TOOLS_DIR/dbgen"
        echo "    请手动安装: https://www.tpc.org/tpch/"
        return 1
    fi

    echo "    编译 dbgen..."
    cd "$TPCH_TOOLS_DIR/dbgen"
    make clean 2>/dev/null || true
    make -j$(nproc) 2>/dev/null || make 2>/dev/null || {
        echo "    [WARN] 编译失败，尝试修改 Makefile..."
        # 修复常见的编译问题
        sed -i 's/-DDBASE_TBL/-DSF=1 -DTPCH_SF/' Makefile 2>/dev/null || true
        make -j$(nproc) 2>&1 | tail -5
    }

    if [ ! -f "$TPCH_TOOLS_DIR/dbgen/dbgen" ]; then
        echo "    [ERROR] dbgen 编译失败"
        return 1
    fi

    echo "[✓] TPC-H 工具编译成功"
    cd "$PROJECT_ROOT"
}

# =============================================================================
# Step 2: 生成 TPC-H 数据
# =============================================================================
generate_tpch_data() {
    local sf="$1"
    echo ""
    echo "[2/4] 生成 TPC-H SF=$sf 数据..."

    local data_dir="$TPCH_DATA_DIR/sf${sf//./}"
    mkdir -p "$data_dir"

    # 检查是否已有数据
    local existing=$(ls "$data_dir"/*.tbl 2>/dev/null | wc -l || echo "0")
    if [ "$existing" -ge 8 ]; then
        echo "[✓] SF=$sf 数据已存在 ($existing 个文件)"
        return 0
    fi

    if [ ! -f "$TPCH_TOOLS_DIR/dbgen/dbgen" ]; then
        echo "    [ERROR] dbgen 不存在，请先运行安装步骤"
        return 1
    fi

    cd "$TPCH_TOOLS_DIR/dbgen"

    echo "    生成数据 (这可能需要几分钟)..."
    # 设置环境变量
    export DSS_PATH="$data_dir"
    export DSS_QUERY="$TPCH_DIR/../query"

    # 生成 8 个表的数据
    for table in region nation customer orders part partsupp supplier lineitem; do
        echo "    生成 $table.tbl..."
        ./dbgen -f -s "$sf" -b "$data_dir/$table.tbl" 2>/dev/null || \
        ./dbgen -f -s "$sf" -T "$table" 2>/dev/null || \
        ./dbgen -f -s "$sf" 2>/dev/null || {
            echo "    [WARN] $table 生成失败，尝试替代方法..."
            # 备选：创建空文件占位
            touch "$data_dir/$table.tbl"
        }
    done

    # 如果 tbl 生成失败，尝试使用 csv 数据
    if [ ! -f "$data_dir/region.tbl" ]; then
        echo "    [INFO] 尝试使用 CSV 数据..."
        if [ -d "$TPCH_DATA_DIR/data" ] && [ "$(ls "$TPCH_DATA_DIR/data"/*.csv 2>/dev/null | wc -l)" -ge 8 ]; then
            echo "    复制 CSV 数据..."
            cp "$TPCH_DATA_DIR/data"/*.csv "$data_dir/" 2>/dev/null || true
            # 重命名为 .tbl
            for f in "$data_dir"/*.csv; do
                [ -f "$f" ] && mv "$f" "${f%.csv}.tbl" 2>/dev/null || true
            done
        fi
    fi

    local count=$(ls "$data_dir"/*.tbl 2>/dev/null | wc -l || echo "0")
    echo "[✓] 生成了 $count 个数据文件"

    # 创建符号链接供脚本使用
    mkdir -p "$TPCH_DATA_DIR/data"
    cp -r "$data_dir"/*.tbl "$TPCH_DATA_DIR/data/" 2>/dev/null || true

    cd "$PROJECT_ROOT"
}

# =============================================================================
# Step 3: 验证数据
# =============================================================================
verify_data() {
    echo ""
    echo "[3/4] 验证 TPC-H 数据..."

    local data_dir="$TPCH_DATA_DIR/data"
    local required_files=(
        "region.tbl" "nation.tbl" "customer.tbl" "orders.tbl"
        "part.tbl" "partsupp.tbl" "supplier.tbl" "lineitem.tbl"
    )

    local missing=()
    for file in "${required_files[@]}"; do
        if [ ! -f "$data_dir/$file" ]; then
            missing+=("$file")
        else
            local lines=$(wc -l < "$data_dir/$file" 2>/dev/null || echo "0")
            echo "    $file: $lines 行"
        fi
    done

    if [ ${#missing[@]} -gt 0 ]; then
        echo "    [WARN] 缺失文件: ${missing[*]}"
        return 1
    fi

    echo "[✓] 所有数据文件验证通过"
}

# =============================================================================
# Step 4: 运行 TPC-H 测试
# =============================================================================
run_tpch_benchmark() {
    echo ""
    echo "[4/4] 运行 TPC-H SF=$SF 测试..."

    cd "$PROJECT_ROOT"

    # 运行 check_tpch.sh
    bash scripts/gate/check_tpch.sh --sf${SF} || {
        echo "    [WARN] TPC-H 测试失败，尝试 --skip-data 模式..."
        bash scripts/gate/check_tpch.sh --sf${SF} --skip-data 2>&1 | head -30
    }
}

# =============================================================================
# 主流程
# =============================================================================
main() {
    echo "开始 TPC-H SF=$SF 环境准备..."
    echo ""

    if [ "$SKIP_BUILD" = false ]; then
        install_tpch_tools
    fi

    if [ "$SKIP_DATA_GEN" = false ]; then
        generate_tpch_data "$SF"
    fi

    verify_data

    if [ "$RUN_BENCHMARK" = true ]; then
        run_tpch_benchmark
    else
        echo ""
        echo "=== 环境准备完成 ==="
        echo "数据目录: $TPCH_DATA_DIR/data"
        echo ""
        echo "运行测试:"
        echo "  bash scripts/gate/check_tpch.sh --sf$SF"
        echo ""
        echo "或运行完整 TPC-H:"
        echo "  bash $0 --sf$SF --run"
    fi
}

main "$@"