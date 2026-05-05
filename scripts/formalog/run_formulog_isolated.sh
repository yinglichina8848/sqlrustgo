#!/usr/bin/env bash
# =============================================================================
# Formulog Isolated Runner - One-shot container per proof
# =============================================================================
# Solves the SymbolManager JVM pollution problem by running each proof
# in an isolated Docker container. No state is shared between runs.
#
# Usage:
#   scripts/formulog/run_formulog_isolated.sh <proof.formulog>
#
# Environment:
#   FORMULOG_JAR   - Path to formulog.jar (default: /tmp/formulog-0.8.0.jar)
#   Z3_PATH        - Path to Z3 binary (default: /usr/bin/z3)
#   TIMEOUT_SECS   - Max time per proof (default: 60)
#
# Requirements:
#   - Docker
#   - Z3 installed on host (mounted into container)
# =============================================================================

set -euo pipefail

FORMULOG_JAR="${FORMULOG_JAR:-/tmp/formulog-0.8.0.jar}"
Z3_PATH="${Z3_PATH:-/usr/bin/z3}"
TIMEOUT_SECS="${TIMEOUT_SECS:-60}"
CONTAINER_IMAGE="${CONTAINER_IMAGE:-tla-java17:latest}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info()  { echo -e "\033[0;34m[INFO]\033[0m $1"; }
log_ok()    { echo -e "\033[0;32m[OK]\033[0m $1"; }
log_warn()  { echo -e "\033[0;33m[WARN]\033[0m $1"; }
log_error() { echo -e "\033[0;31m[ERROR]\033[0m $1"; }

usage() {
    echo "Usage: $0 <proof.formulog>"
    echo ""
    echo "Run a Formulog proof in an isolated Docker container."
    echo ""
    echo "Example:"
    echo "  $0 docs/proof/PROOF-020-null-three-valued-logic.formulog"
    exit 1
}

validate() {
    if [[ ! -f "$FORMULOG_JAR" ]]; then
        log_error "Formulog JAR not found: $FORMULOG_JAR"
        echo "  Run: curl -sSL https://github.com/ucsd-progsys/formulog/releases/download/v0.8.0/formulog-0.8.0.jar -o /tmp/formulog-0.8.0.jar"
        exit 1
    fi

    if [[ ! -f "$Z3_PATH" ]]; then
        log_error "Z3 not found: $Z3_PATH"
        echo "  Run: apt install z3  OR  brew install z3"
        exit 1
    fi

    if ! command -v docker &> /dev/null; then
        log_error "Docker is required but not found"
        exit 1
    fi
}

main() {
    if [[ $# -eq 0 ]]; then
        usage
    fi

    local proof_file="$1"
    if [[ ! -f "$proof_file" ]]; then
        log_error "Proof file not found: $proof_file"
        exit 1
    fi

    local proof_name
    proof_name="$(basename "$proof_file" .formulog)"

    validate

    log_info "Running $proof_name in isolated container..."

    # Create a unique temporary directory for this run
    local run_dir
    run_dir="$(mktemp -d /tmp/formulog-run-XXXXXX)"
    local proof_copy="$run_dir/${proof_name}.formulog"

    # Strip comments from proof file (Formulog 0.8.0 doesn't support // or /* */)
    sed 's|//.*$||g; s|/\*.*\*/||g' "$proof_file" > "$proof_copy"

    # Build Docker command
    local docker_cmd=(
        docker run --rm
        --read-only
        -v "$FORMULOG_JAR:/formulog.jar:ro"
        -v "$Z3_PATH:/usr/bin/z3:ro"
        -v "$run_dir:/work:ro"
        -w /work
        --network none
        --user "$(id -u):$(id -g)"
        "$CONTAINER_IMAGE"
    )

    # Run with timeout
    local start_time
    start_time=$(date +%s)

    if timeout "$TIMEOUT_SECS" "${docker_cmd[@]}" java -jar /formulog.jar "/work/${proof_name}.formulog" --eager-eval > "$run_dir/${proof_name}.out" 2>&1; then
        local end_time
        end_time=$(date +%s)
        local duration=$((end_time - start_time))

        if grep -q "All predicates passed\|Result: Success\|verified" "$run_dir/${proof_name}.out" 2>/dev/null; then
            log_ok "$proof_name PASSED (${duration}s)"
            cat "$run_dir/${proof_name}.out"
        else
            log_warn "$proof_name completed but output unclear:"
            cat "$run_dir/${proof_name}.out"
            log_warn "Manual review required"
        fi
    else
        local exit_code=$?
        local end_time
        end_time=$(date +%s)
        local duration=$((end_time - start_time))

        if [[ $exit_code -eq 124 ]]; then
            log_error "$proof_name TIMEOUT after ${TIMEOUT_SECS}s"
        else
            log_error "$proof_name FAILED (exit $exit_code, ${duration}s)"
        fi

        if [[ -f "$run_dir/${proof_name}.out" ]]; then
            echo "--- Output ---"
            cat "$run_dir/${proof_name}.out"
        fi

        # Keep run dir for debugging on failure
        log_info "Run dir preserved: $run_dir"
        exit 1
    fi

    # Cleanup on success
    rm -rf "$run_dir"
    log_info "Run dir cleaned up: $run_dir"
}

main "$@"
