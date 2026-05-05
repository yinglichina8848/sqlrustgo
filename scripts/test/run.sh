#!/usr/bin/env bash
set -euo pipefail

MANIFEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/manifest" && pwd)"
MANIFEST="$MANIFEST_DIR/test_manifest.yaml"
FAILURES_FILE="$MANIFEST_DIR/known_failures.yaml"

DIMENSION=""
COMPONENT=""
TYPE=""
GROUP=""
TEST_NAME=""
LIST_ONLY=false
VERBOSE=false

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

usage() {
    cat << 'EOF'
SQLRustGo Unified Test Runner

Usage: ./run.sh [OPTIONS]

Options:
    --dimension DIM    correctness, performance, regression, security
    --component COMP   parser, planner, optimizer, executor, storage, etc.
    --type TYPE       unit, integration, e2e, stress, stability, sql_corpus, quality
    --group GROUP     correctness, performance, regression, stability, quick, full
    --name NAME       Run specific test by name
    --list            List all tests (don't run)
    --verbose         Show detailed output
    -h, --help        Show this help

Examples:
    ./run.sh --dimension correctness
    ./run.sh --component executor
    ./run.sh --group quick
    ./run.sh --name integration_aggregate
    ./run.sh --list
EOF
    exit 0
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dimension) DIMENSION="$2"; shift 2 ;;
            --component) COMPONENT="$2"; shift 2 ;;
            --type) TYPE="$2"; shift 2 ;;
            --group) GROUP="$2"; shift 2 ;;
            --name) TEST_NAME="$2"; shift 2 ;;
            --list) LIST_ONLY=true; shift ;;
            --verbose) VERBOSE=true; shift ;;
            -h|--help) usage ;;
            *) echo "Unknown: $1"; exit 1 ;;
        esac
    done
}

log() { echo -e "${BLUE}[runner]${NC} $1"; }
pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

run_python() {
    python3 - "$MANIFEST" "$FAILURES_FILE" "$DIMENSION" "$COMPONENT" "$TYPE" "$GROUP" "$TEST_NAME" "$LIST_ONLY" "$VERBOSE" << 'PYEOF'
import sys, yaml, subprocess

manifest_file = sys.argv[1]
failures_file = sys.argv[2]
dimension = sys.argv[3] if len(sys.argv) > 3 else ""
component = sys.argv[4] if len(sys.argv) > 4 else ""
type_filter = sys.argv[5] if len(sys.argv) > 5 else ""
group = sys.argv[6] if len(sys.argv) > 6 else ""
test_name = sys.argv[7] if len(sys.argv) > 7 else ""
list_only = sys.argv[8] == "true" if len(sys.argv) > 8 else False
verbose = sys.argv[9] == "true" if len(sys.argv) > 9 else False

with open(manifest_file) as f:
    data = yaml.safe_load(f)

tests = data.get("tests", [])

if group:
    group_map = data.get("groups", {})
    if group not in group_map:
        print(f"Unknown group: {group}")
        print(f"Available: {', '.join(group_map.keys())}")
        sys.exit(1)
    group_tests = group_map[group]
    test_names = []
    for gt in group_tests:
        if isinstance(gt, str):
            test_names.append(gt)
        elif isinstance(gt, dict):
            test_names.extend(gt.get('tests', []))
    tests = [t for t in tests if t['name'] in test_names]

filtered = []
for t in tests:
    if test_name and t.get("name") != test_name:
        continue
    if dimension and t.get("dimension") != dimension:
        continue
    if component and t.get("component") != component:
        continue
    if type_filter and t.get("type") != type_filter:
        continue
    filtered.append(t)

tests = filtered

if list_only:
    print(f"SQLRustGo Test Manifest ({len(tests)} tests)\n")
    for t in tests:
        print(f"  {t['name']}")
        print(f"    dim={t.get('dimension')} comp={t.get('component')} type={t.get('type')}: {t.get('description','')}")
        print()
    sys.exit(0)

known_failures = set()
if failures_file:
    try:
        with open(failures_file) as f:
            fd = yaml.safe_load(f)
            for k in fd.get("failures", []):
                known_failures.add(k.get("name", ""))
    except:
        pass

passed = 0
failed = 0
failed_tests = []

for t in tests:
    name = t.get("name", "")
    cmd = t.get("command", "")
    desc = t.get("description", "")

    if not cmd:
        print(f"[FAIL] {name} (no command)")
        failed += 1
        failed_tests.append(name)
        continue

    if name in known_failures:
        print(f"[WARN] {name} SKIP (known failure)")
        continue

    print(f"[{name}] ", end="", flush=True)

    try:
        result = subprocess.run(
            cmd, shell=True, capture_output=True, timeout=300
        )
        if result.returncode == 0:
            print("PASS")
            passed += 1
        else:
            print("FAIL")
            failed += 1
            failed_tests.append(name)
    except subprocess.TimeoutExpired:
        print("TIMEOUT")
        failed += 1
        failed_tests.append(name)
    except Exception as e:
        print(f"ERROR: {e}")
        failed += 1
        failed_tests.append(name)

print(f"\nResults: {passed} passed, {failed} failed (of {len(tests)})")
if failed_tests:
    print(f"Failed: {', '.join(failed_tests)}")
    sys.exit(1)
PYEOF
}

main() {
    parse_args "$@"

    if [[ ! -f "$MANIFEST" ]]; then
        echo -e "${RED}Error: Manifest not found: $MANIFEST${NC}"
        exit 1
    fi

    if [[ "$LIST_ONLY" == true ]]; then
        log "SQLRustGo Test Manifest"
    else
        log "SQLRustGo Test Runner"
    fi
    [[ -n "$DIMENSION" ]] && log "Dimension: $DIMENSION"
    [[ -n "$COMPONENT" ]] && log "Component: $COMPONENT"
    [[ -n "$TYPE" ]] && log "Type: $TYPE"
    [[ -n "$GROUP" ]] && log "Group: $GROUP"
    echo ""
    run_python
}

main "$@"
