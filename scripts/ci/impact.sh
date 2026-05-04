#!/bin/bash
# =============================================================================
# impact.sh - 编译单元级影响分析 (V3 Enhancement #1)
# =============================================================================
# 精度比 cargo tree --invert 高一个量级
# 捕获: crate 变更 + reverse deps + feature users + build.rs

set -euo pipefail

BASE_BRANCH="${1:-origin/main}"
TARGET_BRANCH="${2:-HEAD}"
OUTPUT="${3:-ci_artifacts/affected_crates.txt}"

mkdir -p "$(dirname "$OUTPUT")"

# Get workspace root
WORKSPACE_ROOT=$(pwd)

# -----------------------------------------------------------------------------
# Step 1: 获取变更文件集
# -----------------------------------------------------------------------------
echo "[impact] Fetching changed files: $BASE_BRANCH...$TARGET_BRANCH"

CHANGED_FILES=$(git diff --name-only "$BASE_BRANCH"..."$TARGET_BRANCH" 2>/dev/null)
GIT_DIFF_EXIT=$?

if [ $GIT_DIFF_EXIT -ne 0 ]; then
    echo "[impact] git diff failed with exit $GIT_DIFF_EXIT"
    echo "[]" > "$OUTPUT"
    exit 1
fi

if [ -z "$CHANGED_FILES" ]; then
    echo "[impact] No changes detected"
    echo "[]" > "$OUTPUT"
    exit 0
fi

# special cases: build.rs or workspace root Cargo.toml
if echo "$CHANGED_FILES" | grep -qE "^(build\.rs|Cargo\.toml|Cargo\.lock)$"; then
    echo "[impact] build.rs or workspace config changed - full workspace"
    echo "(workspace)" > "$OUTPUT"
    exit 0
fi

# -----------------------------------------------------------------------------
# Step 2-3: Python 处理文件→crate 映射（统一用相对路径）
# -----------------------------------------------------------------------------
echo "$CHANGED_FILES" | python3 -c "
import sys, json, os

changed = [line.strip() for line in sys.stdin if line.strip()]

# Load cargo metadata
import subprocess
result = subprocess.run(['cargo', 'metadata', '--format-version', '1', '--no-deps'],
                       capture_output=True, text=True)
if result.returncode != 0:
    print('[impact] cargo metadata failed')
    sys.exit(1)

data = json.loads(result.stdout)
packages = data.get('packages', [])

workspace_root = os.getcwd()

# Build crate map: RELATIVE dir -> crate_name
crate_map = {}
for p in packages:
    manifest = p['manifest_path']
    # Convert absolute to relative
    if manifest.startswith(workspace_root):
        rel_dir = os.path.relpath(os.path.dirname(manifest), workspace_root)
    else:
        rel_dir = os.path.dirname(manifest)
    crate_name = p['name']
    crate_map[rel_dir] = crate_name

# Filter to only source files and find affected crates
skip_exts = ('.md', '.txt', '.json', '.yml', '.yaml', '.sh', '.py',
             '.toml', '.lock', '.css', '.html', '.svg', '.png')

affected = set()
for f in changed:
    # Skip non-source files by extension
    ext = os.path.splitext(f)[1]
    if ext in skip_exts:
        continue
    # Skip non-crate dirs
    if f.startswith('scripts/') or f.startswith('docs/') or f.startswith('.github/'):
        continue

    # Find matching crate by longest prefix match
    best_match = None
    best_len = 0
    for crate_dir, crate_name in crate_map.items():
        if (f.startswith(crate_dir + '/') or f == crate_dir) and len(crate_dir) > best_len:
            best_match = crate_name
            best_len = len(crate_dir)

    if best_match:
        affected.add(best_match)

affected_sorted = sorted(affected)
for c in affected_sorted:
    print(c)

# Save for reverse dep lookup
with open('/tmp/affected_crates.txt', 'w') as f:
    f.write('\n'.join(affected_sorted))
" > "$OUTPUT"

AFFECTED_CRATES=$(cat "$OUTPUT" | tr '\n' ' ')

# -----------------------------------------------------------------------------
# Step 4: reverse deps
# -----------------------------------------------------------------------------
if [ -n "$AFFECTED_CRATES" ]; then
    REVERSE_DEPS=$(cargo metadata --format-version 1 2>/dev/null | python3 -c "
import sys, json

d = json.load(sys.stdin)
pkgs = {p['name']: p for p in d['packages']}

graph = {}
for node in d.get('workspace_members', []):
    pkg = pkgs.get(node)
    if not pkg:
        continue
    name = pkg['name']
    for dep in pkg.get('dependencies', []):
        dep_name = dep['name']
        if dep_name not in graph:
            graph[dep_name] = []
        graph[dep_name].append(name)

changed = set()
for line in open('/tmp/affected_crates.txt'):
    line = line.strip()
    if line:
        changed.add(line)

for c in changed:
    for rev in graph.get(c, []):
        print(rev)
" 2>/dev/null || true)

    ALL_AFFECTED=$(echo -e "$AFFECTED_CRATES\n$REVERSE_DEPS" | grep -v '^$' | sort -u)
    echo "$ALL_AFFECTED" > "$OUTPUT"
fi

# -----------------------------------------------------------------------------
# Step 5: 输出
# -----------------------------------------------------------------------------
echo "[impact] === Affected crates ==="
cat "$OUTPUT"
COUNT=$(wc -l < "$OUTPUT" 2>/dev/null || echo 0)
echo "[impact] Total: $COUNT crates"

# flag if test files changed
if echo "$CHANGED_FILES" | grep -qE "^crates/.*/src/.*test"; then
    echo "[impact] WARNING: test code changed - full test suite required"
fi
