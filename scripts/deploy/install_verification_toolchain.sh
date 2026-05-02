#!/usr/bin/env bash
# =============================================================================
# SQLRustGo Formal Verification Toolchain Deployment Script
# =============================================================================
# This script installs and verifies all tools required for formal verification:
#   - Dafny (Rust verification)
#   - TLA+ (Model checking)
#   - Formulog (Logic programming)
#
# Usage:
#   bash scripts/deploy/install_verification_toolchain.sh [--check-only]
#
# For CI/CD, use:
#   CI=true bash scripts/deploy/install_verification_toolchain.sh
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local}"
TOOLCHAIN_DIR="$INSTALL_DIR/toolchain"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Log functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if running in CI
is_ci() { [[ "${CI:-false}" == "true" ]]; }

# =============================================================================
# Tool: Z3 (SMT Solver for Dafny)
# =============================================================================
install_z3() {
    log_info "Installing Z3 solver..."

    local z3_dir="$HOME/.local/z3"
    local z3_version="z3-4.12.2-x64-glibc-2.35"

    if [[ ! -d "$z3_dir/$z3_version" ]]; then
        log_info "Downloading Z3..."
        mkdir -p "$z3_dir"
        cd "$z3_dir"

        local z3_url="https://github.com/Z3Prover/z3/releases/download/z3-4.12.2/z3-4.12.2-x64-glibc-2.35.zip"

        if curl -sSL "$z3_url" -o z3.zip && unzip -q z3.zip; then
            log_success "Z3 downloaded"
        else
            log_warn "Z3 download failed"
            cd - > /dev/null
            return 1
        fi

        cd - > /dev/null
    fi

    if [[ -f "$z3_dir/$z3_version/bin/z3" ]]; then
        export PATH="$z3_dir/$z3_version/bin:$PATH"
        log_success "Z3 installed"
        return 0
    else
        log_warn "Z3 installation incomplete"
        return 1
    fi
}

# =============================================================================
# Tool: Dafny (.NET Verification)
# =============================================================================
install_dafny() {
    log_info "Installing Dafny..."

    # Install Z3 first (required for Dafny)
    install_z3 || true

    local dotnet_dir="$HOME/.dotnet"
    local dotnet_install="$TOOLCHAIN_DIR/dotnet-install.sh"

    # Check if .NET SDK is installed
    if ! "$dotnet_dir/dotnet" --version &> /dev/null; then
        log_info "Installing .NET SDK..."
        mkdir -p "$TOOLCHAIN_DIR"

        # Download dotnet-install script
        curl -sSL "https://dot.net/v1/dotnet-install.sh" -o "$dotnet_install"
        chmod +x "$dotnet_install"

        # Install .NET SDK
        "$dotnet_install" --channel 8.0 --install-dir "$dotnet_dir"
    fi

    # Add dotnet to PATH for this session
    export PATH="$dotnet_dir:$PATH"
    export DOTNET_ROOT="$dotnet_dir"

    # Check .NET version
    if "$dotnet_dir/dotnet" --version &> /dev/null; then
        log_success ".NET SDK $($dotnet_dir/dotnet --version) installed"
    else
        log_error ".NET SDK installation failed"
        return 1
    fi

    # Install Dafny as .NET tool
    if ! "$dotnet_dir/dotnet tool list -g" 2>/dev/null | grep -q "dafny"; then
        "$dotnet_dir/dotnet tool install -g dafny" 2>/dev/null || true
    fi

    # Add Dafny to PATH
    export PATH="$dotnet_dir/tools:$PATH"

    # Verify Dafny
    if command -v dafny &> /dev/null; then
        log_success "Dafny $(dafny --version | head -1) installed"
        return 0
    else
        log_error "Dafny installation failed"
        return 1
    fi
}

# =============================================================================
# Tool: TLA+ (Model Checking)
# =============================================================================
install_tla() {
    log_info "Installing TLA+..."

    local tla_dir="$TOOLCHAIN_DIR/tla"
    mkdir -p "$tla_dir"

    # Try multiple download sources
    local tla_jar=""
    local download_urls=(
        "https://github.com/tlaplus/tlatools/releases/download/v1.8.0/tlatools-1.8.0.jar"
        "https://github.com/tlaplus/tlatools/releases/download/v1.7.0/tlatools-1.7.0.jar"
    )

    for url in "${download_urls[@]}"; do
        log_info "Trying to download from: $url"
        if curl -sSL "$url" -o "$tla_dir/tlatools.jar" && [[ -s "$tla_dir/tlatools.jar" ]]; then
            tla_jar="$tla_dir/tlatools.jar"
            break
        fi
    done

    # Check Java
    if ! command -v java &> /dev/null; then
        log_error "Java is required for TLA+ but not found"
        return 1
    fi

    if [[ -n "$tla_jar" ]] && [[ -s "$tla_jar" ]]; then
        log_success "TLA+ tools downloaded to $tla_jar"

        # Create wrapper script
        cat > "$tla_dir/tlc" << 'WRAPPER'
#!/usr/bin/env bash
java -cp "$HOME/.local/toolchain/tla/tlatools.jar" tlc2.TLC "$@"
WRAPPER
        chmod +x "$tla_dir/tlc"

        # Add to PATH
        export PATH="$tla_dir:$PATH"

        log_success "TLA+ installed (run with: java -cp $tla_jar tlc2.TLC)"
        return 0
    else
        log_warn "TLA+ download failed - TLA+ model checking will not be available"
        log_info "Manual installation: Download from https://github.com/tlaplus/tlatools/releases"
        return 1
    fi
}

# =============================================================================
# Tool: Formulog (Logic Programming)
# =============================================================================
install_formulog() {
    log_info "Installing Formulog..."

    # Formulog requires Python and pip
    if ! command -v python3 &> /dev/null; then
        log_error "Python3 is required for Formulog but not found"
        return 1
    fi

    # Try to install via pip
    if pip3 install formulog &> /dev/null; then
        if command -v formulog &> /dev/null; then
            log_success "Formulog installed"
            return 0
        fi
    fi

    # Try development installation
    if pip3 install git+https://github.com/ucsd-progsys/formulog.git &> /dev/null; then
        log_success "Formulog installed from source"
        return 0
    fi

    log_warn "Formulog installation failed - Logic programming verification will not be available"
    log_info "Manual installation: pip install formulog"
    return 1
}

# =============================================================================
# Tool: Install .NET SDK (for Dafny)
# =============================================================================
install_dotnet() {
    log_info "Installing .NET SDK..."

    local dotnet_dir="$HOME/.dotnet"
    local dotnet_install="$TOOLCHAIN_DIR/dotnet-install.sh"
    mkdir -p "$TOOLCHAIN_DIR"

    curl -sSL "https://dot.net/v1/dotnet-install.sh" -o "$dotnet_install"
    chmod +x "$dotnet_install"

    "$dotnet_install" --channel 8.0 --install-dir "$dotnet_dir"

    export PATH="$dotnet_dir:$PATH"
    export DOTNET_ROOT="$dotnet_dir"

    if "$dotnet_dir/dotnet" --version &> /dev/null; then
        log_success ".NET SDK $($dotnet_dir/dotnet --version) installed"
        return 0
    else
        log_error ".NET SDK installation failed"
        return 1
    fi
}

# =============================================================================
# Verify All Tools
# =============================================================================
verify_tools() {
    log_info "Verifying installed tools..."

    local all_ok=true

    # Export PATH for this function
    export PATH="$HOME/.dotnet/tools:$HOME/.dotnet:$TOOLCHAIN_DIR/tla:$HOME/.local/toolchain/tla:$PATH"
    export DOTNET_ROOT="$HOME/.dotnet"

    echo ""
    echo "Tool Versions:"
    echo "==============="

    # Z3
    if command -v z3 &> /dev/null; then
        echo -e "Z3: ${GREEN}$(z3 --version 2>&1 | head -1)${NC}"
    else
        echo -e "Z3: ${RED}not found${NC}"
    fi

    # Dafny
    if command -v dafny &> /dev/null; then
        echo -e "Dafny: ${GREEN}$(dafny --version 2>&1 | head -1)${NC}"
    else
        echo -e "Dafny: ${RED}not found${NC}"
        all_ok=false
    fi

    # TLA+
    if [[ -f "$TOOLCHAIN_DIR/tla/tlatools.jar" ]]; then
        echo -e "TLA+: ${GREEN}$TOOLCHAIN_DIR/tla/tlatools.jar${NC}"
    else
        echo -e "TLA+: ${YELLOW}not installed${NC}"
    fi

    # Java
    if command -v java &> /dev/null; then
        echo -e "Java: ${GREEN}$(java -version 2>&1 | head -1)${NC}"
    else
        echo -e "Java: ${RED}not found${NC}"
    fi

    # Python/Formulog
    if command -v python3 &> /dev/null; then
        echo -e "Python: ${GREEN}$(python3 --version)${NC}"
        if command -v formulog &> /dev/null; then
            echo -e "Formulog: ${GREEN}$(formulog --version 2>&1 | head -1)${NC}"
        else
            echo -e "Formulog: ${YELLOW}not installed${NC}"
        fi
    else
        echo -e "Python: ${RED}not found${NC}"
    fi

    echo ""
    if $all_ok; then
        log_success "All required tools are installed"
        return 0
    else
        log_warn "Some tools are missing - see above"
        return 1
    fi
}

# =============================================================================
# Usage
# =============================================================================
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --check-only    Only verify installed tools, don't install"
    echo "  --all           Install all tools (default)"
    echo "  --dafny         Install only Dafny"
    echo "  --tla           Install only TLA+"
    echo "  --formulog      Install only Formulog"
    echo "  --dotnet        Install .NET SDK only"
    echo "  --verify        Verify installed tools"
    echo "  --help          Show this help"
    echo ""
    echo "Environment Variables:"
    echo "  INSTALL_DIR     Base installation directory (default: ~/.local)"
    echo "  CI              Set to 'true' for CI mode"
    echo ""
}

# =============================================================================
# Main
# =============================================================================
main() {
    local install_all=true
    local install_dafny_flag=false
    local install_tla_flag=false
    local install_formulog_flag=false
    local install_dotnet_flag=false
    local verify_only=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --check-only|--verify)
                verify_only=true
                shift
                ;;
            --all)
                install_all=true
                shift
                ;;
            --dafny)
                install_dafny_flag=true
                install_all=false
                shift
                ;;
            --tla)
                install_tla_flag=true
                install_all=false
                shift
                ;;
            --formulog)
                install_formulog_flag=true
                install_all=false
                shift
                ;;
            --dotnet)
                install_dotnet_flag=true
                install_all=false
                shift
                ;;
            --help|-h)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    echo "========================================"
    echo "SQLRustGo Formal Verification Toolchain"
    echo "========================================"
    echo ""

    if $verify_only; then
        verify_tools
        exit $?
    fi

    # Initialize PATH
    export PATH="$HOME/.dotnet/tools:$HOME/.dotnet:$TOOLCHAIN_DIR/tla:$HOME/.local/toolchain/tla:$PATH"
    export DOTNET_ROOT="$HOME/.dotnet"

    # Install .NET first (required for Dafny)
    if $install_all || $install_dotnet_flag || $install_dafny_flag; then
        install_dotnet || true
    fi

    # Install tools
    if $install_all || $install_dafny_flag; then
        install_dafny || true
    fi

    if $install_all || $install_tla_flag; then
        install_tla || true
    fi

    if $install_all || $install_formulog_flag; then
        install_formulog || true
    fi

    # Verify
    echo ""
    verify_tools
}

main "$@"